TARGET		= riscv64

CC			= $(TARGET)-unknown-elf-gcc
LD 			= $(TARGET)-unknown-elf-ld
AS 			= $(TARGET)-unknown-elf-as
OBJCOPY		= $(TARGET)-unknown-elf-objcopy 
OBJDUMP		= $(TARGET)-unknown-elf-objdump
CLANG		= clang-10
LLVM_AS		= llvm-as-10

TEST		= testcases

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

gen_ir:
	@$(CLANG) -S -emit-llvm $(TEST)/test.c

gen_bc:
	@(LLVM_AS) test.ll -o test.bc

gen_asm: $(TEST)/test.c
	@$(CC) -S -Og $(TEST)/test.c -o test.S

clean:
	@cargo clean
	@rm test.ll
	@rm test.S