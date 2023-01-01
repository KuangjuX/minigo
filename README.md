# minigo

## Introduction

minigo is a minimal go compiler. minigo compiler backend is written by Rust, it receive llvm ir as input and translate them into RISC -V assembly. It is deeply inspired by [chibicc](https://github.com/rui314/chibicc).  
  
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
riscv64-unknown-elf-gdb testcases/{ELF}
target remote localhost:1234
```

  
We suppply some test examples under testcases directory

## Supported Instructions:
- `ret` instruction
- `xor` instruction
- `add` instruction
- `sub` instruction
- `mul` instruction
- `sdiv` instruction
- `alloca` instruction
- `load` instruction
- `store` instruction
- `icmp` instruction
- `br` instruction
- `zext` instruction
- `call` instruction


## RoadMap
- [x] Return `main` function
- [x] Unary Experssions
- [x] Add, Sub, Mul, Div,Mod
- [x] Compare && Logical Experssions
- [ ] Local Variables and Assignment
- [x] Conditional Expressions
- [ ] Scope && Block Statements
- [x] Loop Statement
- [ ] Functions
- [x] Global Variables
- [ ] Array

## TODO
- [ ] register allocation
- [ ] many parameters function call
- [ ] labels generator

## References:
- [chibicc](https://github.com/rui314/chibicc)
- [minidecaf](https://decaf-lang.github.io/minidecaf-tutorial/)