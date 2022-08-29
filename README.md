# MinimalGo

## Introduction

MinimalGo is a minimal go compiler. MinimalGo compiler backend is written by Rust, it receive llvm ir as input and translate them into RISC -V assembly. It is deeply inspired by [chibicc](https://github.com/rui314/chibicc).  
  
In this project, compiler backend will receive llvm-10 file and translate them into RISC -V asembly and `riscv64-unknown-elf-as` will turn assemble file into elf file. `qemu-riscv64` will enumlate RISC -V 恶女ironment to run elf.

```
Compiler Frontend ---> Compiler Middlend ---> LLVM IR ---> Compiler Backend --> AS --> ELF
```