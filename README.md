# MinimalGo

## Introduction

MinimalGo is a minimal go compiler. MinimalGo compiler backend is written by Rust, it receive llvm ir as input and translate them into RISC -V assembly. It is deeply inspired by [chibicc](https://github.com/rui314/chibicc).  
  
In this project, compiler backend will receive llvm-10 file and translate them into RISC -V asembly and `riscv64-unknown-elf-as` will turn assemble file into elf file. `qemu-riscv64` will enumlate RISC -V Environment to run elf.

```
Compiler Frontend ---> Compiler Middlend ---> LLVM IR ---> Compiler Backend --> AS --> ELF
```

## Environment
- Rust
- QEMU 
```
git clone https://mirrors.tuna.tsinghua.edu.cn/git/qemu.git
cd qemu && ./configure --prefix=/usr/local --target-list=riscv64-linux-user
make
make install
qemu-riscv64 --version
```
- RISC -V ToolChains

## Usage
You can use following commond to run this project and generate RISC-V64 assembly:
  
```
make run PROG={your example}
```  

## Debug

**One Terminal:**
```shell
qemu-riscv64 -g 1234 testcases/{ELF}
```

**Other Terminal:**
```shell
riscv64-unknown-elf-gdb testcases/{E:F}
target remote localhost:1234
```

  
We suppply some test examples under testcases directory

## RoadMap
- [x] Return `main` function
- [x] Unary Experssions
- [ ] Add, Sub, Mul, Div,Mod
- [ ] Compare && Logical Experssions
- [ ] Local Variables and Assignment
- [ ] Conditional Expressions
- [ ] Scope && Block Statements
- [ ] Loop Statement
- [ ] Functions
- [x] Global Variables
- [ ] Array

## References:
- [chibicc](https://github.com/rui314/chibicc)
- [minidecaf](https://decaf-lang.github.io/minidecaf-tutorial/)