use std::fs::File;
use std::io::{ BufRead, BufReader };
use std::str::FromStr;
use regex::Regex;
use crate::codegen::{Program, Function};
use crate::codegen::{ Ty, Var };

pub struct IR {
    pub(crate) ir_file: File
}

impl IR {
    pub fn new(file: File) -> Self {
        Self {
            ir_file: file
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
    fn parse_variable_type(&self, ty: &str, value: &str) -> (Ty, usize) {
        match ty {
            "i32" => {
                if let Some(size) = self.parse_value(value) {
                    return (Ty::I32, size)
                }else{
                    
                    return (Ty::I32, 0)
                }
            },

            "i64" => {
                if let Some(size) = self.parse_value(value) {
                    // let var = Var::new(Ty::I64, true, None, size, align, String::from_str(name).unwrap());
                    return (Ty::I64, size)
                }else{
                    return (Ty::I64, 0)
                }
            },

            _ => {
                return (Ty::Unknown, 0)
            }
        }
    }

    /// parse global variable
    fn parse_variable(&self, line: String) -> Var {
        let re = Regex::new(r"@(.*?) = (.*?), align ([0-9]*)").unwrap();
        let cap = re.captures(&line).unwrap();
        let name = &cap[1];
        let info = &cap[2];
        let align = usize::from_str_radix(&cap[3], 10).unwrap();
                        
        // println!("name: {}, info: {}, align: {}", name, info, align);
        match info.chars().nth(0)  {
            Some('c') => {
                let re = Regex::new(r"common global (.*) (.*)").unwrap();
                let cap = re.captures(info).unwrap();
                let ty = &cap[1];
                let value = &cap[2];
                let (ty, size) = self.parse_variable_type(ty, value);
                let var = Var::new(ty, true, false, size, align, String::from_str(name).unwrap());
                return var
            },

            Some('g') => {
                let re = Regex::new(r"global (.*) (.*)").unwrap();
                let cap = re.captures(info).unwrap();
                let ty = &cap[1];
                let value = &cap[2];
                let (ty, size) = self.parse_variable_type(ty, value);
                let var = Var::new(ty, true, true, size, align, String::from_str(name).unwrap());
                return var
            },

            Some(_) => {
                return Var::uninit()
            },

            None => {
                return Var::uninit()
            }
        }
    }

    // fn parse_function(&self) -> Function {

    // }

    pub fn parse(&self) -> Program {
        let asm = File::create("main.S").unwrap();
        let mut program = Program::new(asm);
        let mut reader = BufReader::new(&self.ir_file);
        loop {
            let mut line = String::new();
            if let Ok(len) = reader.read_line(&mut line) {
                if len == 0 {
                    break;
                }
                match line.chars().nth(0) {
                    // global variables
                    Some('@') => {
                        let var = self.parse_variable(line);
                        // println!("var: {:?}", var);
                        program.vars.push_back(var);
                    },

                    Some(_) => {

                    }

                    None => {

                    }
                }
            }
        }
        program
    }
}