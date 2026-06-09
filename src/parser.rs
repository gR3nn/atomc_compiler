use crate::symbol::{
    CtVal, MemClass, Ret, Symbol, SymbolKind, Type, TypeBase, arith_type_to, can_be_scalar, conv_to,
};
use crate::symtable::SymTable;
use crate::token::{Token, TokenCode};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    sym_table: SymTable,
    current_fn_type: Option<Type>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut sym_table = SymTable::new();
        sym_table.add_ext_funcs();

        Self {
            tokens,
            pos: 0,
            sym_table,
            current_fn_type: None,
        }
    }

    //get current token
    fn crt_tk(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    //consume token
    fn consume(&mut self, expected: TokenCode) -> bool {
        if let Some(tk) = self.crt_tk() {
            if tk.code == expected {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    fn err(&self, msg: &str) -> ! {
        let line = self.crt_tk().map_or(0, |tk| tk.line);
        eprintln!("Syntax Error at line {}: {}", line, msg);
        std::process::exit(-1);
    }

    fn type_with(tb: TypeBase, elements: i32) -> Type {
        let mut t = Type::new();
        t.tb = tb;
        t.elements = elements;
        t
    }

    //special consume cases for checking IDs and constants

    // Extracts the actual string from an ID token
    fn consume_id_name(&mut self) -> Option<String> {
        if let Some(Token {
            code: TokenCode::ID(name),
            ..
        }) = self.crt_tk()
        {
            let id_name = name.clone();
            self.pos += 1;
            Some(id_name)
        } else {
            None
        }
    }

    //Lexical rules

    //unit: (struct_def | fn_def | var_def)* END

    pub fn parse(&mut self) {
        loop {
            if self.struct_def() {
                continue;
            }
            if self.fn_def() {
                continue;
            }
            if self.var_def() {
                continue;
            }
            break;
        }
        if !self.consume(TokenCode::END) {
            self.err("Expected EOF, struct_def, fn_def or var_def");
        }
    }

    // struct_def: STRUCT ID LACC var_def* RACC SEMICOLON
    fn struct_def(&mut self) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::STRUCT) {
            //Use consume_id_name to get the struct's name
            if let Some(struct_name) = self.consume_id_name() {
                if self.consume(TokenCode::LACC) {
                    //Add Struct to Symbol Table & Open Scope
                    let mut struct_sym = Symbol::new(
                        struct_name.clone(),
                        SymbolKind::Struct,
                        self.sym_table.current_depth,
                    );
                    struct_sym.type_info.tb = TypeBase::Struct;
                    struct_sym.type_info.struct_name = Some(struct_name.clone());
                    struct_sym.members = Some(Vec::new());

                    if let Err(e) = self.sym_table.add_symbol(struct_sym) {
                        self.err(&e);
                    }

                    self.sym_table.set_owner_to_last();
                    self.sym_table.push_domain(); // Open scope for struct members

                    while self.var_def() {} // Parse all internal variables (x, y)

                    if self.consume(TokenCode::RACC) {
                        // Close Struct Scope
                        self.sym_table.drop_domain();
                        self.sym_table.clear_owner();

                        if self.consume(TokenCode::SEMICOLON) {
                            return true;
                        } else {
                            self.err("Missing ';' after struct definition");
                        }
                    } else {
                        self.err("Missing '}' in struct definition");
                    }
                }
            } else {
                self.err("Missing ID after 'struct'");
            }
        }
        self.pos = start_pos;
        false
    }

    // var_def: type_base ID array_decl? (COMMA ID array_decl?)* SEMICOLON
    fn var_def(&mut self) -> bool {
        let start_pos = self.pos;

        if let Some(base_type) = self.type_base() {
            if let Some(var_name) = self.consume_id_name() {
                let mut current_type = base_type.clone();
                self.array_decl(&mut current_type); // Modifies current_type if it's an array
                if current_type.elements == 0 {
                    self.err("A vector variable must have a specified dimension");
                }

                //Add the first variable to the Symbol Table
                let mut sym = Symbol::new(var_name, SymbolKind::Var, self.sym_table.current_depth);
                sym.type_info = current_type;
                sym.mem = match self.sym_table.current_depth {
                    0 => MemClass::Global,
                    _ => MemClass::Local,
                };
                if let Err(e) = self.sym_table.add_symbol(sym) {
                    self.err(&e);
                }

                let added_sym = self
                    .sym_table
                    .symbols
                    .last()
                    .cloned()
                    .unwrap_or_else(|| self.err("Internal error storing declared symbol"));

                match self.sym_table.owner_kind() {
                    Some(SymbolKind::Struct) => self.sym_table.add_owner_member(added_sym),
                    Some(SymbolKind::Fn) => self.sym_table.add_owner_local(added_sym),
                    _ => {}
                }

                // Loop to handle comma-separated variables
                while self.consume(TokenCode::COMMA) {
                    if let Some(next_var_name) = self.consume_id_name() {
                        let mut next_type = base_type.clone();
                        self.array_decl(&mut next_type);
                        if next_type.elements == 0 {
                            self.err("A vector variable must have a specified dimension");
                        }

                        //Add the next variable
                        let mut next_sym = Symbol::new(
                            next_var_name,
                            SymbolKind::Var,
                            self.sym_table.current_depth,
                        );
                        next_sym.type_info = next_type;
                        next_sym.mem = match self.sym_table.current_depth {
                            0 => MemClass::Global,
                            _ => MemClass::Local,
                        };
                        if let Err(e) = self.sym_table.add_symbol(next_sym) {
                            self.err(&e);
                        }

                        let added_sym =
                            self.sym_table.symbols.last().cloned().unwrap_or_else(|| {
                                self.err("Internal error storing declared symbol")
                            });

                        match self.sym_table.owner_kind() {
                            Some(SymbolKind::Struct) => self.sym_table.add_owner_member(added_sym),
                            Some(SymbolKind::Fn) => self.sym_table.add_owner_local(added_sym),
                            _ => {}
                        }
                    } else {
                        self.err("Missing variable name after ','");
                    }
                }

                if self.consume(TokenCode::SEMICOLON) {
                    return true;
                } else {
                    self.err("Missing ';' at the end of variable declaration");
                }
            }
        }

        self.pos = start_pos;
        false
    }

    // type_base [out Type *t]: INT | DOUBLE | CHAR | STRUCT ID
    fn type_base(&mut self) -> Option<Type> {
        let start_pos = self.pos;
        let mut t = Type::new();

        if self.consume(TokenCode::INT) {
            t.tb = TypeBase::Int;
            return Some(t);
        }
        if self.consume(TokenCode::DOUBLE) {
            t.tb = TypeBase::Double;
            return Some(t);
        }
        if self.consume(TokenCode::CHAR) {
            t.tb = TypeBase::Char;
            return Some(t);
        }
        if self.consume(TokenCode::STRUCT) {
            if let Some(name) = self.consume_id_name() {
                //Check if struct exists
                if self.sym_table.find_struct_symbol(&name).is_none() {
                    self.err(&format!("Undefined struct: {}", name));
                }
                t.tb = TypeBase::Struct;
                t.struct_name = Some(name);
                return Some(t);
            } else {
                self.err("Missing ID after 'struct'");
            }
        }

        self.pos = start_pos;
        None
    }

    // array_decl [inout Type *t]: LBRACKET expr? RBRACKET
    fn array_decl(&mut self, t: &mut Type) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::LBRACKET) {
            if self.expr().is_some() {
                t.elements = 1;
            } else {
                t.elements = 0;
            }

            if self.consume(TokenCode::RBRACKET) {
                return true;
            } else {
                self.err("Missing ']' in array declaration");
            }
        }
        self.pos = start_pos;
        false
    }

    // fnDef: (type_base | VOID) ID LPAR (fnParam (COMMA fnParam)*)? RPAR stm_compound
    fn fn_def(&mut self) -> bool {
        let start_pos = self.pos;
        let mut has_type = false;
        let mut return_type = Type::new();

        if let Some(base_type) = self.type_base() {
            has_type = true;
            return_type = base_type;
        } else if self.consume(TokenCode::VOID) {
            has_type = true;
            return_type.tb = TypeBase::Void;
        }

        if has_type {
            if let Some(fn_name) = self.consume_id_name() {
                if self.consume(TokenCode::LPAR) {
                    //Add function to global scope & open param scope
                    let mut fn_sym = Symbol::new(
                        fn_name.clone(),
                        SymbolKind::Fn,
                        self.sym_table.current_depth,
                    );
                    fn_sym.type_info = return_type.clone();
                    fn_sym.args = Some(Vec::new());
                    fn_sym.locals = Some(Vec::new());
                    if let Err(e) = self.sym_table.add_symbol(fn_sym) {
                        self.err(&e);
                    }

                    self.sym_table.set_owner_to_last();
                    self.sym_table.push_domain(); // Open scope for arguments

                    if self.fn_param(&fn_name) {
                        while self.consume(TokenCode::COMMA) {
                            if !self.fn_param(&fn_name) {
                                self.err("Expected function parameter after ','");
                            }
                        }
                    }
                    if self.consume(TokenCode::RPAR) {
                        self.current_fn_type = Some(return_type.clone());

                        if self.stm_compound(false) {
                            self.current_fn_type = None;

                            self.sym_table.drop_domain();
                            self.sym_table.clear_owner();
                            return true;
                        } else {
                            self.err("Expected compound statement '{...}' for function body");
                        }
                    } else {
                        self.err("Missing ')' in function definition");
                    }
                }
            }
        }
        self.pos = start_pos;
        false
    }

    // fnParam: type_base ID array_decl?
    fn fn_param(&mut self, fn_name: &str) -> bool {
        let start_pos = self.pos;
        if let Some(base_type) = self.type_base() {
            if let Some(var_name) = self.consume_id_name() {
                let mut current_type = base_type.clone();
                self.array_decl(&mut current_type); // optional array modification
                if current_type.elements > 0 {
                    current_type.elements = 0;
                }
                let current_depth = self.sym_table.current_depth;

                if let Some(fn_sym) = self.sym_table.find_symbol_mut(fn_name) {
                    if let Some(args) = &mut fn_sym.args {
                        let mut arg_sym =
                            Symbol::new(var_name.clone(), SymbolKind::Param, current_depth);
                        arg_sym.type_info = current_type.clone();
                        args.push(arg_sym);
                    }
                }

                //Add parameter to the current scope
                let mut sym =
                    Symbol::new(var_name, SymbolKind::Param, self.sym_table.current_depth);
                sym.type_info = current_type;
                sym.mem = MemClass::Arg;
                if let Err(e) = self.sym_table.add_symbol(sym) {
                    self.err(&e);
                }

                return true;
            } else {
                self.err("Missing ID in function parameter");
            }
        }
        self.pos = start_pos;
        false
    }

    // stm: stm_compound
    //     | IF LPAR expr RPAR stm (ELSE stm)?
    //     | WHILE LPAR expr RPAR stm
    //     | FOR LPAR expr? SEMICOLON expr? SEMICOLON expr? RPAR stm
    //     | BREAK SEMICOLON
    //     | RETURN expr? SEMICOLON
    //     | expr? SEMICOLON

    fn stm(&mut self) -> bool {
        let start_pos = self.pos;

        //stm_compound
        if self.stm_compound(true) {
            return true;
        }

        // IF LPAR expr RPAR stm (ELSE stm)?
        if self.consume(TokenCode::IF) {
            if !self.consume(TokenCode::LPAR) {
                self.err("Missing '(' after 'if'");
            }
            let r_cond = self
                .expr()
                .unwrap_or_else(|| self.err("Invalid expression in 'if' condition"));

            if !can_be_scalar(&r_cond) {
                self.err("The if condition must be a scalar value");
            }
            if !self.consume(TokenCode::RPAR) {
                self.err("Missing ')' after 'if' condition");
            }
            if !self.stm() {
                self.err("Missing statement for 'if' branch");
            }
            if self.consume(TokenCode::ELSE) {
                if !self.stm() {
                    self.err("Missing statement for 'else' branch");
                }
            }
            return true;
        }

        // WHILE LPAR expr RPAR stm
        if self.consume(TokenCode::WHILE) {
            if !self.consume(TokenCode::LPAR) {
                self.err("Missing '(' after 'while'");
            }
            let r_cond = self
                .expr()
                .unwrap_or_else(|| self.err("Invalid expression in 'while' condition"));

            if !can_be_scalar(&r_cond) {
                self.err("The while condition must be a scalar value");
            }
            if !self.consume(TokenCode::RPAR) {
                self.err("Missing ')' after 'while' condition");
            }
            if !self.stm() {
                self.err("Missing statement for 'while' loop");
            }
            return true;
        }

        // FOR LPAR expr? SEMICOLON expr? SEMICOLON expr? RPAR stm
        if self.consume(TokenCode::FOR) {
            if !self.consume(TokenCode::LPAR) {
                self.err("Missing '(' after 'for'");
            }

            // init expression: optional, no scalar check needed
            self.expr();

            if !self.consume(TokenCode::SEMICOLON) {
                self.err("Missing first ';' in 'for' loop");
            }

            // condition expression: optional, but if present it must be scalar
            if let Some(r_cond) = self.expr() {
                if !can_be_scalar(&r_cond) {
                    self.err("The for condition must be a scalar value");
                }
            }

            if !self.consume(TokenCode::SEMICOLON) {
                self.err("Missing second ';' in 'for' loop");
            }

            // step expression: optional, no scalar check needed
            self.expr();

            if !self.consume(TokenCode::RPAR) {
                self.err("Missing ')' in 'for' loop");
            }

            if !self.stm() {
                self.err("Missing statement for 'for' loop body");
            }

            return true;
        }

        // BREAK SEMICOLON
        if self.consume(TokenCode::BREAK) {
            if self.consume(TokenCode::SEMICOLON) {
                return true;
            }
            self.err("Missing ';' after 'break'");
        }

        // RETURN expr? SEMICOLON
        if self.consume(TokenCode::RETURN) {
            let fn_type = self
                .current_fn_type
                .clone()
                .unwrap_or_else(|| self.err("'return' used outside of a function"));

            let r_expr = self.expr();

            if fn_type.tb == TypeBase::Void {
                if r_expr.is_some() {
                    self.err("A void function cannot return a value");
                }
            } else {
                let r_expr =
                    r_expr.unwrap_or_else(|| self.err("A non-void function must return a value"));

                if !can_be_scalar(&r_expr) {
                    self.err("The return value must be a scalar value");
                }

                if !conv_to(&r_expr.type_info, &fn_type) {
                    self.err(
                        "Cannot convert the return expression type to the function return type",
                    );
                }
            }

            if self.consume(TokenCode::SEMICOLON) {
                return true;
            }

            self.err("Missing ';' after 'return'");
        }

        // expr? SEMICOLON
        if self.expr().is_some() {
            if self.consume(TokenCode::SEMICOLON) {
                return true;
            }
            self.err("Missing ';' after expression statement");
        } else if self.consume(TokenCode::SEMICOLON) {
            return true; // empty statement
        }

        self.pos = start_pos;
        false
    }

    // stm_compound [in bool newDomain]: LACC (var_def | stm)* RACC
    fn stm_compound(&mut self, new_domain: bool) -> bool {
        let start_pos = self.pos;
        if self.consume(TokenCode::LACC) {
            // Open scope
            if new_domain {
                self.sym_table.push_domain();
            }

            loop {
                if self.var_def() {
                    continue;
                }
                if self.stm() {
                    continue;
                }
                break;
            }
            if self.consume(TokenCode::RACC) {
                // Close scope
                if new_domain {
                    self.sym_table.drop_domain();
                }

                return true;
            } else {
                self.err("Missing '}' to close compound statement");
            }
        }
        self.pos = start_pos;
        false
    }
    //Expressions

    //expr: expr_assign

    fn expr(&mut self) -> Option<Ret> {
        self.expr_assign()
    }

    //expr_assign: _u ASSIGN expr_assign | expr_or

    fn expr_assign(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(r_dst) = self._u() {
            if self.consume(TokenCode::ASSIGN) {
                let mut r_src = self
                    .expr_assign()
                    .unwrap_or_else(|| self.err("Missing expression after '='"));

                if !r_dst.lval {
                    self.err("The assign destination must be a left-value");
                }

                if r_dst.ct {
                    self.err("The assign destination cannot be constant");
                }

                if !can_be_scalar(&r_dst) {
                    self.err("The assign destination must be scalar");
                }

                if !can_be_scalar(&r_src) {
                    self.err("The assign source must be scalar");
                }

                if !conv_to(&r_src.type_info, &r_dst.type_info) {
                    self.err("The assign source cannot be converted to destination");
                }

                r_src.lval = false;
                r_src.ct = true;
                r_src.ct_val = None;

                return Some(r_src);
            }
        }

        self.pos = start_pos;
        self.expr_or()
    }

    //expr_or: expr_and (OR expr_and)*

    fn expr_or(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_and() {
            loop {
                if self.consume(TokenCode::OR) {
                    let right = self
                        .expr_and()
                        .unwrap_or_else(|| self.err("Expected expression after '||'"));

                    if arith_type_to(&r.type_info, &right.type_info).is_none() {
                        self.err("Invalid operand type for '||'");
                    }

                    let mut int_type = Type::new();
                    int_type.tb = TypeBase::Int;
                    int_type.elements = -1;

                    r.type_info = int_type;
                    r.lval = false;
                    r.ct = true;
                    r.ct_val = None;

                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    //expr_and: expr_eq (AND expr_eq)*

    fn expr_and(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_eq() {
            loop {
                if self.consume(TokenCode::AND) {
                    let right = self
                        .expr_eq()
                        .unwrap_or_else(|| self.err("Expected expression after '&&'"));

                    if arith_type_to(&r.type_info, &right.type_info).is_none() {
                        self.err("Invalid operand type for '&&'");
                    }

                    let mut int_type = Type::new();
                    int_type.tb = TypeBase::Int;
                    int_type.elements = -1;

                    r.type_info = int_type;
                    r.lval = false;
                    r.ct = true;
                    r.ct_val = None;

                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    //expr_eq: expr_rel ((EQUAL | NOTEQ) expr_rel)*

    fn expr_eq(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_rel() {
            loop {
                if self.consume(TokenCode::EQUAL) || self.consume(TokenCode::NOTEQ) {
                    let right = self
                        .expr_rel()
                        .unwrap_or_else(|| self.err("Missing expression after '==' or '!='"));

                    if arith_type_to(&r.type_info, &right.type_info).is_none() {
                        self.err("Invalid operand type for equality operator");
                    }

                    let mut int_type = Type::new();
                    int_type.tb = TypeBase::Int;
                    int_type.elements = -1;

                    r.type_info = int_type;
                    r.lval = false;
                    r.ct = true;
                    r.ct_val = None;

                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    //expr_rel: expr_add ((LESS | LESSEQ | GREATER | GREATEREQ) expr_add)*

    fn expr_rel(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_add() {
            loop {
                if self.consume(TokenCode::LESS)
                    || self.consume(TokenCode::LESSEQ)
                    || self.consume(TokenCode::GREATER)
                    || self.consume(TokenCode::GREATEREQ)
                {
                    let right = self.expr_add().unwrap_or_else(|| {
                        self.err("Missing expression after relational operator")
                    });

                    if arith_type_to(&r.type_info, &right.type_info).is_none() {
                        self.err("Invalid operand type for relational operator");
                    }

                    let mut int_type = Type::new();
                    int_type.tb = TypeBase::Int;
                    int_type.elements = -1;

                    r.type_info = int_type;
                    r.lval = false;
                    r.ct = true;
                    r.ct_val = None;

                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    //expr_add: expr_mul ((ADD | SUB) expr_mul)*

    fn expr_add(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_mul() {
            loop {
                if self.consume(TokenCode::ADD) || self.consume(TokenCode::SUB) {
                    let right = self
                        .expr_mul()
                        .unwrap_or_else(|| self.err("Expected expression after '+' or '-'"));

                    if let Some(result_type) = arith_type_to(&r.type_info, &right.type_info) {
                        r.type_info = result_type;
                        r.lval = false;
                        r.ct = true;
                        r.ct_val = None;
                    } else {
                        self.err("Invalid operand type for '+' or '-'");
                    }

                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    //expr_mul: expr_cast ((MUL | DIV) expr_cast)*

    fn expr_mul(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_cast() {
            loop {
                if self.consume(TokenCode::MUL) || self.consume(TokenCode::DIV) {
                    let right = self
                        .expr_cast()
                        .unwrap_or_else(|| self.err("Expected expression after '*' or '/'"));

                    if let Some(result_type) = arith_type_to(&r.type_info, &right.type_info) {
                        r.type_info = result_type;
                        r.lval = false;
                        r.ct = true;
                        r.ct_val = None;
                    } else {
                        self.err("Invalid operand type for '*' or '/'");
                    }

                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    // expr_cast: LPAR type_base array_decl? RPAR expr_cast | expr_unary
    fn expr_cast(&mut self) -> Option<Ret> {
        let start_pos = self.pos;
        if self.consume(TokenCode::LPAR) {
            if let Some(mut base_type) = self.type_base() {
                self.array_decl(&mut base_type);
                if self.consume(TokenCode::RPAR) {
                    let mut r = self
                        .expr_cast()
                        .unwrap_or_else(|| self.err("Missing expression after cast"));

                    let cast_is_array = base_type.elements >= 0;
                    let expr_is_array = r.type_info.elements >= 0;

                    if base_type.tb == TypeBase::Struct || r.type_info.tb == TypeBase::Struct {
                        self.err("Cannot cast to or from a struct type");
                    }

                    if cast_is_array != expr_is_array {
                        self.err("Cannot cast between scalar and array types");
                    }

                    r.type_info = base_type;
                    r.lval = false;
                    r.ct = false;
                    r.ct_val = None;
                    return Some(r);
                }
            }

            self.pos = start_pos;
        }

        self._u()
    }

    // _u: (SUB | NOT) _u | expr_postfix

    fn _u(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if self.consume(TokenCode::SUB) {
            if let Some(mut r) = self._u() {
                if !can_be_scalar(&r) {
                    self.err("Unary operator must have a scalar operand");
                }

                r.lval = false;
                r.ct = false;
                r.ct_val = None;

                return Some(r);
            }

            self.err("Missing expression after unary operator");
        }

        if self.consume(TokenCode::NOT) {
            let r = self
                ._u()
                .unwrap_or_else(|| self.err("Missing expression after unary operator"));

            if !can_be_scalar(&r) {
                self.err("Unary operator must have a scalar operand");
            }

            return Some(Ret::new(Self::type_with(TypeBase::Int, -1), false, false));
        }

        self.pos = start_pos;

        self.expr_postfix()
    }

    // expr_postfix: expr_primary (LBRACKET expr RBRACKET | DOT ID)*

    fn expr_postfix(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        if let Some(mut r) = self.expr_primary() {
            loop {
                // array indexing: v[i]
                if self.consume(TokenCode::LBRACKET) {
                    let idx = self
                        .expr()
                        .unwrap_or_else(|| self.err("Missing expression inside '[...]'"));

                    if !self.consume(TokenCode::RBRACKET) {
                        self.err("Missing ']' after array index");
                    }

                    if r.type_info.elements < 0 {
                        self.err("Only an array can be indexed");
                    }

                    let int_type = Self::type_with(TypeBase::Int, -1);
                    if !conv_to(&idx.type_info, &int_type) {
                        self.err("Array index is not convertible to int");
                    }

                    // After indexing, the result is one element, not an array
                    r.type_info.elements = -1;
                    r.lval = true;
                    r.ct = false;
                    r.ct_val = None;

                    continue;
                }

                // struct field access: p.x
                if self.consume(TokenCode::DOT) {
                    let field_name = self
                        .consume_id_name()
                        .unwrap_or_else(|| self.err("Missing ID after '.'"));

                    if r.type_info.tb != TypeBase::Struct {
                        self.err("A field can only be selected from a struct");
                    }

                    let struct_name =
                        r.type_info.struct_name.clone().unwrap_or_else(|| {
                            self.err("Anonymous struct types are not supported")
                        });

                    let struct_sym = self
                        .sym_table
                        .find_struct_symbol(&struct_name)
                        .unwrap_or_else(|| self.err(&format!("Undefined struct: {}", struct_name)));

                    if struct_sym.kind != SymbolKind::Struct {
                        self.err(&format!("'{}' is not a structure type", struct_name));
                    }

                    let field = struct_sym
                        .members
                        .as_ref()
                        .and_then(|members| members.iter().find(|member| member.name == field_name))
                        .cloned()
                        .unwrap_or_else(|| {
                            self.err(&format!(
                                "The structure {} does not have a field {}",
                                struct_name, field_name
                            ))
                        });

                    r = Ret::new(field.type_info.clone(), true, field.type_info.elements >= 0);
                    continue;
                }

                break;
            }

            return Some(r);
        }

        self.pos = start_pos;
        None
    }

    // expr_primary: ID (LPAR (expr (COMMA expr)*)? RPAR)? | CT_INT | CT_REAL | CT_CHAR | CT_STRING | LPAR expr RPAR

    fn expr_primary(&mut self) -> Option<Ret> {
        let start_pos = self.pos;

        // ID or function call
        if let Some(id_name) = self.consume_id_name() {
            let sym = self
                .sym_table
                .find_symbol(&id_name)
                .unwrap_or_else(|| self.err(&format!("Undefined id: {}", id_name)))
                .clone();

            // Function call: ID(...)
            if self.consume(TokenCode::LPAR) {
                if sym.kind != SymbolKind::Fn && sym.kind != SymbolKind::ExtFn {
                    self.err("Only a function can be called");
                }

                let mut args = Vec::new();
                if let Some(arg) = self.expr() {
                    args.push(arg);
                    while self.consume(TokenCode::COMMA) {
                        args.push(self.expr().unwrap_or_else(|| {
                            self.err("Expected expression after ',' in function call")
                        }));
                    }
                }

                if !self.consume(TokenCode::RPAR) {
                    self.err("Missing ')' after function call arguments");
                }

                let expected_args = sym.args.clone().unwrap_or_default();
                if args.len() != expected_args.len() {
                    self.err(&format!(
                        "Function '{}' called with {} arguments, expected {}",
                        id_name,
                        args.len(),
                        expected_args.len()
                    ));
                }

                for (arg, param) in args.iter().zip(expected_args.iter()) {
                    if !conv_to(&arg.type_info, &param.type_info) {
                        self.err(&format!("Argument type mismatch in call to '{}'", id_name));
                    }
                }

                return Some(Ret::new(sym.type_info.clone(), false, false));
            }

            // Normal identifier, not a function call
            if sym.kind == SymbolKind::Fn || sym.kind == SymbolKind::ExtFn {
                self.err("A function can only be called");
            }

            let is_array = sym.type_info.elements >= 0;

            return Some(Ret::new(sym.type_info.clone(), true, is_array));
        }

        // Integer constant
        if let Some(Token {
            code: TokenCode::CtInt(value),
            ..
        }) = self.crt_tk()
        {
            let value = *value;
            self.pos += 1;

            let mut t = Type::new();
            t.tb = TypeBase::Int;
            t.elements = -1;

            let mut r = Ret::new(t, false, true);
            r.ct_val = Some(CtVal::Int(value));
            return Some(r);
        }

        // Real constant
        if let Some(Token {
            code: TokenCode::CtReal(value),
            ..
        }) = self.crt_tk()
        {
            let value = *value;
            self.pos += 1;

            let mut t = Type::new();
            t.tb = TypeBase::Double;
            t.elements = -1;

            let mut r = Ret::new(t, false, true);
            r.ct_val = Some(CtVal::Double(value));
            return Some(r);
        }

        // Character constant
        if let Some(Token {
            code: TokenCode::CtChar(value),
            ..
        }) = self.crt_tk()
        {
            let value = *value;
            self.pos += 1;

            let mut t = Type::new();
            t.tb = TypeBase::Char;
            t.elements = -1;

            let mut r = Ret::new(t, false, true);
            r.ct_val = Some(CtVal::Char(value));
            return Some(r);
        }

        // String constant
        if let Some(Token {
            code: TokenCode::CtString(value),
            ..
        }) = self.crt_tk()
        {
            let value = value.clone();
            self.pos += 1;

            let mut t = Type::new();
            t.tb = TypeBase::Char;
            t.elements = 0; // char array

            let mut r = Ret::new(t, false, true);
            r.ct_val = Some(CtVal::Str(value));
            return Some(r);
        }

        // Parenthesized expression: (expr)
        if self.consume(TokenCode::LPAR) {
            if let Some(r) = self.expr() {
                if self.consume(TokenCode::RPAR) {
                    return Some(r);
                } else {
                    self.err("Missing ')' to close grouped expression");
                }
            } else {
                self.err("Missing expression after '('");
            }
        }

        self.pos = start_pos;
        None
    }
}
