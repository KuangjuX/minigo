use llvm_ir::{ Type, TypeRef, Operand, constant::Constant, name::Name };
use crate::codegen::ConstValue;
use crate::codegen::Ty;
use crate::codegen::Op;
use crate::codegen::{ Result, Error };

pub fn align_to(size: usize, align: usize) -> usize {
    let size = (size + align - 1) / align * align;
    size
}

/// parse llvm ir type to minimalgo Ty && size
pub fn parse_type(rawty: &TypeRef) -> Result<(Ty, usize)> {
    let ty = &**rawty;
    match ty {
        &Type::IntegerType { bits } => {
            return Ok((Ty::Num, 8))
        }
        _ => {}
    }
    Err(Error::ParseErr{ err: String::from("Fail to parse type")})
}

pub fn parse_operand(operand: &Operand) -> Option<Op> {
    match operand {
        Operand::ConstantOperand(constref) => {
            let constval = &**constref;
            match &constval {
                Constant::Int{ bits, value} => {
                    return Some(Op::ConstValue(ConstValue::Num(*value as usize, (bits / 8) as usize)))
                }
                Constant::GlobalReference{name, ty} => {
                    return Some(Op::ConstValue(ConstValue::Ref(name.clone())))
                }
                _ => {}
            }
        }
        Operand::LocalOperand{name, ty} => {
            return Some(Op::LocalValue(name.clone()))
        }
        _ => {}
    }
    None
}

pub fn parse_operand_2(op1: &Operand, op2: &Operand) -> Option<(Op, Op)> {
    match (parse_operand(op1), parse_operand(op2)) {
        (Some(ans1), Some(ans2)) => { Some((ans1, ans2)) },
        _ => None
    }
}

// pub fn parse_function(func: Either<InlineAssembly, Operand>) -> Result<Op> {

// }

// pub fn trim_func_name(name: Name) -> Name {
//     match name {
//         Name::Number(_) => { name },
//         Name::Name(symbol) => {
//             let symbol = &symbol[1..symbol.len() - 1];
//             return Name::Name(Box::new(String::from(symbol)))
//         }
//     }
// }