use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::cell::RefCell;
use bit_field::BitField;

// use crate::arch::Instruction;
use super::{Function, VarValue, Ty};
use super::{CodeGen, Var};
use crate::utils::align_to;



pub struct Program {
    pub(crate) asm_file: RefCell<File>,
    // pub(crate) module: Module,
    pub(crate) funcs: VecDeque<Function>,
    pub(crate) vars: VecDeque<Var>,
}

impl Program {
    pub fn new(asm: File) -> Self {
        
        Self{
            /// Output assemble file
            asm_file: RefCell::new(asm),
            /// All function in ir
            funcs: VecDeque::new(),
            /// All global variable in ir
            vars: VecDeque::new()
        }
    }

    pub fn debug(&self) {
        for var in self.vars.iter() {
            println!("var: {:?}", var);
        }
    }

    fn write_asm<S>(&self, asm: S) where S: Into<String> {
        let asm = format!("{}\n", asm.into());
        let mut asm_file = self.asm_file.borrow_mut();
        asm_file.write(asm.as_bytes()).unwrap();
    }

    fn assign_var_offset(&mut self) {
        for func in self.funcs.iter_mut() {
        }
    }

}

impl CodeGen for Program {
    /// generation text section
    fn emit_text(&mut self) {        
        // generate section
        for func in self.funcs.iter() {
            if func.is_static {
                self.write_asm("    .local");
            }else{
                self.write_asm("    .globl");
            }
            self.write_asm("    .text");
            let name = format!("{}:\n", func.name);
            self.write_asm(name);

            // push all arguments into stack
            /*
            * Stack:
            * ----------------------- // sp
            *        ra             
            * ---------------------- // ra = sp - 8
            *        fp
            * ---------------------- // fp = sp - 16
            *       vars           
            * ---------------------- // sp = sp - 16 - stacksize
            *      exprs
            * ----------------------
            */
            self.write_asm("    # Store ra register");
            // sp = sp - 16
            self.write_asm("    addi sp, sp, -16");
            self.write_asm("    sd ra, 8(sp)");

            // store fp register
            self.write_asm("    # Store fp register");
            self.write_asm("    sd fp, 0(sp)");

            // sp = sp - stack_size
            self.write_asm("    # Store params");
            let asm = format!("    addi sp, sp, -{}", func.stack_size);
            self.write_asm(asm);

            // Store all params
            let mut offset = 0;
            for (index ,param) in func.params.iter().enumerate() {
                match param.ty {
                    Ty::I32 => {
                        let asm = format!("    sw a{}, ({})sp", index, offset);
                        self.write_asm(asm);
                        offset += 4;
                    },
                    Ty::I64 => {
                        let asm = format!("    sd a{}, ({})sp", index, offset);
                        self.write_asm(asm);
                        offset += 8;
                    },
                    _ => {}
                }
            }
        }
        
    }

    /// generate data section
    fn emit_data(&mut self) {
        for var in self.vars.iter() {
            if var.is_static {
                let line = format!("    .local {}", var.name);
                self.write_asm(line);
            }else{
                let line = format!("    .globl {}", var.name);
                self.write_asm(line);
            }

            // .data or .tdata
            if var.initiazed {
                if var.is_tls {
                    self.write_asm("    .section .tdata,\"awT\",@progbits");
                }else{
                    self.write_asm("    .data");
                }
                if let Some(init) = &var.init_data {
                    match init {
                        VarValue::Int(val) => {
                            match &var.ty {
                                &super::Ty::I32 => {
                                    let write_val = format!("    .word  {}", *val as i32);
                                    self.write_asm(write_val);
                                },

                                &super::Ty::I64 => {
                                    let write_val = format!("    .dword  {}", *val as i64);
                                    self.write_asm(write_val);
                                },

                                _ => {}
                            }

                        }

                        VarValue::Pointer(name) => {
                            if var.ty == Ty::Pointer {
                                let write_val = format!("    .dword  {}", name);
                                self.write_asm(write_val);
                            }
                        },

                        VarValue::Array{bits, elements} => {
                            let mut pos = 0;
                            let mut x = 0;
                            let mut i = 0;
                            let size = bits / 8;
                            while i < var.size {
                                if x == size {
                                    pos += 1;
                                    x = 0;
                                }
                                let element = elements[pos];
                                let low = x * 8;
                                let high = (x + 1) * 8;
                                let byte = element.get_bits(low..high);
                                let info = format!("    .byte {}", byte);
                                self.write_asm(info);
                                x += 1;
                                i += 1;
                            }
                        }
                    }
                }
                let ty = format!("    .type {}, @object", var.name);
                self.write_asm(ty);
                let size = format!("    .size {}, {}", var.name, var.size);
                self.write_asm(size);
                let align = format!("    .align {}", var.align);
                self.write_asm(align);
                let name = format!("{}:", var.name);
                self.write_asm(name);
            }else {
                // .bss or .tbss
                if var.is_tls {
                    self.write_asm("    .section .tbss,\"awT\",@nobit");
                }else{
                    self.write_asm("    .bss");
                }
                let align = format!("    .align {}", var.align);
                self.write_asm(align);
                let name = format!("{}:", var.name);
                self.write_asm(name);
                let zero = format!("    .zero {}", var.ty.size());
                self.write_asm(zero);
            }

        }
    }

    fn codegen(&mut self) {
        self.emit_text();
        self.emit_data();
    }
}