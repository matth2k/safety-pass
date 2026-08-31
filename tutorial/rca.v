`include "cells.v"

module rca (
    input wire [3:0] a,
    input wire [3:0] b,
    output wire [4:0] sum
);

  // Desired functionality:
  // assign sum = a + b;

  wire [2:0] carry;

  FA fa_0 (
      .A (a[0]),
      .B (b[0]),
      .CI(1'b0),
      .S (sum[0]),
      .CO(carry[0])
  );

  FA fa_1 (
      .A (a[1]),
      .B (b[1]),
      .CI(carry[0]),
      .S (sum[1]),
      .CO(carry[1])
  );

  FA fa_2 (
      .A (a[2]),
      .B (b[2]),
      .CI(carry[1]),
      .S (sum[2]),
      .CO(carry[2])
  );

  FA fa_3 (
      .A (a[3]),
      .B (b[3]),
      //    .CI(carry[2]),
      .S (sum[3]),
      .CO(sum[4])
  );

endmodule
