#[derive(Debug, Clone, PartialEq)]
pub enum TypeBase {
    Int,
    Double,
    Char,
    Struct,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Var,
    Fn,
    ExtFn,
    Struct,
    Param,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemClass {
    Global,
    Arg,
    Local,
    NotApplicable, //used for symbols where memory class doesn't apply (e.g., structs, functions)
}

#[derive(Debug, Clone)]
pub struct Type {
    pub tb: TypeBase,
    pub struct_name: Option<String>, // Safely stores the name if tb == Struct
    pub elements: i32,               // <0 for non-array, 0 for size-less array, >0 for sized array
}

impl Type {
    // quick helper to create a default empty type
    pub fn new() -> Self {
        Self {
            tb: TypeBase::Void,
            struct_name: None,
            elements: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CtVal {
    Int(i64),
    Double(f64),
    Char(char),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct Ret {
    pub type_info: Type,
    pub lval: bool,
    pub ct: bool,
    pub ct_val: Option<CtVal>,
}

impl Ret {
    pub fn new(type_info: Type, lval: bool, ct: bool) -> Self {
        Self {
            type_info,
            lval,
            ct,
            ct_val: None,
        }
    }
}

// Helper to check if a return type can be used in scalar contexts
pub fn can_be_scalar(ret: &Ret) -> bool {
    ret.type_info.elements < 0 && ret.type_info.tb != TypeBase::Struct
}

// Helper to check if two types can be converted for assignment or arithmetic
pub fn conv_to(src: &Type, dst: &Type) -> bool {
    // arrays can only convert to arrays of same base type
    if src.elements >= 0 || dst.elements >= 0 {
        return src.elements >= 0 && dst.elements >= 0 && src.tb == dst.tb;
    }

    match (&src.tb, &dst.tb) {
        (TypeBase::Char, TypeBase::Char)
        | (TypeBase::Char, TypeBase::Int)
        | (TypeBase::Char, TypeBase::Double)
        | (TypeBase::Int, TypeBase::Char)
        | (TypeBase::Int, TypeBase::Int)
        | (TypeBase::Int, TypeBase::Double)
        | (TypeBase::Double, TypeBase::Char)
        | (TypeBase::Double, TypeBase::Int)
        | (TypeBase::Double, TypeBase::Double) => true,

        (TypeBase::Struct, TypeBase::Struct) => src.struct_name == dst.struct_name,

        _ => false,
    }
}

// Helper to determine the resulting type of an arithmetic operation between two types
pub fn arith_type_to(t1: &Type, t2: &Type) -> Option<Type> {
    if t1.elements >= 0 || t2.elements >= 0 {
        return None;
    }

    if t1.tb == TypeBase::Struct || t2.tb == TypeBase::Struct {
        return None;
    }

    let mut result = Type::new();
    result.elements = -1;

    if t1.tb == TypeBase::Double || t2.tb == TypeBase::Double {
        result.tb = TypeBase::Double;
    } else if t1.tb == TypeBase::Int || t2.tb == TypeBase::Int {
        result.tb = TypeBase::Int;
    } else {
        result.tb = TypeBase::Char;
    }

    Some(result)
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub mem: MemClass,
    pub type_info: Type,
    pub depth: i32, // 0 = global, 1 = function, 2+ = nested blocks

    pub args: Option<Vec<Symbol>>,    // For functions
    pub locals: Option<Vec<Symbol>>,  // For functions
    pub members: Option<Vec<Symbol>>, // For structs
}

impl Symbol {
    // Constructor for a new basic symbol
    pub fn new(name: String, kind: SymbolKind, depth: i32) -> Self {
        Self {
            name,
            kind,
            mem: MemClass::NotApplicable,
            type_info: Type::new(),
            depth,
            args: None,
            locals: None,
            members: None,
        }
    }
}
