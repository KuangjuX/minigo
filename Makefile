RVDIR 		= /home/kuangjux/software/riscv/riscv-toolchains/bin
TARGET		= riscv64-unknown-elf-

CC			= $(RVDIR)/$(TARGET)gcc
LD 			= $(RVDIR)/$(TARGET)ld
AS 			= $(RVDIR)/$(TARGET)as
OBJCOPY		= $(RVDIR)/$(TARGET)objcopy 
OBJDUMP		= $(RVDIR)/$(TARGET)objdump
CFLAGS      = -Og

CLANG		= clang-14
LLVM_AS		= llvm-as-14
LLC 		= llc-14
QEMU		= qemu-riscv64

TEST		= testcases
PROG 		?= fib
TESTELF		= $(TEST)/$(PROG)
TESTASM     = $(TESTELF).S
TESTPROG	= $(TESTELF).c
TESTOUT     = $(TESTELF).o


ASMS  		= $(wildcard $(TEST)/*.S)
OBJS 		= $(wildcard $(TEST)/*.o)
ELFS 		= $(TEST)/test $(TEST)hello_world

TEST_LL		= test.ll

FRONT_SRC   ?= test/test.go
FRONT_LLIR	?= frontend/test/test.go.ll
FRONT_LL    ?= frontend/test/test.go


$(minigo): minigo

TEST_SRCS=$(wildcard testcases/*.c)
TESTS=$(TEST_SRCS:.c=.exe)
BC = $(TEST_SRCS:.c=.bc)


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

# .PHONY: run build
# build:
# 	@cargo build

# run: gen_bc
# 	@RUST_BACKTRACE=1 PROG=$(PROG) cargo run $(FEATURES) 

# exe:
# 	@$(AS) -c main.S -o main
# 	@$(QEMU) main

gen_ir: 
	@$(CLANG) -S -O0 -emit-llvm --target=riscv64-unknown-linux-gnu $(TESTPROG)

gen_bc:
	@$(LLVM_AS) $(PROG).ll -o $(PROG).bc

llc:
	@$(LLC) --march=riscv64 $(PROG).ll

# gen_asm: $(TESTPROG)
# 	@$(CC) -S $(CFLAGS) $(TESTPROG) -o $(TESTASM)

# display_compile:
# 	@$(CC) $(TESTPROG) $(CFLAGS) -v -o $(TESTELF)

# gen_exe: 
# 	@$(CC) $(TESTPROG) $(CFLAGS) -o test

# qemu_test: $(TESTELF) 
# 	@$(QEMU) $(TESTELF)

# debug:
# 	@tmux new-session -d \
# 		"$(QEMU) -s -S" && \
# 		tmux split-window -h "$(RVDIR)/riscv64-unknown-elf-gdb -ex 'file $(TESTELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
# 		tmux -2 attach-session -d

clean:
	@rm *.s *.ll *.bc *.S *.o *.exe minigo || true
	@make -C testcases

# Stage 1
minigo:
	@cargo build
	cp target/debug/minigo ./

testcases/%.ll: minigo testcases/%.c 
	$(CLANG) -S -O0 -emit-llvm --target=riscv64-unknown-linux-gnu testcases/$*.c -o testcases/$*.ll
	
testcases/%.bc: testcases/%.ll 
	$(LLVM_AS) testcases/$*.ll -o testcases/$*.bc 

# bc: $(BC)

testcases/%.exe: testcases/%.bc
	./minigo testcases/$*

test: $(TESTS)
	testcases/driver.sh ./minigo

$(FRONT_LLIR):
	cd frontend && pytyon3 main.py $(FRONT_SRC)

run: $(FRONT_LLIR) minigo
	make gen_bc PROG=$(FRONT_LL)
	./minigo $(FRONT_LL)



