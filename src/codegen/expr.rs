use super::{ Program, Op, Function, Error, ConstValue, ProgInner };
use llvm_ir::instruction::{Xor, Load};
use llvm_ir::terminator::Ret;
use crate::{utils::parse_operand, ir::VirtualReg};

impl Program {
    /// Handle xori instruction
    /// xori rd, rs1, imm12
    /// Note, XORI rd, rs1, -1 rs1 (assembler pseudoinstruction NOT rd, rs ).
    pub(crate) fn handle_xor(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Xor) -> Result<(), Error> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        match (parse_operand(op0), parse_operand(op1)) {
            (Some(Op::LocalValue(op1)), Some(Op::ConstValue(op2))) => {
                // op is local variable, op2 is immediate
                let ConstValue::Num(imm, _) = op2;
                let local_var = func.find_local_var(op1).ok_or(Error::new("Fail to find local variable"))?;
                let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to load reg var"))?;
                match &local_var {
                    VirtualReg::Stack(rs1_stack_var) => {
                        // stack_var
                        let rs1_reg_var = rs1_stack_var.load_stack_var(self, prog_inner).ok_or(Error::new("Fail to load stack var"))?;
                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                        self.write_asm(asm);
                    },
                    VirtualReg::Reg(rs1_reg_var) => {
                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                        self.write_asm(asm);
                    }
                }
                Ok(())
            },
            _ => { Err(Error::new("Invalid xor instruction")) }
        }
    }

    pub(crate) fn handle_load(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Load) -> Result<(), Error> {
        let address = &inst.address;
        let dest = &inst.dest;
        let dest_reg_var = VirtualReg::allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        match parse_operand(address) {
            Some(Op::LocalValue(op)) => {
                let local_virt_reg = func.find_local_var(op).ok_or(Error::new("Fail to find local var"))?;
                match local_virt_reg {
                    VirtualReg::Stack(stack_var) => {
                        let offset = stack_var.addr;
                        let asm = format!("    ld {}, -{}(fp)", dest_reg_var.name, offset);
                        self.write_asm(asm);
                    },
                    _ => { return Err(Error::new("Fail to find stack var")) }
                }
            },
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn handle_ret(&self, func: &Function, inst: &Ret) -> Result<(), Error> {
        // return 
        if let Some(op) = &inst.return_operand {
            match parse_operand(op) {
                Some(Op::ConstValue(constval)) => {
                    match constval {
                        ConstValue::Num(num, _) => {
                            let asm = format!("    li a0, {}", num);
                            self.write_asm(asm);
                        }
                    }
                }
                Some(Op::LocalValue(name)) => {
                    match func.find_local_var(name) {
                        Some(VirtualReg::Reg(reg_var)) => {
                            let asm = format!("    mv a0, {}", reg_var.name);
                            self.write_asm(asm);
                        },
                        Some(VirtualReg::Stack(stack_var)) => {
                            todo!()
                        },
                        _ => { return Err(Error::new("Fail to find local var")) }
                    }
                }
                _ => { return Err(Error::new("Fail to parse operand")) }
            }
        }
        self.write_asm("# function return");
        self.write_asm("    mv sp, fp");
        self.write_asm("    ld fp, 0(sp)");
        self.write_asm("    ld ra, 8(sp)");
        self.write_asm("    addi sp, sp, 16");
        self.write_asm("    ret\n\n");
        Ok(())
    }
}