#include "Vrca.h"
#include "verilated.h"
int main(int argc, char **argv) {
  VerilatedContext *contextp = new VerilatedContext;
  contextp->commandArgs(argc, argv);
  Vrca *top = new Vrca{contextp};

  // Test addition
  top->subtract = 0;
  for (int i = 0; i < 16; ++i) {
    for (int j = 0; j < 16; ++j) {
      top->a = i;
      top->b = j;
      top->eval();
      if (top->sum != (i + j)) {
        printf("ERROR: %d + %d != %d\n", i, j, top->sum);
        delete top;
        delete contextp;
        return 1;
      } else {
        printf("OK: %d + %d = %d\n", i, j, top->sum);
      }
    }
  }

  // Test subtraction
  top->subtract = 1;
  for (int i = 0; i < 16; ++i) {
    for (int j = 0; j < 16; ++j) {
      top->a = i;
      top->b = j;
      top->eval();
      int shift = sizeof(int) * 8 - 4;
      int result = ((top->sum) << shift) >> shift;
      if (result != (i - j)) {
        printf("ERROR: %d - %d != %d\n", i, j, result);
        delete top;
        delete contextp;
        return 1;
      } else {
        printf("OK: %d - %d = %d\n", i, j, result);
      }
    }
  }
  delete top;
  delete contextp;
  return 0;
}