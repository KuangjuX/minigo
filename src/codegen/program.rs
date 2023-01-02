use std::collections::VecDeque;
use std::fs::File;
use std::io::Write as Write2;
use std::cell::{ RefCell, UnsafeCell };
use std::fmt::{Write, self};
use bit_field::BitField;
use llvm_ir::{ Instruction, terminator::Terminator };
use super::error::Error;
use super::func::Label;
use super::{PhysicalRegs};


// use crate::arch::Instruction;
use super::{Function, VarValue, Ty};
use super::{CodeGen, Var};

impl Write for Program {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_asm(s);
        Ok(())
    }
}



pub struct Program {
    pub(crate) asm_file: RefCell<File>,
    pub(crate) inner: UnsafeCell<ProgInner>
}

pub struct ProgInner {
    pub(crate) funcs: VecDeque<Function>,
    pub(crate) vars: VecDeque<Var>,
    pub(crate) regs: PhysicalRegs
}

impl Program {
    pub fn new(asm: File) -> Self { 
        Self{
            /// Output assemble file
            asm_file: RefCell::new(asm),
            inner: UnsafeCell::new(
                    ProgInner {
                        /// All function in ir
                        funcs: VecDeque::new(),
                        /// All global variable in ir
                        vars: VecDeque::new(),
                        regs: PhysicalRegs::init()
                    })
        }
    }

    pub(crate) fn write_asm<S>(&self, asm: S) where S: Into<String> {
        let asm = format!("{}\n", asm.into());
        let mut asm_file = self.asm_file.borrow_mut();
        asm_file.write(asm.as_bytes()).unwrap();
    }


    fn gen_label(&self, func: &Function){
        let mut inner = func.inner.borrow_mut();
        for index in 0..func.blocks.len() {
            let label = Label {
                llvm_name: func.blocks[index].name.clone(),
                label_name: format!(".label{}", index)
            };
            inner.labels.push(label);
        }
    }

    fn gen_expr(&self, inner: &mut ProgInner, func: &Function) -> Result<(), Error> {
        let mut index = 0;
        for block in func.blocks.iter() {
            if index != 0 {
                let asm = format!(".label{}:", index);
                self.write_asm(asm);
            }
            for inst in block.instrs.iter() {
                match inst {
                    Instruction::Alloca(alloca) => { self.handle_alloca(inner, func, &alloca)? },
                    Instruction::Store(store) => { self.handle_store(inner, func, &store)? },
                    Instruction::Xor(xor) => { self.handle_xor(inner, func, &xor)? }
                    Instruction::Load(load) => { self.handle_load(inner, func, &load)? }
                    Instruction::Add(add) => { 
                        let op0 = add.operand0.clone();
                        let op1 = add.operand1.clone();
                        let dest = add.dest.clone();
                        let f = move |op0, op1, dest, inst_type| {
                            match inst_type {
                                super::InstType::I => {
                                    format!("\taddi {}, {}, {}", dest, op0, op1)
                                },
                                super::InstType::R => {
                                    format!("\tadd {}, {}, {}", dest, op0, op1)
                                }
                            }
                        };
                        self.handle_add_sub_mul_div_mod(inner, func, op0, op1, dest, f)?;
                    }
                    Instruction::Sub(sub) => { 
                        let op0 = sub.operand0.clone();
                        let op1 = sub.operand1.clone();
                        let dest = sub.dest.clone();
                        let f = move |op0, op1, dest, inst_type| {
                            match inst_type {
                                super::InstType::I => {
                                    format!("\taddi {}, {}, -{}", dest, op0, op1)
                                },
                                super::InstType::R => {
                                    format!("\tsub {}, {}, {}", dest, op0, op1)
                                }
                            }
                        };
                        self.handle_add_sub_mul_div_mod(inner, func, op0, op1, dest, f)?;
                    }
                    Instruction::Mul(mul) => { 
                        let op0 = mul.operand0.clone();
                        let op1 = mul.operand1.clone();
                        let dest = mul.dest.clone();
                        let f = move |op0, op1, dest, inst_type| {
                            match inst_type {
                                super::InstType::I => {
                                    format!("\tmul {}, {}, {}", dest, op0, op1)
                                },
                                super::InstType::R => {
                                    format!("\tmul {}, {}, {}", dest, op0, op1)
                                }
                            }
                        };
                        self.handle_add_sub_mul_div_mod(inner, func, op0, op1, dest, f)?;
                    }
                    Instruction::SDiv(sdiv) => { 
                        let op0 = sdiv.operand0.clone();
                        let op1 = sdiv.operand1.clone();
                        let dest = sdiv.dest.clone();
                        let f = move |op0, op1, dest, inst_type| {
                            match inst_type {
                                super::InstType::I => {
                                    format!("\tdiv {}, {}, {}", dest, op0, op1)
                                },
                                super::InstType::R => {
                                    format!("\tdiv {}, {}, {}", dest, op0, op1)
                                }
                            }
                        };
                        self.handle_add_sub_mul_div_mod(inner, func, op0, op1, dest, f)?;
                    }
                    Instruction::ICmp(icmp) => { self.handle_icmp(inner, func, &icmp)? }
                    Instruction::ZExt(zext) => { self.handle_zext(inner, func, &zext)? }
                    Instruction::Call(call) => { self.handle_call(inner, func, &call)? }
                    _ => {}
                }
            }
            let termianl = &block.term;
            match termianl {
                Terminator::Ret(ret) => { self.handle_ret(inner, func, &ret)?; }
                Terminator::Br(br) => { self.handle_br(func, &br)?; }
                Terminator::CondBr(condbr) => { self.handle_condbr(inner, func, &condbr)?; }
                _ => {}
            }
            self.write_asm("\n\n");
            index += 1;
        }

        Ok(())
    }

    // 计算函数中变量所需要的栈空间
    // fn assign_lvar_offsets(&self, inner: &mut ProgInner, func: &Function) {
    //     for block in func.blocks {

    //     }
    // }

}

impl CodeGen for Program {
    /// generation text section
    fn emit_text(&mut self) {      
        // generate section
        let inner = unsafe{ &mut *self.inner.get() };
        for func in inner.funcs.iter() {
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
            *       exprs
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
            let asm = format!("    addi sp, sp, -{}", func.stack_size());
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

            self.write_asm("    # generate expr");
            let inner = unsafe{ &mut *self.inner.get() };
            self.gen_label(func);
            if let Err(err) = self.gen_expr(inner, func) {
                match err {
                    _ => {
                        panic!("{:?}", err);
                    }
                }
            }
            let inner = unsafe{ &mut *self.inner.get() };
            inner.free_all_physical_regs();
        }
        
    }

    /// generate data section
    fn emit_data(&mut self) {
        let inner = unsafe{ &mut *self.inner.get() };
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