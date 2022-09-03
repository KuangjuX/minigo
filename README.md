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

## RoadMap
### Global Variable
- [x] I32
- [x] I64
- [x] Pointer
- [ ] Array
- [ ] Structure
- [ ] Enum

### Function 
- [x] Parameters into Stack
- [ ] Local Varaiables into Stack
- [ ] Exprs generation

## References:
- [chibicc](https://github.com/rui314/chibicc)
- [minidecaf](https://decaf-lang.github.io/minidecaf-tutorial/)