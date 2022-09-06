use std::collections::VecDeque;
use std::fs::File;
use std::io::Write as Write2;
use std::cell::RefCell;
use std::fmt::{Write, self};
use std::collections::HashMap;
use bit_field::BitField;
use llvm_ir::{ Instruction, operand::Operand, constant::Constant, terminator::Terminator };
use super::ConstValue;

use crate::ir::{StackVar, VirtualReg};
use crate::utils::{parse_type, align_to, parse_operand};
use crate::warning;

// use crate::arch::Instruction;
use super::{Function, VarValue, Ty, Op};
use super::{CodeGen, Var};

impl Write for Program {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_asm(s);
        Ok(())
    }
}



pub struct Program {
    pub(crate) asm_file: RefCell<File>,
    // pub(crate) module: Module,
    pub(crate) inner: RefCell<ProgInner>
}

pub struct ProgInner {
    pub(crate) funcs: VecDeque<Function>,
    pub(crate) vars: VecDeque<Var>,
    pub(crate) stack_depth: usize
}

impl Program {
    pub fn new(asm: File) -> Self { 
        Self{
            /// Output assemble file
            asm_file: RefCell::new(asm),
            inner: RefCell::new(
                ProgInner {
                    /// All function in ir
                    funcs: VecDeque::new(),
                    /// All global variable in ir
                    vars: VecDeque::new(),
                    stack_depth: 0
                }
            )
        }
    }

    pub(crate) fn write_asm<S>(&self, asm: S) where S: Into<String> {
        let asm = format!("{}\n", asm.into());
        let mut asm_file = self.asm_file.borrow_mut();
        asm_file.write(asm.as_bytes()).unwrap();
    }


    fn gen_expr(&self, func: &mut Function) {
        for block in func.blocks.iter() {
            for inst in block.instrs.iter() {
                match inst {
                    Instruction::Alloca(alloca) => {
                        let mut offset = 0;
                        match &alloca.num_elements {
                            Operand::ConstantOperand(constref) => {
                                let constval = &**constref;
                                match constval {
                                    &Constant::Int{ bits, value} => {
                                        let mut size = (bits as usize / 8) * value as usize;
                                        size = align_to(size, alloca.alignment as usize);
                                        offset = size;
                                        func.stack_size += size;
                                        let asm = format!("    addi sp, sp, -{}", size);
                                        self.write_asm(asm);
                                    },
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                        
                        if let Ok((ty, size)) = parse_type(&alloca.allocated_type) {
                            let reg = &alloca.dest;
                            if func.locals.iter().position(|local| local.name == Some(reg.clone())).is_none() {
                                let mut local_var = Var::uninit();
                                // Set local variable type
                                local_var.ty = ty;
                                // Set local variable size
                                local_var.size = size;
                                // Set local variable name
                                local_var.name = Some(reg.clone());
                                // Set stack variable(address, size)
                                let stack_var = StackVar::new(func.stack_size - offset, size);
                                local_var.local_val = Some(VirtualReg::Stack(stack_var));
                                func.locals.push(local_var);
                            } 
                        }else{
                            warning!("Fail to parse type: {:?}", &alloca.allocated_type);
                        }
                    },

                    Instruction::Store(store) => {
                        let address = &store.address;
                        let value = &store.value;
                        if let (Some(address), Some(value)) = (parse_operand(address), parse_operand(value)) {
                            match (address, value) {
                                (Op::LocalValue(name), Op::ConstValue(constval)) => {
                                    for local in func.locals.iter() {
                                        if local.name == Some(name.clone()) {
                                            match &local.local_val {
                                                Some(VirtualReg::Stack(stack_var)) => {
                                                    let addr = stack_var.addr;
                                                    match constval {
                                                        ConstValue::Num(val) => {
                                                            let asm = format!("    addi zero, zero, {}", val);
                                                            self.write_asm(asm);
                                                            let asm = format!("    sd zero, -{}(fp)", addr);
                                                            self.write_asm(asm)
                                                        }
                                                    }
                                                }
                                                Some(VirtualReg::Reg(reg)) => {

                                                },
                                                None => {}
                                            }
                                        }
                                    }
                                }
                                _ =>{}
                            }
                        }
                    }
        
                    _ => {}
                }
            }
            let termianl = &block.term;
            match termianl {
                Terminator::Ret(ret) => {
                    // return 
                    self.write_asm("# function return");
                    self.write_asm("    mv sp, fp");
                    self.write_asm("    ld fp, 0(sp)");
                    self.write_asm("    ld ra, 8(sp)");
                    self.write_asm("    addi sp, sp, 16");
                    if let Some(op) = &ret.return_operand {
                        if let Some(op) = parse_operand(op) {
                            match op {
                                Op::ConstValue(constval) => {
                                    match constval {
                                        ConstValue::Num(num) => {
                                            let asm = format!("    li a0, {}", num);
                                            self.write_asm(asm);
                                        }
                                    }
                                }
                                Op::LocalValue(name) => {
                                    // let asm = format!("mv a0, {}", )
                                }
                            }
                        }
                    }
                    self.write_asm("    ret\n\n");
                }
                _ => {}
            }
        }

        
    }

}

impl CodeGen for Program {
    /// generation text section
    fn emit_text(&mut self) {        
        // generate section
        let mut inner = self.inner.borrow_mut();
        for func in inner.funcs.iter_mut() {
            if func.is_static {
                let asm = format!("    .local {}", func.name);
                self.write_asm(asm);
            }else{
                let asm = format!("    .globl {}", func.name);
                self.write_asm(asm);
            }
            self.write_asm("    .text");
            let name = format!("{}:", func.name);
            self.write_asm(name);

            // push all arguments into stack
            /*
            * Stack:
            * ----------------------- // sp
            *        ra             
            * ---------------------- // ra = sp - 8
            *        fp
            * ---------------------- // fp = sp - 16
            *       params           
            * ---------------------- // sp = sp - 16 - params size
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

            // write fp to sp
            self.write_asm("    # write sp to fp");
            self.write_asm("    mv fp, sp");

            // sp = sp - stack_size
            self.write_asm("    # Store params");
            let asm = format!("    addi sp, sp, -{}", func.stack_size);
            self.write_asm(asm);

            // Store all params
            let mut offset = 0;
            for (index ,param) in func.params.iter().enumerate() {
                match param.ty {
                    Ty::Num => {
                        let asm = format!("    sd a{}, {}(sp)", index, offset);
                        self.write_asm(asm);
                        offset += 8;
                    },
                    _ => {}
                }
            }

            self.gen_expr(func);

        }
        
    }

    /// generate data section
    fn emit_data(&mut self) {
        let inner = self.inner.borrow();
        for var in inner.vars.iter() {
            if var.is_static {
                if let Some(name) = var.name.clone() {
                    let asm = format!("    .local {}", name);
                    self.write_asm(asm);
                }
            }else{
                if let Some(name) = var.name.clone() {
                    let asm = format!("    .globl {}", name);
                    self.write_asm(asm);
                }
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
                        &VarValue::Num(val, size) => {
                            // match &var.ty {
                            //     &super::Ty::Num => {
                            //         let write_val = format!("    .word  {}", *val as i64);
                            //         self.write_asm(write_val);
                            //     },

                            //     _ => {}
                            // }
                            for i in 0..size {
                                let low = i * 8;
                                let high = (i + 1) * 8;
                                let byte = val.get_bits(low..high);
                                let info = format!("    .byte {}", byte);
                                self.write_asm(info);
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
                if let Some(name) = var.name.clone() {
                    let asm = format!("    .type {}, @object", name);
                    self.write_asm(asm);
                    let asm = format!("    .size {}, {}", name, var.size);
                    self.write_asm(asm);
                    let asm = format!("    .align {}", var.align);
                    self.write_asm(asm);
                    let asm = format!("{}:", name);
                    self.write_asm(asm);
                }
            }else {
                // .bss or .tbss
                if var.is_tls {
                    self.write_asm("    .section .tbss,\"awT\",@nobit");
                }else{
                    self.write_asm("    .bss");
                }
                let asm = format!("    .align {}", var.align);
                self.write_asm(asm);
                if let Some(name) = var.name.clone() {
                    let asm = format!("{}:", name);
                    self.write_asm(asm);
                }
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