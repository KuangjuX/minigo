use std::process::Command;
use std::path::Path;
use std::vec;

pub fn assemble(input: &str, output: &str) {
    if let Some(assembler) = find_assembler() {
        Command::new(assembler)
            .arg("-c")
            .arg(input)
            .arg("-o")
            .arg(output)
            .spawn().unwrap();
    }else{
        println!("[Debug] Fail to find riscv64 assembler");
    }
}


pub fn run_linker(input: &str, output: &str) {
    match (find_ld(), find_libpath()) {
        (Some(ld), Some(lib_path)) => {
            Command::new(ld)
                .arg("-o")
                .arg(output) 
                .arg("-m")
                .arg("elf_riscv64") 
                .arg(format!("{}/crt1.o", lib_path))
                .arg(format!("{}/ctri.o", lib_path))
                .arg(format!("{}/crtbegin.o", lib_path))
                .spawn().unwrap();
        },
        _ => {

        }
    }
}

fn find_assembler() -> Option<String> {
    let paths = vec!["/home/kuangjux/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin/riscv64-unknown-elf-as"];
    for path in paths.iter() {
        if Path::new(path).exists() {
            let assembler = format!("{}", path);
            return Some(assembler)
        }
    }
    None
}

/// Find riscv64-unknown-elf-ld
fn find_ld() -> Option<String> {
    let paths = ["/home/kuangjux/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin/riscv64-unknown-elf-ld"];
    for path in paths.iter() {
        if Path::new(path).exists() {
            let ld = format!("{}", path);
            return Some(ld)
        }
    }
    None
}

/// Find riscv64 runtime library
fn find_libpath() -> Option<String> {
    if Path::new("/home/kuangjux/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/lib/gcc/riscv64-unknown-elf/8.3.0/ctri.o").exists() {
        let path = format!("{}", "/home/kuangjux/riscv/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/lib/gcc/riscv64-unknown-elf/8.3.0");
        return Some(path)
    }
    None
}