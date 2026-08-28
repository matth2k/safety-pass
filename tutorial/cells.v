
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

module MUX2 (
    input  wire S,
    input  wire B,
    input  wire A,
    output wire Z
);

  assign Z = S ? B : A;

endmodule
