RVDIR 		= /home/kuangjux/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin
TARGET		= riscv64-unknown-elf-

CC			= $(RVDIR)/$(TARGET)gcc
LD 			= $(RVDIR)/$(TARGET)ld
AS 			= $(RVDIR)/$(TARGET)as
OBJCOPY		= $(RVDIR)/$(TARGET)objcopy 
OBJDUMP		= $(RVDIR)/$(TARGET)objdump
CFLAGS      = -Og

CLANG		= clang-10
LLVM_AS		= llvm-as-10
QEMU		= qemu-riscv64

TEST		= testcases
TESTASM     = $(TEST)/test.S
TESTPROG	= $(TEST)/test.c
TESTOUT     = $(TEST)/test.o
TESTELF		= $(TEST)/test

ASMS  		= $(wildcard $(TEST)/*.S)
OBJS 		= $(wildcard $(TEST)/*.o)
ELFS 		= $(TEST)/test $(TEST)hello_world

TEST_LL		= test.ll


ifeq ($(TARGET),riscv64)
	FEATURES += --features riscv64
endif 

ifeq ($(TARGET),riscv32)
	FEATURES += --features riscv32
endif

ifeq ($(TARGET),riscv32_test)
	FEATURES += --features riscv32_test
endif

ifeq ($(TARGET),riscv64_test)
	FEATURES += --features riscv64_test 
endif

.PHONY: run build
build:
	@cargo build

run: 
	@cargo run $(FEATURES)

exe:
	@$(AS) -c main.S -o main
	@$(QEMU) main

gen_ir:
	@$(CLANG) -S -emit-llvm $(TEST)/test.c

gen_bc:
	@$(LLVM_AS) test.ll -o test.bc

gen_asm: $(TEST)/test.c
	@$(CC) -S $(CFLAGS) $(TEST)/test.c -o $(TESTASM)

display_compile:
	@$(CC) $(TESTPROG) $(CFLAGS) -v -o $(TESTELF)

gen_exe: 
	@$(CC) $(TESTPROG) $(CFLAGS) -o $(TESTELF)

qemu_test: $(TESTELF) 
	@$(QEMU) $(TESTELF)

clean:
	@cargo clean 
	@rm $(ASMS)
	@rm $(OBJS)
	@rm $(ELF)


