use crate::symbol::{Symbol, SymbolKind, Type, TypeBase};

pub struct SymTable {
    pub symbols: Vec<Symbol>,
    pub current_depth: i32,

    pub owner_idx: Option<usize>,
}

impl SymTable {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            current_depth: 0,
            owner_idx: None,
        }
    }

    // Opens a new scope
    pub fn push_domain(&mut self) {
        self.current_depth += 1;
    }

    // Closes a scope and destroys all variables created inside it
    pub fn drop_domain(&mut self) {
        // Rust's `retain` keeps only the elements that match the condition.
        // We throw away any symbol whose depth matches the current (closing) depth.
        self.symbols.retain(|s| s.depth < self.current_depth);
        self.current_depth -= 1;
    }

    // Searches for a symbol by name.
    // We search backwards (.rev()) so that inner-scope variables
    // shadow outer-scope variables with the same name.
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().rev().find(|s| s.name == name)
    }

    pub fn find_struct_symbol(&self, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|s| s.kind == SymbolKind::Struct && s.name == name)
    }

    // Same as above, but allows us to modify the symbol (like adding args to a function)
    pub fn find_symbol_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.iter_mut().rev().find(|s| s.name == name)
    }

    // Adds a new symbol to the table
    pub fn add_symbol(&mut self, mut sym: Symbol) -> Result<(), String> {
        // Rule: You cannot define two variables with the SAME name at the SAME depth.
        // (However, an inner block *can* redefine a global variable)
        if self
            .symbols
            .iter()
            .any(|s| s.name == sym.name && s.depth == self.current_depth)
        {
            return Err(format!("Symbol redefinition: {}", sym.name));
        }

        sym.depth = self.current_depth;
        self.symbols.push(sym);
        Ok(())
    }

    // Helper to set the 'owner' to the most recently added struct/function
    pub fn set_owner_to_last(&mut self) {
        if !self.symbols.is_empty() {
            self.owner_idx = Some(self.symbols.len() - 1);
        }
    }

    // Helper to clear the owner when we finish parsing a struct/function
    pub fn clear_owner(&mut self) {
        self.owner_idx = None;
    }

    pub fn owner_kind(&self) -> Option<SymbolKind> {
        self.owner_idx
            .and_then(|idx| self.symbols.get(idx))
            .map(|sym| sym.kind.clone())
    }

    pub fn add_owner_member(&mut self, member: Symbol) {
        if let Some(idx) = self.owner_idx {
            if let Some(owner) = self.symbols.get_mut(idx) {
                if let Some(members) = &mut owner.members {
                    members.push(member);
                }
            }
        }
    }

    pub fn add_owner_local(&mut self, local: Symbol) {
        if let Some(idx) = self.owner_idx {
            if let Some(owner) = self.symbols.get_mut(idx) {
                if let Some(locals) = &mut owner.locals {
                    locals.push(local);
                }
            }
        }
    }

    // Helper to get a reference to the current owner (struct/function) if it exists
    pub fn add_ext_func(&mut self, name: &str, return_type: Type) {
        let mut sym = Symbol::new(name.to_string(), SymbolKind::ExtFn, 0);
        sym.type_info = return_type;
        sym.args = Some(Vec::new());

        if let Err(e) = self.add_symbol(sym) {
            panic!("{}", e);
        }
    }

    // Helper to add an argument to an existing function symbol
    pub fn add_ext_func_arg(&mut self, func_name: &str, arg_name: &str, arg_type: Type) {
        let arg = Symbol::new(arg_name.to_string(), SymbolKind::Param, 0);

        if let Some(func) = self.find_symbol_mut(func_name) {
            if let Some(args) = &mut func.args {
                let mut final_arg = arg;
                final_arg.type_info = arg_type;
                args.push(final_arg);
            }
        }
    }

    fn create_type(tb: TypeBase, elements: i32) -> Type {
        let mut t = Type::new();
        t.tb = tb;
        t.elements = elements;
        t
    }

    pub fn add_ext_funcs(&mut self) {
        // void put_s(char s[])
        self.add_ext_func("put_s", Self::create_type(TypeBase::Void, -1));
        self.add_ext_func_arg("put_s", "s", Self::create_type(TypeBase::Char, 0));

        // void get_s(char s[])
        self.add_ext_func("get_s", Self::create_type(TypeBase::Void, -1));
        self.add_ext_func_arg("get_s", "s", Self::create_type(TypeBase::Char, 0));

        // void put_i(int i)
        self.add_ext_func("put_i", Self::create_type(TypeBase::Void, -1));
        self.add_ext_func_arg("put_i", "i", Self::create_type(TypeBase::Int, -1));

        // int get_i()
        self.add_ext_func("get_i", Self::create_type(TypeBase::Int, -1));

        // void put_d(double d)
        self.add_ext_func("put_d", Self::create_type(TypeBase::Void, -1));
        self.add_ext_func_arg("put_d", "d", Self::create_type(TypeBase::Double, -1));

        // double get_d()
        self.add_ext_func("get_d", Self::create_type(TypeBase::Double, -1));

        // void put_c(char c)
        self.add_ext_func("put_c", Self::create_type(TypeBase::Void, -1));
        self.add_ext_func_arg("put_c", "c", Self::create_type(TypeBase::Char, -1));

        // char get_c()
        self.add_ext_func("get_c", Self::create_type(TypeBase::Char, -1));

        // double seconds()
        self.add_ext_func("seconds", Self::create_type(TypeBase::Double, -1));
    }
}
