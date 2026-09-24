
module FA (
    input  wire A,
    input  wire B,
    input  wire CI,
    output wire S,
    output wire CO
);

  assign S  = A ^ B ^ CI;
  assign CO = (A & B) | (CI & (A ^ B));

endmodule

module XOR2 (
    input  wire A,
    input  wire B,
    output wire Z
);

  assign Z = A ^ B;

endmodule

module OR2 (
    input  wire A1,
    input  wire A2,
    output wire ZN
);

  assign ZN = A1 | A2;

endmodule

module AND2 (
    input  wire A1,
    input  wire A2,
    output wire ZN
);

  assign ZN = A1 & A2;

endmodule
