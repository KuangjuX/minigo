
AS = riscv64-unknown-elf-as

.PHONY: run build
build:
	@cargo build 

run: 
	@cargo run

exe:
	$(AS) -c main.S -o main