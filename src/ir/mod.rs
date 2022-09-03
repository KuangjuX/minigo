use std::fs::File;
use llvm_ir::module::{Linkage, GlobalVariable};
use llvm_ir::name::Name;
use llvm_ir::constant::Constant;
use crate::codegen::{Program, Function, FuncParameter};
use crate::codegen::{ Ty, Var, VarValue };
use crate::debug;
use llvm_ir::{Module, Type, self};

pub struct IR {
    pub(crate) module: Module,
}

impl IR {
    pub fn new<S>(bc_file: S) -> Self 
    where S: Into<String> 
    {
        let module = Module::from_bc_path(bc_file.into()).unwrap();
        // for var in module.global_vars.iter() {
        //     println!("[Debug] {:?}", var);
        // }
        Self {
            module
        }
    }

    fn parse_init_value(&self, var: &GlobalVariable) -> Option<VarValue> {
        if let Some(constref) = &var.initializer {
            let initval = &**constref;
            match initval {
                Constant::Int{bits, value} => {
                    Some(VarValue::Int(*value as usize))
                },

                Constant::GetElementPtr(element_ptr) => {
                    let addr = &*element_ptr.address;
                    if let Constant::GlobalReference{name, ty} = addr {
                        let mut name = format!("{}", *name);
                        if name.chars().nth(0) == Some('%') {
                            name.remove(0);
                        }
                        return Some(VarValue::Pointer(name))
                    }
                    None
                },

                Constant::Array{element_type, elements} => {
                    let element_type = &**element_type;
                    let mut array: Vec<usize> = vec![];
                    match element_type {
                        Type::IntegerType { bits } => {
                            for element in elements {
                                let element = &**element;
                                match element {
                                    &Constant::Int{bits, value} => {
                                        array.push(value as usize);
                                    },
                                    _ => {}
                                }
                                
                            }
                            return Some(VarValue::Array{
                                bits: *bits as usize, 
                                elements: array
                            })
                        },
                        _ => { None }
                    }
                }

                _ => {
                    None
                }
            }
        }else{
            None
        }
    }

    /// parse variable type && size
    fn parse_variable_type(&self, var: &GlobalVariable, new_var: &mut Var) {
        // println!("[Debug] var: {:?}\n\n", var);
        let ty = &*var.ty;
        match ty {
            Type::PointerType{ pointee_type, addr_space } => {
                let pointee_ty = &**pointee_type;
                match pointee_ty {
                    Type::IntegerType{ bits } => {
                        let size = bits / 8;
                        match size  {
                            4 => {
                                let init_val = self.parse_init_value(var);
                                if let Some(val) = init_val {
                                    new_var.ty = Ty::I32;
                                    new_var.size = 4;
                                    new_var.init_data = Some(val);
                                }
                            },

                            8 => {
                                let init_val = self.parse_init_value(var);
                                if let Some(val) = init_val {
                                    match val {
                                        VarValue::Int(_) => {
                                            // return (Ty::I64, 8, Some(val))
                                            new_var.ty = Ty::I64;
                                            new_var.size = 8;
                                            new_var.init_data = Some(val);
                                        },
                                        _ => {}
                                    }
                                }

                            },

                            _ => {}
                        }
                    },

                    Type::ArrayType{element_type, num_elements} => {
                        let element_type = &**element_type;
                        match element_type {
                            &Type::IntegerType{ bits} => {
                                if let Some(val) = self.parse_init_value(var) {
                                    new_var.ty = Ty::Array;
                                    if let VarValue::Array{ bits, elements} = &val {
                                        new_var.size = (bits/8) * elements.len();
                                    }
                                    new_var.init_data = Some(val.clone());
                                }
                            },
                            _ => {}
                        }
                    }

                    Type::PointerType{pointee_type, addr_space} => {
                        let init_val = self.parse_init_value(var);
                        new_var.ty = Ty::Pointer;
                        new_var.size = 8;
                        new_var.init_data = init_val;
                    }   
                    _ => {}
                }
            }

            _ => {}
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
            Some(_) => {
                new_var.initiazed = true;
            },

            None => {
                new_var.initiazed = false;
            }
        }

        new_var.is_constant = var.is_constant;
        self.parse_variable_type(var, &mut new_var);
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

    /// parse IR function
    fn parse_function(&self, func: &llvm_ir::Function) -> Function {
        debug!("func: {:?}\n\n", func);
        let mut function = Function::uninit();
        function.name = func.name.clone();
        for param in func.parameters.iter() {
            let ty = &*param.ty.clone();
            match ty {
                &Type::IntegerType{bits} => {
                    let size = bits / 8;
                    match size {
                        4 => {
                            function.stack_size += 4;
                            function.params.push(FuncParameter{
                                ty: Ty::I32,
                                size: 4,
                            });
                            function.stack_size += 4;
                        },
                        8 => {
                            function.stack_size += 8;
                            function.params.push(FuncParameter{
                                ty: Ty::I64,
                                size: 8,
                            });
                            function.stack_size += 8;
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        function
    }

    pub fn parse(&self, asm: &str) -> Program {
        let asm = File::create(asm).unwrap();
        let mut program = Program::new(asm);
        // parse global variable
        for var in self.module.global_vars.iter() {
            let new_var = self.parse_variable(var);
            program.vars.push_back(new_var);
        }
        for function in self.module.functions.iter() {
            let func = self.parse_function(function);
            program.funcs.push_back(func);
        }
        program
    }
}