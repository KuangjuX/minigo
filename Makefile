PATH 		= /home/kuangjux/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin
TARGET		= riscv64

CC			= $(PATH)/$(TARGET)-unknown-elf-gcc
LD 			= $(PATH)/$(TARGET)-unknown-elf-ld
AS 			= $(PATH)/$(TARGET)-unknown-elf-as
OBJCOPY		= $(PATH)/$(TARGET)-unknown-elf-objcopy 
OBJDUMP		= $(PATH)/$(TARGET)-unknown-elf-objdump
CLANG		= clang-10
LLVM_AS		= llvm-as-10
QEMU		= qemu-riscv64

TEST		= testcases
TESTPROG	= $(TEST)/test.c
TESTELF		= $(TEST)/test

TEST_LL		= test.ll

CFLAGS      = -Og

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
	@$(CC) -S $(CFLAGS) $(TEST)/test.c -o test.S

gen_exe: test.S
	@$(CC) $(TESTPROG) $(CFLAGS) -o $(TESTELF)

qemu_test: test 
	$(QEMU) $(TESTELF)

clean:
	@cargo clean
	@rm test.ll
	@rm test.S