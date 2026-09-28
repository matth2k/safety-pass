module AND(
    input A,
    input B,
    output Y
);
wire n2;
assign n2 = A & B;
assign Y = n2;
endmodule

module NAND(
    input A,
    input B,
    output Y
);
wire n2;
wire n3;
assign n2 = A & B;
assign n3 = ~n2;
assign Y = n3;
endmodule

module OR(
    input A,
    input B,
    output Y
);
wire n2;
wire n3;
wire n4;
wire n5;
assign n2 = ~A;
assign n3 = ~B;
assign n4 = n2 & n3;
assign n5 = ~n4;
assign Y = n5;
endmodule

module NOR(
    input A,
    input B,
    output Y
);
wire n2;
wire n3;
wire n4;
wire n5;
wire n6;
assign n2 = ~A;
assign n3 = ~B;
assign n4 = n2 & n3;
assign n5 = ~n4;
assign n6 = ~n5;
assign Y = n6;
endmodule

module XOR(
    input A,
    input B,
    output Y
);
wire n2;
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n2 = ~A;
assign n3 = ~B;
assign n4 = A & n3;
assign n5 = n2 & B;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign Y = n9;
endmodule

module XNOR(
    input A,
    input B,
    output Y
);
wire n2;
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
assign n2 = ~A;
assign n3 = ~B;
assign n4 = A & n3;
assign n5 = n2 & B;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign n10 = ~n9;
assign Y = n10;
endmodule

module NOT(
    input A,
    output Y
);
wire n1;
assign n1 = ~A;
assign Y = n1;
endmodule

module INV(
    input A,
    output ZN
);
wire n1;
assign n1 = ~A;
assign ZN = n1;
endmodule

module AND2(
    input A1,
    input A2,
    output ZN
);
wire n2;
assign n2 = A1 & A2;
assign ZN = n2;
endmodule

module NAND2(
    input A1,
    input A2,
    output ZN
);
wire n2;
wire n3;
assign n2 = A1 & A2;
assign n3 = ~n2;
assign ZN = n3;
endmodule

module OR2(
    input A1,
    input A2,
    output ZN
);
wire n2;
wire n3;
wire n4;
wire n5;
assign n2 = ~A1;
assign n3 = ~A2;
assign n4 = n2 & n3;
assign n5 = ~n4;
assign ZN = n5;
endmodule

module NOR2(
    input A1,
    input A2,
    output ZN
);
wire n2;
wire n3;
wire n4;
wire n5;
wire n6;
assign n2 = ~A1;
assign n3 = ~A2;
assign n4 = n2 & n3;
assign n5 = ~n4;
assign n6 = ~n5;
assign ZN = n6;
endmodule

module XOR2(
    input A,
    input B,
    output Z
);
wire n2;
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n2 = ~A;
assign n3 = ~B;
assign n4 = A & n3;
assign n5 = n2 & B;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign Z = n9;
endmodule

module XNOR2(
    input A,
    input B,
    output ZN
);
wire n2;
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
assign n2 = ~A;
assign n3 = ~B;
assign n4 = A & n3;
assign n5 = n2 & B;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign n10 = ~n9;
assign ZN = n10;
endmodule

module AND3(
    input A1,
    input A2,
    input A3,
    output ZN
);
wire n3;
wire n4;
assign n3 = A1 & A2;
assign n4 = n3 & A3;
assign ZN = n4;
endmodule

module NAND3(
    input A1,
    input A2,
    input A3,
    output ZN
);
wire n3;
wire n4;
wire n5;
assign n3 = A1 & A2;
assign n4 = n3 & A3;
assign n5 = ~n4;
assign ZN = n5;
endmodule

module OR3(
    input A1,
    input A2,
    input A3,
    output ZN
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
assign n3 = ~A1;
assign n4 = ~A2;
assign n5 = n3 & n4;
assign n6 = ~n5;
assign n7 = ~n6;
assign n8 = ~A3;
assign n9 = n7 & n8;
assign n10 = ~n9;
assign ZN = n10;
endmodule

module NOR3(
    input A1,
    input A2,
    input A3,
    output ZN
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
assign n3 = ~A1;
assign n4 = ~A2;
assign n5 = n3 & n4;
assign n6 = ~n5;
assign n7 = ~n6;
assign n8 = ~A3;
assign n9 = n7 & n8;
assign n10 = ~n9;
assign n11 = ~n10;
assign ZN = n11;
endmodule

module AND4(
    input A1,
    input A2,
    input A3,
    input A4,
    output ZN
);
wire n4;
wire n5;
wire n6;
assign n4 = A1 & A2;
assign n5 = n4 & A3;
assign n6 = n5 & A4;
assign ZN = n6;
endmodule

module NAND4(
    input A1,
    input A2,
    input A3,
    input A4,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
assign n4 = A1 & A2;
assign n5 = n4 & A3;
assign n6 = n5 & A4;
assign n7 = ~n6;
assign ZN = n7;
endmodule

module OR4(
    input A1,
    input A2,
    input A3,
    input A4,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
wire n14;
wire n15;
assign n4 = ~A1;
assign n5 = ~A2;
assign n6 = n4 & n5;
assign n7 = ~n6;
assign n8 = ~n7;
assign n9 = ~A3;
assign n10 = n8 & n9;
assign n11 = ~n10;
assign n12 = ~n11;
assign n13 = ~A4;
assign n14 = n12 & n13;
assign n15 = ~n14;
assign ZN = n15;
endmodule

module NOR4(
    input A1,
    input A2,
    input A3,
    input A4,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
wire n14;
wire n15;
wire n16;
assign n4 = ~A1;
assign n5 = ~A2;
assign n6 = n4 & n5;
assign n7 = ~n6;
assign n8 = ~n7;
assign n9 = ~A3;
assign n10 = n8 & n9;
assign n11 = ~n10;
assign n12 = ~n11;
assign n13 = ~A4;
assign n14 = n12 & n13;
assign n15 = ~n14;
assign n16 = ~n15;
assign ZN = n16;
endmodule

module MUX(
    input S,
    input A,
    input B,
    output Y
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n3 = ~S;
assign n4 = n3 & A;
assign n5 = S & B;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign Y = n9;
endmodule

module MUX2(
    input S,
    input B,
    input A,
    output Z
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n3 = ~S;
assign n4 = n3 & A;
assign n5 = S & B;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign Z = n9;
endmodule

module MUXF7(
    input S,
    input I1,
    input I0,
    output O
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n3 = ~S;
assign n4 = n3 & I0;
assign n5 = S & I1;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign O = n9;
endmodule

module MUXF8(
    input S,
    input I1,
    input I0,
    output O
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n3 = ~S;
assign n4 = n3 & I0;
assign n5 = S & I1;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign O = n9;
endmodule

module MUXF9(
    input S,
    input I1,
    input I0,
    output O
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
assign n3 = ~S;
assign n4 = n3 & I0;
assign n5 = S & I1;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign O = n9;
endmodule

module MAJ3(
    input A1,
    input A2,
    input A3,
    output ZN
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
assign n3 = A1 & A2;
assign n4 = A1 & A3;
assign n5 = A2 & A3;
assign n6 = ~n3;
assign n7 = ~n4;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign n10 = ~n9;
assign n11 = ~n5;
assign n12 = n10 & n11;
assign n13 = ~n12;
assign ZN = n13;
endmodule

module AOI21(
    input A,
    input B1,
    input B2,
    output ZN
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
assign n3 = B1 & B2;
assign n4 = ~n3;
assign n5 = ~A;
assign n6 = n4 & n5;
assign n7 = ~n6;
assign n8 = ~n7;
assign ZN = n8;
endmodule

module OAI21(
    input A,
    input B1,
    input B2,
    output ZN
);
wire n3;
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
assign n3 = ~B1;
assign n4 = ~B2;
assign n5 = n3 & n4;
assign n6 = ~n5;
assign n7 = n6 & A;
assign n8 = ~n7;
assign ZN = n8;
endmodule

module AOI211(
    input A,
    input B,
    input C1,
    input C2,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
assign n4 = C1 & C2;
assign n5 = ~n4;
assign n6 = ~B;
assign n7 = n5 & n6;
assign n8 = ~n7;
assign n9 = ~n8;
assign n10 = ~A;
assign n11 = n9 & n10;
assign n12 = ~n11;
assign n13 = ~n12;
assign ZN = n13;
endmodule

module AOI22(
    input A1,
    input A2,
    input B1,
    input B2,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
assign n4 = A1 & A2;
assign n5 = B1 & B2;
assign n6 = ~n4;
assign n7 = ~n5;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign n10 = ~n9;
assign ZN = n10;
endmodule

module OAI211(
    input A,
    input B,
    input C1,
    input C2,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
assign n4 = ~C1;
assign n5 = ~C2;
assign n6 = n4 & n5;
assign n7 = ~n6;
assign n8 = n7 & B;
assign n9 = n8 & A;
assign n10 = ~n9;
assign ZN = n10;
endmodule

module OAI22(
    input A1,
    input A2,
    input B1,
    input B2,
    output ZN
);
wire n4;
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
assign n4 = ~A1;
assign n5 = ~A2;
assign n6 = n4 & n5;
assign n7 = ~n6;
assign n8 = ~B1;
assign n9 = ~B2;
assign n10 = n8 & n9;
assign n11 = ~n10;
assign n12 = n7 & n11;
assign n13 = ~n12;
assign ZN = n13;
endmodule

module OAI221(
    input A,
    input B1,
    input B2,
    input C1,
    input C2,
    output ZN
);
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
wire n14;
wire n15;
assign n5 = ~B1;
assign n6 = ~B2;
assign n7 = n5 & n6;
assign n8 = ~n7;
assign n9 = ~C1;
assign n10 = ~C2;
assign n11 = n9 & n10;
assign n12 = ~n11;
assign n13 = n8 & n12;
assign n14 = n13 & A;
assign n15 = ~n14;
assign ZN = n15;
endmodule

module AOI221(
    input A,
    input B1,
    input B2,
    input C1,
    input C2,
    output ZN
);
wire n5;
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
wire n14;
wire n15;
assign n5 = B1 & B2;
assign n6 = C1 & C2;
assign n7 = ~n5;
assign n8 = ~n6;
assign n9 = n7 & n8;
assign n10 = ~n9;
assign n11 = ~n10;
assign n12 = ~A;
assign n13 = n11 & n12;
assign n14 = ~n13;
assign n15 = ~n14;
assign ZN = n15;
endmodule

module OAI222(
    input A1,
    input A2,
    input B1,
    input B2,
    input C1,
    input C2,
    output ZN
);
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
wire n14;
wire n15;
wire n16;
wire n17;
wire n18;
wire n19;
wire n20;
assign n6 = ~A1;
assign n7 = ~A2;
assign n8 = n6 & n7;
assign n9 = ~n8;
assign n10 = ~B1;
assign n11 = ~B2;
assign n12 = n10 & n11;
assign n13 = ~n12;
assign n14 = ~C1;
assign n15 = ~C2;
assign n16 = n14 & n15;
assign n17 = ~n16;
assign n18 = n9 & n13;
assign n19 = n18 & n17;
assign n20 = ~n19;
assign ZN = n20;
endmodule

module AOI222(
    input A1,
    input A2,
    input B1,
    input B2,
    input C1,
    input C2,
    output ZN
);
wire n6;
wire n7;
wire n8;
wire n9;
wire n10;
wire n11;
wire n12;
wire n13;
wire n14;
wire n15;
wire n16;
wire n17;
assign n6 = A1 & A2;
assign n7 = B1 & B2;
assign n8 = C1 & C2;
assign n9 = ~n6;
assign n10 = ~n7;
assign n11 = n9 & n10;
assign n12 = ~n11;
assign n13 = ~n12;
assign n14 = ~n8;
assign n15 = n13 & n14;
assign n16 = ~n15;
assign n17 = ~n16;
assign ZN = n17;
endmodule


