#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
/*!

Extra features for nl_opt

*/

use ascii_dag::render::colors::Palette;
use ascii_dag::{Graph, LayoutConfig, RenderMode};
use safety_net::Instantiable;
use safety_pass::passes::*;
use safety_pass::{Cell, Pass, register_passes};
use std::collections::HashSet;
use std::fmt::{Debug, Display};

/// Produce an ASCII graph of the netlist.
pub struct ASCIIGraph<I: Instantiable>(std::marker::PhantomData<I>);

impl<I: Instantiable> Debug for ASCIIGraph<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ASCIIGraph")
    }
}

impl<I: Instantiable> Display for ASCIIGraph<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ASCIIGraph")
    }
}

impl<I: Instantiable> Pass for ASCIIGraph<I> {
    type I = I;

    fn run(
        &self,
        netlist: &std::rc::Rc<safety_net::Netlist<Self::I>>,
    ) -> Result<String, safety_net::Error> {
        use safety_net::graph::{MultiDiGraph, Node};
        let analysis = netlist.get_analysis::<MultiDiGraph<Self::I>>()?;
        let pg = analysis.get_graph();
        let mut graph = Graph::new();

        let mut node_labels = Vec::new();
        let mut skip_nodes = HashSet::new();
        for node in pg.node_indices() {
            let n = &pg[node];

            // Skip constant input wires
            if let Node::NetRef(nr) = n
                && nr
                    .get_instance_type()
                    .is_some_and(|t| t.get_constant().is_some())
            {
                skip_nodes.insert(node);
            }

            node_labels.push(pg[node].to_string());
        }

        for node in pg.node_indices() {
            if skip_nodes.contains(&node) {
                continue;
            }
            let label = &node_labels[node.index()];
            graph.add_node(node.index(), label);
        }

        let mut edge_labels = Vec::new();
        for edge in pg.edge_indices() {
            edge_labels.push(pg[edge].to_string());
        }

        for edge in pg.edge_indices() {
            let (a, b) = pg.edge_endpoints(edge).unwrap();
            if skip_nodes.contains(&a) || skip_nodes.contains(&b) {
                continue;
            }
            let label = &edge_labels[edge.index()];
            graph.add_edge(a.index(), b.index(), Some(label));
        }

        use ascii_dag::{
            AlgorithmConfig, CrossingReducer, CycleBreaking, Layering, Positioning, Routing,
        };
        let ir = graph.compute_layout_with_config(&LayoutConfig {
            algorithm: AlgorithmConfig::Sugiyama {
                cycle_breaking: CycleBreaking::DepthFirst,
                layering: Layering::LongestPath,
                crossing_pipeline: &[
                    CrossingReducer::Median(8),
                    CrossingReducer::AdjacentExchange(4),
                    CrossingReducer::Median(2),
                ],
                positioning: Positioning::Compact,
            },
            routing: Routing::Direct,
            node_spacing: 4,
            level_spacing: 3,
            render_mode: RenderMode::Vertical,
            include_dummy_nodes: false,
            skip_validation: false,
        });

        Ok(ir.render_scanline_colored(Palette::AnsiLight))
    }
}

register_passes!(OptPasses<Cell>;
    /// Produce a netlist graph in ASCII form
    ASCIIGraph<Cell>,
    /// Prints stats on all the cell types in the netlist.
    CellStats<Cell>,
    /// A pass that cleans the netlist.
    Clean<Cell>,
    /// A pass that prints the dot graph of the netlist.
    DotGraph<Cell>,
    /// A pass that runs all built-in patterns to a fixed point.
    FoldAllPatterns,
    /// Insert a pair of inverters at every point in the netlist.
    InsertInv,
    /// A dummy pass that emits the Verilog of the netlist.
    PrintVerilog<Cell>,
    /// A pass that renames wires and instances sequentially __0__, __1__, ...
    RenameNets<Cell>,
);
