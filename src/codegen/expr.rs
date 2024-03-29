use std::fmt::{Debug, Display};

use super::{ Program, Op, Function, Result, Error, ConstValue, ProgInner, InstType};
use llvm_ir::{Name, IntPredicate, Operand};
use llvm_ir::instruction::{Xor, Load, Store, Alloca,  ICmp, ZExt, Call};
use llvm_ir::terminator::{Ret, Br, CondBr};
use crate::utils::{ parse_operand, parse_operand_2};
use crate::ir::{VirtualReg,  RegVar};

impl Program {
    /// 处理谓词逻辑
    fn handle_predicate<S: Display, U: Display, V: Display>(&self, predicate: &IntPredicate, prog_inner: &mut ProgInner, op0: S, op1: U, dest: V, inst_type: InstType) {
        match predicate {
            IntPredicate::EQ => {
                // 等于
                let help_reg = prog_inner.get_help_reg_1();
                match inst_type {
                    InstType::I => {
                        let asm = format!("\txori {}, {}, {}", help_reg.name, op0, op1);
                        self.write_asm(asm);
                    }
                    InstType::R => {
                        let asm = format!("\txor {}, {}, {}", help_reg.name, op0, op1);
                        self.write_asm(asm);
                    }
                }
                let asm = format!("\tseqz {}, {}", dest, help_reg.name);
                self.write_asm(asm);
            },
            IntPredicate::NE => {
                // 不等于
                let help_reg = prog_inner.get_help_reg_1();
                match inst_type {
                    InstType::I => {
                        let asm = format!("\txori {}, {}, {}", help_reg.name, op0, op1);
                        self.write_asm(asm);
                    }
                    InstType::R => {
                        let asm = format!("\txor {}, {}, {}", help_reg.name, op0, op1);
                        self.write_asm(asm);
                    }
                }
                let asm = format!("\tsnez {}, {}", dest, help_reg.name);
                self.write_asm(asm);
            },
            IntPredicate::SGT => {
                // 大于
                match inst_type {
                    InstType::I => {
                        let asm = format!("\tslti {}, {}, {}", dest, op0, op1);
                        self.write_asm(asm);
                        let asm = format!("\tnot {}, {}", dest, dest);
                        self.write_asm(asm);
                    }
                    InstType::R => {
                        let asm = format!("\tslt {}, {}, {}", dest, op0, op1);
                        self.write_asm(asm);
                        let asm = format!("\tnot {}, {}", dest, dest);
                        self.write_asm(asm);
                    }
                }
            },
            IntPredicate::SGE => {
                // 大于等于
                // 由于前两个帮助寄存器可能用到，所以申请第三个帮助寄存器
                let help_reg_3 = prog_inner.get_help_reg_3();
                match inst_type {
                    InstType::I => {
                        let asm = format!("\tslti {}, {}, {}", dest, op0, op1);
                        self.write_asm(asm);
                        let asm = format!("\tnot {}, {}", dest, dest);
                        self.write_asm(asm);
                        let asm = format!("\txori {}, {}, {}", help_reg_3.name, op0, op1);
                        self.write_asm(asm);
                    }
                    InstType::R => {
                        let asm = format!("\tslt {}, {}, {}", dest, op0, op1);
                        self.write_asm(asm);
                        let asm = format!("\tnot {}, {}", dest, dest);
                        self.write_asm(asm);
                        let asm = format!("\txor {}, {}, {}", help_reg_3.name, op0, op1);
                        self.write_asm(asm);
                    }
                }
                let asm = format!("\tseqz {}, {}", help_reg_3.name, help_reg_3.name);
                self.write_asm(asm);
                let asm = format!("\tor {}, {}, {}", dest, help_reg_3.name, dest);
                self.write_asm(asm);
            },
            IntPredicate::SLT => {
                match inst_type {
                    InstType::I => {
                        let asm = format!("\tslti {}, {}, {}",dest, op0, op1);
                        self.write_asm(asm);
                    }
                    InstType::R => {
                        let asm = format!("\tslt {}, {}, {}",dest, op0, op1);
                        self.write_asm(asm);
                    }
                }
            },
            IntPredicate::SLE => {
                // 小于等于
                let help_reg_3 = prog_inner.get_help_reg_3();
                match inst_type {
                    InstType::I => {
                        let asm = format!("\tslti {}, {}, {}", dest, op0, op1);
                        self.write_asm(asm);
                        let asm = format!("\txori {}, {}, {}", help_reg_3.name, op0, op1);
                        self.write_asm(asm);
                    }
                    InstType::R => {
                        let asm = format!("\tslt {}, {}, {}", dest, op0, op1);
                        self.write_asm(asm);
                        let asm = format!("\txor {}, {}, {}", help_reg_3.name, op0, op1);
                        self.write_asm(asm);
                    }
                }
                let asm = format!("\tseqz {}, {}", help_reg_3.name, help_reg_3.name);
                self.write_asm(asm);
                let asm = format!("\tor {}, {}, {}", dest, help_reg_3.name, dest);
                self.write_asm(asm);
            }
            _ => { todo!() }
        }
    }

    /// icmp 的核心逻辑，可以同时处理栈变量和寄存器变量
    fn handle_common_icmp(&self, prog_inner: &mut ProgInner, func: &Function, predicate: &IntPredicate, op0: & Operand, op1: &Operand, dest_reg_var: RegVar) -> Result<()> {
        match parse_operand_2(op0, op1) {
            Some((ans1, ans2)) => {
                match (ans1, ans2) {
                    (Op::LocalValue(var1), Op::LocalValue(var2)) => {
                        let var1 = func.find_local_var(var1).ok_or(Error::new("Fail to find var"))?;
                        let var2 = func.find_local_var(var2).ok_or(Error::new("Fail to find var"))?;
                        match (var1, var2) {
                            (VirtualReg::Stack(stack_var_1), VirtualReg::Stack(stack_var_2)) => {
                                // 为 stack1 和 stack2 分配两个帮助寄存器
                                let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                let help_reg_2 = stack_var_2.load_stack_var_2(self, prog_inner);
                                self.handle_predicate(predicate, prog_inner, help_reg_1.name, help_reg_2.name, dest_reg_var.name, InstType::R);
                            },
                            (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                self.handle_predicate(predicate, prog_inner, reg1.name, reg2.name, dest_reg_var.name, InstType::R);
                            },
                            _ => { todo!() }
                        }
    
                    },
                    (Op::LocalValue(var), Op::ConstValue(val)) => {
                        let var = func.find_local_var(var).ok_or(Error::new("Fail to find var"))?;
                        match val {
                            ConstValue::Num(num, _) => {
                                match var {
                                    VirtualReg::Reg(reg) => {
                                        self.handle_predicate(predicate, prog_inner, reg.name, num, dest_reg_var.name, InstType::I);
                                    },
                                    VirtualReg::Stack(stack) => {
                                        let help_reg_1 = stack.load_stack_var_1(self, prog_inner);
                                        self.handle_predicate(predicate, prog_inner,help_reg_1.name, num, dest_reg_var.name, InstType::I);
                                    }
                                }
                            },
                            _ => { todo!() }
                        }
                    }
                    _ => {}
                }
            },
            None => return Err(Error::new("Fail to parse operand"))
        }
        Ok(())
    }

    /// 处理加减乘除模的核心逻辑，用于简化
    fn handle_common_add_sub_mul_div_mod<F> (
        &self, prog_inner: &mut ProgInner, func: &Function,
        op0: Operand, op1: Operand, dest_reg_var: RegVar, f: F
    ) -> Result<()>
    where F: FnOnce(String, String, String, InstType) -> String 
    {
        match parse_operand_2(&op0, &op1) {
            Some((ans1, ans2)) => {
                match (ans1, ans2) {
                    (Op::LocalValue(loc1), Op::LocalValue(loc2)) => {
                        let var1 = func.find_local_var(loc1).ok_or(Error::new("Fail to find var"))?;
                        let var2 = func.find_local_var(loc2).ok_or(Error::new("Fail to find var"))?;
                        match (var1, var2) {
                            (VirtualReg::Stack(stack_var_1), VirtualReg::Stack(stack_var_2)) => {
                                // 分配两个帮助寄存器并将两个栈加载到帮助寄存器中
                                let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                let help_reg_2 = stack_var_2.load_stack_var_2(self, prog_inner);
                                // 计算结果并写回目标寄存器
                                let asm = f(help_reg_1.name, help_reg_2.name, dest_reg_var.name, InstType::R);
                                self.write_asm(asm);
                            },
                            (VirtualReg::Reg(reg1), VirtualReg::Reg(reg2)) => {
                                let asm = f(reg1.name, reg2.name, dest_reg_var.name, InstType::R);
                                self.write_asm(asm)
                            },
                            (VirtualReg::Reg(reg_var_1), VirtualReg::Stack(stack_var_2)) => {
                                let help_reg_1 = stack_var_2.load_stack_var_1(self, prog_inner);
                                let asm = f(reg_var_1.name.clone(), help_reg_1.name.clone(), dest_reg_var.name.clone(), InstType::R);
                                self.write_asm(asm);
                            },
                            (VirtualReg::Stack(stack_var_1), VirtualReg::Reg(reg_var_2)) => {
                                let help_reg_1 = stack_var_1.load_stack_var_1(self, prog_inner);
                                let asm = f(help_reg_1.name.clone(), reg_var_2.name.clone(), dest_reg_var.name.clone(), InstType::R);
                                self.write_asm(asm);
                            }
                        }

                    },
                    (Op::LocalValue(var), Op::ConstValue(val)) => {
                        let var = func.find_local_var(var).ok_or(Error::new("Fail to find var"))?;
                        if let ConstValue::Num(num, _) = val {
                            match var {
                                VirtualReg::Reg(reg) => {
                                    let asm = f(reg.name, format!("{}", num), dest_reg_var.name, InstType::I);
                                    self.write_asm(asm);
                                },
                                VirtualReg::Stack(stack_var) => {
                                    let help_reg = stack_var.load_stack_var_1(self, prog_inner);
                                    let asm = f(help_reg.name, format!("{}", num), dest_reg_var.name, InstType::I);
                                    self.write_asm(asm);
                                }
                            }
                        }else{
                            todo!()
                        }
                        
                    }   
                    _ => {}
                }
            },
            None => return Err(Error::new("Fail to parse operand"))
        }
        Ok(())
    }

    /// 函数调用时对参数的处理
    fn store_params(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Call) -> Result<()> {
        let mut index = 0;
        let args_num = inst.arguments.len();
        if args_num <= 7 {
            // 参数个数小于 7 时，将参数存在寄存器中
            for (arg, _) in inst.arguments.iter() {
                if let Some(op) = parse_operand(arg) {
                    match op {
                        Op::LocalValue(name) => {
                            let var = func.find_local_var(name).ok_or(Error::new("Failed to find local var"))?;
                            match var {
                                VirtualReg::Stack(stack_var) => {
                                    // 当参数为栈变量时, 分配帮助寄存器
                                    let help_reg = stack_var.load_stack_var_1(self, prog_inner);
                                    let asm = format!("\t mv a{}, {}", index, help_reg.name);
                                    self.write_asm(asm);
                                },
                                VirtualReg::Reg(reg_var) => {
                                    let asm = format!("\tmv a{}, {}", index, reg_var.name);
                                    self.write_asm(asm);
                                }
                            }
                        },
                        Op::ConstValue(val) => {
                            match val {
                                ConstValue::Num(num,  _) => {
                                    let asm = format!("\taddi a{}, zero, {}", index, num);
                                    self.write_asm(asm);
                                },
                                _ => { todo!() }
                            }
                            
                        }
                    }
                }
                index += 1;
            }
        }else{
            // 将所有参数全部放在栈里
            // 扩展栈空间
            let size = 8 * args_num as isize;
            let asm = format!("addi sp, sp, -{}", size);
            self.write_asm(asm);
            // func.add_stack_size(size);
            for (arg, _) in inst.arguments.iter() {
                if let Some(op) = parse_operand(arg) {
                    match op {
                        Op::LocalValue(name) => {
                            let var = func.find_local_var(name).ok_or(Error::new("Failed to find local var"))?;
                            match var {
                                VirtualReg::Stack(stack_var) => {
                                    // 当参数为栈变量时, 分配帮助寄存器
                                    let help_reg = stack_var.load_stack_var_1(self, prog_inner);
                                    let asm = format!("\tsd {}, {}(sp)", help_reg.name, index);
                                    self.write_asm(asm);
                                },
                                VirtualReg::Reg(reg_var) => {
                                    let asm = format!("\tsd {}, {}(sp)", reg_var.name, index);
                                    self.write_asm(asm);
                                }
                            }
                        },
                        Op::ConstValue(val) => {
                            // 目前只考虑参数是数字的情况
                            match val {
                                ConstValue::Num(num,_) => {
                                    let asm = format!("sd {}, {}(sp)", num, index * 8);
                                    self.write_asm(asm);
                                },
                                _ => { todo!() }
                            }
                            
                        }
                    }
                }
                index += 8;
            }
        }
        Ok(())
    }

    /// 如果参数大于 7 个，恢复栈
    fn restore_params(&self, inst: &Call) {
        let args_num = inst.arguments.len();
        if args_num > 7 {
            let size = args_num * 7;
            let asm = format!("\taddi sp, sp, {}", size);
            self.write_asm(asm);
        }
    }

    pub(crate) fn handle_alloca(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Alloca) -> Result<()> {
        let num_elements = &inst.num_elements;
        let dest = &inst.dest;
        match parse_operand(num_elements) {
            Some(Op::ConstValue(op)) => {
                match op {
                    ConstValue::Num(_, size) => {
                        // 计算需要分配的栈空间
                        let size = if size >= 8 { size }else{ 8 };
                        VirtualReg::allocate_virt_stack_var(self, func, size, dest.clone());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Ok(())
    }


    /// Handle xori instruction
    /// xori rd, rs1, imm12
    /// Note, XORI rd, rs1, -1 rs1 (assembler pseudoinstruction NOT rd, rs ).
    pub(crate) fn handle_xor(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Xor) -> Result<()> {
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        match (parse_operand(op0), parse_operand(op1)) {
            (Some(Op::LocalValue(op1)), Some(Op::ConstValue(op2))) => {
                // op1 是变量, op2 是立即数
                match op2 {
                    ConstValue::Num(imm, _) => {
                        let local_var = func.find_local_var(op1).ok_or(Error::new("Fail to find local variable"))?;
                        // 尝试分配一个物理寄存器
                        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
                            Some(dest_reg_var) => {
                                // 分配成功
                                match &local_var {
                                    VirtualReg::Stack(rs1_stack_var) => {
                                        // stack_var
                                        let rs1_reg_var = rs1_stack_var.load_stack_var_1(self, prog_inner);
                                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                    },
                                    VirtualReg::Reg(rs1_reg_var) => {
                                        let asm = format!("    xori {}, {}, {}", dest_reg_var.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                    }
                                }
                            },
                            None => {
                                // 分配失败，物理寄存器溢出
                                // 分配一个栈变量
                                let stack_var = VirtualReg::spill_virtual_var(self, func, 8, dest.clone());
                                match &local_var {
                                    VirtualReg::Stack(rs1_stack_var) => {
                                        // stack_var
                                        let rs1_reg_var = rs1_stack_var.load_stack_var_1(self, prog_inner);
                                        // 将栈变量的值写入寄存器中
                                        let asm = format!("\txori {}, {}, {}", rs1_reg_var.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                        // 将寄存器的值加载到栈变量中
                                        let asm = format!("\tsd {}, -{}(fp)", rs1_reg_var.name, stack_var.addr);
                                        self.write_asm(asm);
                                    },
                                    VirtualReg::Reg(rs1_reg_var) => {
                                        // 获取帮助寄存器
                                        let help_reg = prog_inner.get_help_reg_1();
                                        // 将结果写入帮助寄存器中
                                        let asm = format!("\txori {}, {}, {}", help_reg.name, rs1_reg_var.name, imm as i32);
                                        self.write_asm(asm);
                                        // 将帮助寄存器结果写入栈中
                                        let asm = format!("\tsd {}, -{}(fp)", help_reg.name, stack_var.addr);
                                        self.write_asm(asm);
                                    }
                                }
                            }
                        }
                        
                    },
                    _ => { todo!() }
                }
                Ok(())
            },
            _ => { Err(Error::new("Invalid xor instruction")) }
        }
    }

    /// handle store instruction
    /// 将寄存器中的值存入栈中
    pub(crate) fn handle_store(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Store) -> Result<()> {
        let address = &inst.address;
        let value = &inst.value;
        match parse_operand_2(address, value) {
            Some((Op::LocalValue(name), Op::ConstValue(constval))) => {
                let local = func.find_local_var(name).ok_or(Error::new("Fail to find local var"))?;
                match &local {
                    VirtualReg::Stack(stack_var) => {
                        let addr = stack_var.addr;
                        match constval {
                            ConstValue::Num(val, _) => {
                                let help_reg = prog_inner.get_help_reg_1();
                                let asm = format!("    addi {}, zero, {}", help_reg.name, val);
                                self.write_asm(asm);
                                let asm = format!("    sd {}, -{}(fp)", help_reg.name, addr);
                                self.write_asm(asm)
                            },
                            _ => { todo!() }
                        }
                    }
                    VirtualReg::Reg(reg) => {
                        todo!()
                    }
                }
            },
            Some((Op::LocalValue(op1), Op::LocalValue(op2))) => {
                let var1 = func.find_local_var(op1).ok_or(Error::new("Fail to find local var"))?;
                let var2 = func.find_local_var(op2).ok_or(Error::new("Fail to find local var"))?;
                match (var1, var2) {
                    (VirtualReg::Stack(stack), VirtualReg::Reg(reg)) => {
                        let addr = stack.addr;
                        let asm = format!("\tsd {}, -{}(fp)", reg.name, addr);
                        self.write_asm(asm);
                    },
                    (VirtualReg::Stack(stack_var_1), VirtualReg::Stack(stack_var_2)) => {
                        let help_reg_1 = stack_var_2.load_stack_var_1(self, prog_inner);
                        let asm = format!("\tsd {}, -{}(fp)", help_reg_1.name, stack_var_1.addr);
                        self.write_asm(asm);
                    }
                    _ => { todo!() }
                }
            }
            _ =>{ return Err(Error::new("Fail to parse operand"))}
        }
        Ok(())
    }

    /// handle load instruction
    pub(crate) fn handle_load(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Load) -> Result<()> {
        let address = &inst.address;
        let dest = &inst.dest;
        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
            Some(dest_reg_var) => {
                // 可以分配寄存器
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
                    _ => { todo!() }
                }
            },
            None => {
                // 物理寄存器分配溢出
                // 分配栈寄存器
                let dest_stack_var = VirtualReg::spill_virtual_var(self, func, 8, dest.clone());
                match parse_operand(address) {
                    Some(Op::LocalValue(op)) => {
                        let local_virt_reg = func.find_local_var(op).ok_or(Error::new("Fail to find local var"))?;
                        let help_reg = prog_inner.get_help_reg_1();
                        match local_virt_reg {
                            VirtualReg::Stack(src_stack_var) => {
                                // 将内存放入帮助寄存器中
                                let offset = src_stack_var.addr;
                                let asm = format!("\tld {}, -{}(fp)", help_reg.name, offset);
                                self.write_asm(asm);
                                // 将帮助寄存器的值放入栈中
                                let asm = format!("\tsd {}, -{}(fp)", help_reg.name, dest_stack_var.addr);
                                self.write_asm(asm);
                            },
                            _ => { return Err(Error::new("Fail to find stack var")) }
                        }
                    },
                    _ => { todo!() }
                }
            }
        }
        Ok(())
    }

    /// FnOnce(op0, op1, dest)
    pub(crate) fn handle_add_sub_mul_div_mod<F>(
        &self, prog_inner: &mut ProgInner, func: &Function, 
        op0: Operand, op1: Operand, dest: Name, f: F) -> Result<()> 
        where F: FnOnce(String, String, String, InstType) -> String
    {
        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
            Some(dest_reg_var) => {
                self.handle_common_add_sub_mul_div_mod(prog_inner, func, op0, op1, dest_reg_var, f)?;
            }
            None => {
                // 物理寄存器分配溢出
                // 分配栈变量
                let stack_dest_var = VirtualReg::spill_virtual_var(self, func, 8, dest);
                let help_reg = stack_dest_var.load_stack_var_1(self, prog_inner);
                self.handle_common_add_sub_mul_div_mod(prog_inner, func, op0, op1, help_reg.clone(), f)?;
                stack_dest_var.store_stack_var(self, help_reg);
            }
        }
        Ok(())
    }


    /// handle icmp instruction
    pub(crate) fn handle_icmp(&self, prog_inner: &mut ProgInner, func: &Function, inst: &ICmp) -> Result<()> {
        let predicate = &inst.predicate;
        let op0 = &inst.operand0;
        let op1 = &inst.operand1;
        let dest = &inst.dest;
        match VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()) {
            Some(dest_reg_var) => {
                self.handle_common_icmp(prog_inner, func, predicate, op0, op1, dest_reg_var)?;
            },
            None => {
                let stack_dest_var = VirtualReg::spill_virtual_var(self, func, 8, dest.clone());
                let help_reg = stack_dest_var.load_stack_var_1(self, prog_inner);
                self.handle_common_icmp(prog_inner, func, predicate, op0, op1, help_reg.clone())?;
                stack_dest_var.store_stack_var(self, help_reg);
            }
        }
        
        Ok(())
    } 

    /// handle zext
    pub(crate) fn handle_zext(&self, prog_inner: &mut ProgInner, func: &Function, inst: &ZExt) -> Result<()> {
        let dest = &inst.dest;
        let op = &inst.operand;
        if let Some(op) = parse_operand(op) {
            match op {
                Op::LocalValue(name) => {
                    let var = func.find_local_var(name.clone()).ok_or(Error::new("Fail to find var"))?;
                    match var {
                        VirtualReg::Reg(reg) => {
                            func.remove_local_var(name.clone());
                            VirtualReg::insert_virt_reg_var(prog_inner, func, dest.clone(), reg);
                        },
                        VirtualReg::Stack(stack) => {
                            todo!()
                        }
                    }
                }
                Op::ConstValue(_) => { todo!() }
            }
        }else{
            panic!()
        }
        Ok(())
    }


    /// handle ret instruction
    pub(crate) fn handle_ret(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Ret) -> Result<()> {
        // return 
        if let Some(op) = &inst.return_operand {
            match parse_operand(op) {
                Some(Op::ConstValue(constval)) => {
                    match constval {
                        ConstValue::Num(num, _) => {
                            let asm = format!("\tli a0, {}", num);
                            self.write_asm(asm);
                        }
                        _ => { todo!() }
                    }
                }
                Some(Op::LocalValue(name)) => {
                    match func.find_local_var(name) {
                        Some(VirtualReg::Reg(reg_var)) => {
                            let asm = format!("\tmv a0, {}", reg_var.name);
                            self.write_asm(asm);
                        },
                        Some(VirtualReg::Stack(stack_var)) => {
                            let help_reg = stack_var.load_stack_var_1(self, prog_inner);
                            let asm = format!("\tmv a0, {}", help_reg.name);
                            self.write_asm(asm);
                        },
                        _ => { return Err(Error::new("Fail to find local var")) }
                    }
                }
                _ => { return Err(Error::new("Fail to parse operand")) }
            }
        }
        let asm = " \
        \t# function return \n \
        \tmv sp, fp\n \
        \tld fp, 0(sp)\n \
        \tld ra, 8(sp)\n \
        \taddi sp, sp, 16\n \
        \tret\n\n \
        ";
        // self.write_asm("    # function return");
        // self.write_asm("    mv sp, fp");
        // self.write_asm("    ld fp, 0(sp)");
        // self.write_asm("    ld ra, 8(sp)");
        // self.write_asm("    addi sp, sp, 16");
        // self.write_asm("    ret\n\n");
        self.write_asm(asm);
        Ok(())
    }

    /// handle br
    pub(crate) fn handle_br(&self, func: &Function, inst: &Br) -> Result<()> {
        let llvm_name = inst.dest.clone();
        if let Some(label) = func.find_label(llvm_name.clone()) {
            let asm = format!("\tj {}", label.label_name);
            self.write_asm(asm);
            return Ok(())
        }
        Err(Error::LabelNotFoundErr{ err: format!("Fail to found label {}", llvm_name)})
    }

    pub(crate) fn handle_condbr(&self, func: &Function, inst: &CondBr) -> Result<()> {
        let condvar = &inst.condition;
        let true_dest = inst.true_dest.clone();
        let false_dest = inst.false_dest.clone();
        let true_dest = func.find_label(true_dest.clone()).ok_or(Error::LabelNotFoundErr{ err: format!("Fail to find label {:?}", true_dest.clone())})?;
        let false_dest = func.find_label(false_dest.clone()).ok_or(Error::LabelNotFoundErr{ err: format!("Fail to find label {:?}", false_dest.clone())})?;
        if let Some(condvar) = parse_operand(condvar) {
            match condvar {
                Op::LocalValue(llvm_reg) => {
                    let phy_reg = func.find_local_var(llvm_reg.clone()).ok_or(Error::RegNotFoundErr{ err: format!("Fail to find reg {}", llvm_reg)})?;
                    if let VirtualReg::Reg(reg) = phy_reg {
                        let asm = format!("\tbne {}, zero, {}", reg.name, true_dest.label_name);
                        self.write_asm(asm);
                        let asm = format!("\tj {}", false_dest.label_name);
                        self.write_asm(asm);
                        return Ok(())
                    }else{
                        return Err(Error::new(format!("Unexpected variable type, {:?}", llvm_reg)));
                    }
                },
                _ => { return Err(Error::new(format!("Unexpected local variable type, {:?}", condvar)));}
            }
        }
        Err(Error::ParseErr{ err: format!("Fail to found parse {:?}", condvar)})
    }

    /// 处理函数调用
    pub(crate) fn handle_call(&self, prog_inner: &mut ProgInner, func: &Function, inst: &Call) -> Result<()> {
        // STEP1: 将参数放在 a0 - a7 寄存器中，如果还有其他参数，则以从右向左的顺序压栈
        // 第 9 个参数在栈顶位置
        self.store_params(prog_inner, func, inst)?;
        
        // 获取参数上下文
        let param_ctx = func.get_param_context();
        // 保存参数上下文
        func.store_context(self, &param_ctx);

        let dest = inst.dest.clone().ok_or(Error::new("[Call] Fail to get target register"))?;
        // 分配物理寄存器
        let dest_reg_var = VirtualReg::try_allocate_virt_reg_var(prog_inner, func, dest.clone()).ok_or(Error::new("Fail to allocate reg var"))?;
        let func_op = inst.function.clone().right().unwrap();
        // STEP2: 保存上下文
        // 获取上下文
        let ctx = func.get_reg_context();
        func.store_context(self, &ctx);
        // STEP3: 调用 call 指令，执行函数
        if let Some(func) = parse_operand(&func_op) {
            match func {
                Op::ConstValue(val) => {
                    match val {
                        ConstValue::Ref(symbol) => {
                            match symbol {
                                Name::Name(symbol) => {
                                    let asm = format!("\tcall {}", symbol);
                                    self.write_asm(asm);
                                },
                                Name::Number(num) => {
                                    let asm = format!("\tcall {}", num);
                                    self.write_asm(asm);
                                }
                            }
                        },
                        _ => { return Err(Error::new(format!("[Call] Unexpected function type: {:?}",val )))}
                    }
                  
                },
                _ => { panic!() }
            }
        }else{
            return Err(Error::ParseErr{ err: format!("[Call] Fail to parse function {:?}", func_op)})
        }
        // STEP4: 恢复上下文
        func.restore_context(self, &ctx);
        // STEP5： 恢复参数设置
        self.restore_params(inst);
        // STEP6: 获取函数返回值
        let asm = format!("\tmv {}, a0", dest_reg_var.name);
        self.write_asm(asm);

        // STEP7: 恢复参数上下文
        func.restore_context(self, &param_ctx);

        Ok(())
    }
}