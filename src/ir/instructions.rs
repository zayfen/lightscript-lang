//! IR Instructions for LightLang

use std::fmt;

/// IR Value (represents a value in IR)
#[derive(Debug, Clone, PartialEq)]
pub enum IRValue {
    Const(i64),
    ConstF(f64),
    ConstStr(String),
    Var(String),
    Label(String),
}

impl fmt::Display for IRValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRValue::Const(n) => write!(f, "{}", n),
            IRValue::ConstF(n) => write!(f, "{}", n),
            IRValue::ConstStr(s) => write!(f, "\"{}\"", s),
            IRValue::Var(name) => write!(f, "%{}", name),
            IRValue::Label(name) => write!(f, "@{}", name),
        }
    }
}

/// IR Type
#[derive(Debug, Clone, PartialEq)]
pub enum IRType {
    I64,
    F64,
    String,
    Bool,
    Void,
}

impl fmt::Display for IRType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRType::I64 => write!(f, "i64"),
            IRType::F64 => write!(f, "f64"),
            IRType::String => write!(f, "str"),
            IRType::Bool => write!(f, "bool"),
            IRType::Void => write!(f, "void"),
        }
    }
}

/// IR Instructions
#[derive(Debug, Clone)]
pub enum IRInstruction {
    // Arithmetic
    Add {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    Sub {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    Mul {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    Div {
        dest: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
    
    // Memory
    Alloc {
        dest: String,
        ty: IRType,
    },
    Store {
        dest: String,
        ty: IRType,
        value: IRValue,
    },
    Load {
        dest: String,
        ty: IRType,
        ptr: String,
    },
    
    // Control Flow
    Ret {
        ty: IRType,
        value: Option<IRValue>,
    },
    Branch {
        label: String,
    },
    CondBranch {
        cond: IRValue,
        true_label: String,
        false_label: String,
    },
    Label {
        name: String,
    },
    
    // Call
    Call {
        dest: Option<String>,
        ret_ty: IRType,
        func: String,
        args: Vec<(IRType, IRValue)>,
    },
    
    // Comparison
    Cmp {
        dest: String,
        op: String,
        ty: IRType,
        lhs: IRValue,
        rhs: IRValue,
    },
}

impl fmt::Display for IRInstruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IRInstruction::Add { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = add {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Sub { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = sub {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Mul { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = mul {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Div { dest, ty, lhs, rhs } => {
                write!(f, "  %{} = div {} {}, {}", dest, ty, lhs, rhs)
            },
            IRInstruction::Alloc { dest, ty } => {
                write!(f, "  %{} = alloc {}", dest, ty)
            },
            IRInstruction::Store { dest, ty, value } => {
                write!(f, "  store {} {}, %{}", ty, value, dest)
            },
            IRInstruction::Load { dest, ty, ptr } => {
                write!(f, "  %{} = load {} %{}", dest, ty, ptr)
            },
            IRInstruction::Ret { ty, value } => {
                if let Some(v) = value {
                    write!(f, "  ret {} {}", ty, v)
                } else {
                    write!(f, "  ret void")
                }
            },
            IRInstruction::Branch { label } => {
                write!(f, "  br @{}", label)
            },
            IRInstruction::CondBranch { cond, true_label, false_label } => {
                write!(f, "  br {}, @{}, @{}", cond, true_label, false_label)
            },
            IRInstruction::Label { name } => {
                write!(f, "@{}:", name)
            },
            IRInstruction::Call { dest, ret_ty, func, args } => {
                let args_str: Vec<String> = args.iter()
                    .map(|(ty, val)| format!("{} {}", ty, val))
                    .collect();
                if let Some(d) = dest {
                    write!(f, "  %{} = call {} @{}({})", d, ret_ty, func, args_str.join(", "))
                } else {
                    write!(f, "  call {} @{}({})", ret_ty, func, args_str.join(", "))
                }
            },
            IRInstruction::Cmp { dest, op, ty, lhs, rhs } => {
                write!(f, "  %{} = icmp {} {} {}, {}", dest, op, ty, lhs, rhs)
            },
        }
    }
}

/// IR Function
#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<(String, IRType)>,
    pub ret_ty: IRType,
    pub instructions: Vec<IRInstruction>,
}

impl IRFunction {
    pub fn new(name: String, ret_ty: IRType) -> Self {
        IRFunction {
            name,
            params: Vec::new(),
            ret_ty,
            instructions: Vec::new(),
        }
    }
    
    pub fn add_param(&mut self, name: String, ty: IRType) {
        self.params.push((name, ty));
    }
    
    pub fn add_instruction(&mut self, instr: IRInstruction) {
        self.instructions.push(instr);
    }
}

impl fmt::Display for IRFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let params_str: Vec<String> = self.params.iter()
            .map(|(name, ty)| format!("{} %{}", ty, name))
            .collect();
        
        writeln!(f, "define {} @{}({}) {{", self.ret_ty, self.name, params_str.join(", "))?;
        
        for instr in &self.instructions {
            writeln!(f, "{}", instr)?;
        }
        
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_value() {
        let val = IRValue::Const(42);
        assert_eq!(format!("{}", val), "42");
        
        let var = IRValue::Var("x".to_string());
        assert_eq!(format!("{}", var), "%x");
    }

    #[test]
    fn test_ir_function() {
        let func = IRFunction::new("main".to_string(), IRType::I64);
        assert_eq!(func.name, "main");
    }
}
