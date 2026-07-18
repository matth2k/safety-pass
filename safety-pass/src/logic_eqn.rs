use std::fmt;

/// A single node of a logic equation
pub enum Node {
    /// A named input wire
    Input(String),

    /// The AND of two earlier nodes (references by index)
    And(usize, usize),

    /// The INV of an earlier node
    Inv(usize),
}

/// Topologically sorted vector of nodes, represents a logic function
pub struct LogicEqn {

    nodes: Vec<Node>,
}

impl LogicEqn {
    /// Creates a new, empty logic equation
    pub fn new() -> LogicEqn { 
        LogicEqn{
            nodes: Vec::new() // Default constructor
        }
    }
    /// Returns a reference to the internal node list
    pub fn nodes(&self) -> &Vec<Node> { 
        &self.nodes
    }

    /// Adds a node to the end of the list and  returns the index.
    pub fn push(&mut self, node: Node) -> usize {
         // TODO; add a check for validty of nodes being inserted
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Adds a named input node, returns the index
    pub fn input(&mut self, name: &str) ->usize {
        self.push(Node::Input(name.to_string() ))
    }

    /// Adds an AND node between two previous indicies (a, b) and returns the index of the AND node
    pub fn and(&mut self, a: usize, b: usize) -> usize {
        self.push(Node::And(a, b))
    }
    /// Adds an INV node of a previous index(a), and returns the index of the INV node 
    pub fn inv(&mut self, a: usize) -> usize {
        self.push(Node::Inv(a))
    }

    /// Returns the index of the last node, the output
    pub fn output(&self) -> Option<usize> {
        if self.nodes.len() == 0 {
            return None;
        }
        Some(self.nodes.len() - 1)
    }

    /// Return a vector of all the node names, in order
    pub fn input_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for node in &self.nodes {
            if let Node::Input(name) = node {
                names.push(name.clone());
            }
        }
        return names;
    }
    }

// Builds a logic equation from a list of nodes
impl From<Vec<Node>> for LogicEqn {
    fn from(nodes: Vec<Node>) -> LogicEqn {
        let mut eqn = LogicEqn::new();
        for node in nodes {
            eqn.push(node);
        }
        return eqn;
    }
}

// inputs keep their names, other stuff becomes n{index}
fn net_name(eqn: &LogicEqn, idx: usize) -> String {
    let node = &eqn.nodes[idx];

    if let Node::Input(name) = node {
        return name.clone();
    }
    return format!("n{}", idx);
}

impl fmt::Display for LogicEqn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // declare wire for all non input nodes
        let mut i = 0;
        while i < self.nodes.len() {
            let node = &self.nodes[i];

            let is_input = if let Node::Input(_) = node {
                true
            } else {
                false
            };

            if is_input == false {
                write!(f, "wire {};\n", net_name(self, i))?;
            }
            i = i + 1;
        }
        //write the actual logic for each node
        let mut j = 0;
        while j < self.nodes.len() {
            let node = &self.nodes[j];

            if let Node::And(a, b) = node {
                let slot_name = net_name(self, j);
                let input_a = net_name(self, *a);
                let input_b = net_name(self, *b);
                write!(f, "assign {} = {} & {};\n", slot_name, input_a, input_b)?;
            }
            if let Node::Inv(a) = node {
                let slot_a = net_name(self, j);
                let input_a = net_name(self, *a);
                write!(f, "assign {} = ~{};\n", slot_a, input_a)?;
                }

            j = j + 1;
        }
        return Ok(());
    }
}