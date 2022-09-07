use super::{ Program, Op, Function };
use llvm_ir::instruction::Xor;
use crate::{utils::parse_operand, ir::{StackVar, VirtualReg}};

impl Program {
    pub(crate) fn handle_xor(&self, func: &Function, inst: &Xor) {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        match (parse_operand(op0), parse_operand(op1)) {
            (Some(Op::ConstValue(op1)), Some(Op::ConstValue(op2))) => {

            },
            (Some(Op::LocalValue(op1)), Some(Op::ConstValue(op2))) => {
                // op is local variable, op2 is immediate
                if let Some(local_var) = func.find_local_var(op1) {
                    match &local_var {
                        VirtualReg::Stack(stack_var) => {
                            
                        },
                        VirtualReg::Reg(reg_var) => {

                        }
                    }
                }
            },
            (Some(Op::ConstValue(op1)), Some(Op::LocalValue(op2))) => {

            },
            (Some(Op::LocalValue(op1)), Some(Op::LocalValue(op2))) => {
                
            }
            _ => {}
        }
    }
}