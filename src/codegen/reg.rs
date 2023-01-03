use crate::ir::RegVar;

use super::{  ProgInner };

pub static CALLER_SAVED_REGS: [&str; 16] = [
    "ra", "t0", "t1", "t2", "t3", "t4", "t5", "t6",
    "a0", "a1","a2", "a3", "a4", "a5", "a6", "a7",
];

pub static CALLEE_SAVED_REGD: [&str; 13] = [
    "sp", "fp", "s1", "s2", "s3", "s4", "s5", "s6",
    "s7", "s8", "s9", "s10", "s11"
];

impl ProgInner {
    pub(crate) fn allocate_physical_reg(&mut self) -> Option<PhysicalReg> {
        if let Some(reg) = self.regs.find_free_reg() {
            return Some(reg.clone())
        }
        None
    }

    pub(crate) fn free_physical_reg(&mut self, name: String) -> bool {
        self.regs.free_physical_name(name)
    }

    pub(crate) fn free_all_physical_regs(&mut self) {
        self.regs.free_all_physical_regs();
    }

    /// 获取帮助寄存器 t0
    pub(crate) fn get_help_reg_1(&self) -> PhysicalReg {
        self.regs.get_help_physical_reg_1()
    }

    /// 获取帮助寄存器 t1
    pub(crate) fn get_help_reg_2(&self) -> PhysicalReg {
        self.regs.get_help_physical_reg_2()
    }

}

#[derive(Debug, Clone)]
pub struct PhysicalReg {
    pub(crate) allocated: bool,
    pub(crate) index: usize,
    pub(crate) name: String
}

impl Into<RegVar> for PhysicalReg {
    fn into(self) -> RegVar {
        RegVar { 
            id: self.index,
            name: self.name
         }
    }
}

pub struct PhysicalRegs {
    regs: Vec<PhysicalReg>,
}

impl PhysicalRegs {
    pub fn init() -> Self {
        let mut regs = vec![];
        for i in 1..11 {
            let reg = PhysicalReg{
                allocated: false,
                index: i,
                name: format!("s{}", i)
            };
            regs.push(reg);
        }
        Self{
            regs
        }
    }

    pub fn find_free_reg(&mut self) -> Option<&PhysicalReg> {
        for reg in self.regs.iter_mut() {
            if !reg.allocated {
                reg.allocated = true;
                return Some(reg)
            }
        }
        None
    }

    pub(crate) fn free_physical_name(&mut self, name: String) -> bool {
        for reg in self.regs.iter_mut() {
            if reg.name == name {
                reg.allocated = false;
                return true
            }
        }
        false
    }
    
    pub(crate) fn free_all_physical_regs(&mut self) {
        for reg in self.regs.iter_mut() {
            if reg.allocated {
                reg.allocated = false
            }
        }
    }

    /// 获取一个帮助寄存器
    pub(crate) fn get_help_physical_reg_1(&self) -> PhysicalReg {
        PhysicalReg {
            allocated: false,
            index: 0,
            name: String::from("t0")
        }
    }

    pub(crate) fn get_help_physical_reg_2(&self) -> PhysicalReg {
        PhysicalReg {
            allocated: false,
            index: 1,
            name:String::from("t1")
        }
    }

   
}