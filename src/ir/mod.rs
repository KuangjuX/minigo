use std::fs::File;
use llvm_ir::module::{Linkage, GlobalVariable};
use llvm_ir::name::Name;
use crate::codegen::Program;
use crate::codegen::{ Ty, Var };
use llvm_ir::{Module, Type};

pub struct IR {
    pub(crate) module: Module,
}

impl IR {
    pub fn new<S>(bc_file: S) -> Self 
    where S: Into<String> 
    {
        let module = Module::from_bc_path(bc_file.into()).unwrap();
        for var in module.global_vars.iter() {
            println!("[Debug] {:?}", var);
        }
        Self {
            module
        }
    }

    /// parse value
    fn parse_value(&self, value: &str) -> Option<usize> {
        match usize::from_str_radix(value, 10) {
            Ok(value) => {
                return Some(value)
            },

            Err(_) => {
                return None
            }
        }
    }

    /// parse variable type && size
    fn parse_variable_type(&self, var: &GlobalVariable) -> (Ty, usize) {
        let ty = &*var.ty;
        match ty {
            Type::PointerType{ pointee_type, addr_space } => {
                let mut pointee_ty = &**pointee_type;
                match pointee_ty {
                    Type::IntegerType{ bits } => {
                        let size = bits / 8;
                        match size  {
                            4 => {
                                (Ty::I32, 4)
                            },

                            8 => {
                                (Ty::I64, 8)
                            },

                            _ => {
                                (Ty::Unknown, *bits as usize)
                            }
                        }
                    },

                    _ => { (Ty::Unknown, 0) }
                }
            }

            _ => { (Ty::Unknown, 0) }
        }
    }

    /// parse global variable
    fn parse_variable(&self, var: &GlobalVariable) -> Var {
        let mut new_var = Var::uninit();
        new_var.global = true;
        // check link option
        match var.linkage {
            Linkage::Internal => {
                new_var.is_static = true;
            },

            Linkage::External => {
                new_var.is_static = false;
            },

            _ => {}
        }
        // check init
        match &var.initializer {
            Some(constref) => {
                new_var.initiazed = true;
            },

            None => {
                new_var.initiazed = false;
            }
        }

        new_var.is_constant = var.is_constant;
        let (ty, size) = self.parse_variable_type(var);
        new_var.ty = ty;
        new_var.size = size;
        new_var.align = var.alignment as usize;
        match &var.name  {
            Name::Name(name) => {
                let name = format!("{}", *name);
                new_var.name = name;
            },

            Name::Number(num) => {}
        }
        new_var
    }

    // fn parse_function(&self) -> Function {

    // }

    pub fn parse(&self) -> Program {
        let asm = File::create("main.S").unwrap();
        let mut program = Program::new(asm);
        // parse global variable
        for var in self.module.global_vars.iter() {
            let mut new_var = self.parse_variable(var);
            program.vars.push_back(new_var);
        }
        program
    }
}