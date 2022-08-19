TARGET		= riscv64
AS 			= $(TARGET)-unknown-elf-as
CLANG		= clang

TEST		= testcases

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

clean:
	@cargo clean
	@rm main.S
	@rm main 
	@rm test.ll