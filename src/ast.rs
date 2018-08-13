use std::collections::HashMap;

use runtime::{DefaultScriptInterface, ScriptInterface};

// Language scope struct
#[derive(Debug)]
pub struct Scope {
    pub funcs: HashMap<String, Function>,
    pub vars: HashMap<String, Value>,
}
impl Scope {
    pub fn new() -> Scope {
        Scope {
            funcs: HashMap::new(),
            vars: HashMap::new(),
        }
    }

    pub fn from_args(args: &Vec<(&Ident, &Value)>) -> Scope {
        let mut scope = Scope::new();
        for arg in args {
            scope.vars.insert(arg.0.clone(), arg.1.clone());
        }
        scope
    }
}

// Chain of Scopes
pub struct ScopeChain {
    scopes: Vec<Scope>,
}
impl ScopeChain {
    pub fn new() -> ScopeChain {
        ScopeChain { scopes: vec![] }
    }

    pub fn from_scope(scope: Scope) -> ScopeChain {
        ScopeChain {
            scopes: vec![scope],
        }
    }

    pub fn push(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    pub fn insert_func(&mut self, key: &str, val: Function) {
        match self.scopes.last_mut() {
            Some(ref mut scope) => scope.funcs.insert(key.clone().to_string(), val),
            _ => None,
        };
    }

    pub fn insert_var(&mut self, key: &str, val: Value) {
        match self.scopes.last_mut() {
            Some(ref mut scope) => scope.vars.insert(key.clone().to_string(), val),
            _ => None,
        };
    }

    pub fn resolve_func(&self, key: &str) -> Option<&Function> {
        for scope in self.scopes.iter().rev() {
            match scope.funcs.get(key) {
                Some(x) => return Some(x),
                _ => {}
            }
        }
        None
    }

    pub fn resolve_var(&self, key: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            match scope.vars.get(key) {
                Some(ref x) => return Some(x),
                _ => {}
            }
        }
        None
    }
}

// Statement block
pub type StmtBlock = Vec<Box<Stmt>>;

// Language statements
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let(Ident, Box<Expr>),
    Print(Box<Expr>, bool),
    FnDef(Ident, Vec<Ident>, StmtBlock),
    Return(Box<Expr>),
    If(Box<Expr>, StmtBlock),
    IfElse(Box<Expr>, StmtBlock, StmtBlock),
    Break,
    Loop(StmtBlock),
}

// Language expression: numbers/identifiers and operations thereon
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Int(isize),
    Real(f64),
    Str(String),
    Bool(bool),
    Id(Ident),
    BinOp(Box<Expr>, Opcode, Box<Expr>),
    UnaryOp(Opcode, Box<Expr>),
    FuncCall(Ident, Vec<Box<Expr>>),
}

// Language identifier
pub type Ident = String;

#[derive(Clone, Debug, PartialEq)]
pub enum Opcode {
    Add,
    Div,
    Mod,
    Mul,
    Sub,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
    LogicalXor,
    Not,
}
impl Opcode {
    fn calc_i(&self, l: isize, r: isize) -> isize {
        match *self {
            Opcode::Add => l + r,
            Opcode::Div => l / r,
            Opcode::Mod => l % r,
            Opcode::Mul => l * r,
            Opcode::Sub => l - r,
            _ => 0,
        }
    }

    fn calc_f(&self, l: f64, r: f64) -> f64 {
        match *self {
            Opcode::Add => l + r,
            Opcode::Div => l / r,
            Opcode::Mul => l * r,
            Opcode::Sub => l - r,
            _ => 0f64,
        }
    }

    fn logical(&self, l: Value, r: Value) -> Value {
        match *self {
            Opcode::Equal => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Bool(l == r),
                (Value::Int(l), Value::Real(r)) => Value::Bool(l as f64 == r),
                (Value::Real(l), Value::Int(r)) => Value::Bool(l == r as f64),
                (Value::Real(l), Value::Real(r)) => Value::Bool(l == r),
                (Value::Str(l), Value::Str(r)) => Value::Bool(l == r),
                (_, _) => Value::None,
            },
            Opcode::NotEqual => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Bool(l != r),
                (Value::Int(l), Value::Real(r)) => Value::Bool(l as f64 != r),
                (Value::Real(l), Value::Int(r)) => Value::Bool(l != r as f64),
                (Value::Real(l), Value::Real(r)) => Value::Bool(l != r),
                (Value::Str(l), Value::Str(r)) => Value::Bool(l != r),
                (_, _) => Value::None,
            },
            Opcode::LessThan => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Bool(l < r),
                (Value::Int(l), Value::Real(r)) => Value::Bool((l as f64) < r),
                (Value::Real(l), Value::Int(r)) => Value::Bool(l < r as f64),
                (Value::Real(l), Value::Real(r)) => Value::Bool(l < r),
                (Value::Str(l), Value::Str(r)) => Value::Bool(l < r),
                (_, _) => Value::None,
            },
            Opcode::GreaterThan => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Bool(l > r),
                (Value::Int(l), Value::Real(r)) => Value::Bool(l as f64 > r),
                (Value::Real(l), Value::Int(r)) => Value::Bool(l > r as f64),
                (Value::Real(l), Value::Real(r)) => Value::Bool(l > r),
                (Value::Str(l), Value::Str(r)) => Value::Bool(l > r),
                (_, _) => Value::None,
            },
            Opcode::LessThanOrEqual => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Bool(l <= r),
                (Value::Int(l), Value::Real(r)) => Value::Bool(l as f64 <= r),
                (Value::Real(l), Value::Int(r)) => Value::Bool(l <= r as f64),
                (Value::Real(l), Value::Real(r)) => Value::Bool(l <= r),
                (Value::Str(l), Value::Str(r)) => Value::Bool(l <= r),
                (_, _) => Value::None,
            },
            Opcode::GreaterThanOrEqual => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Bool(l >= r),
                (Value::Int(l), Value::Real(r)) => Value::Bool(l as f64 >= r),
                (Value::Real(l), Value::Int(r)) => Value::Bool(l >= r as f64),
                (Value::Real(l), Value::Real(r)) => Value::Bool(l >= r),
                (Value::Str(l), Value::Str(r)) => Value::Bool(l >= r),
                (_, _) => Value::None,
            },
            Opcode::LogicalAnd => match (l, r) {
                (Value::Bool(l), Value::Bool(r)) => Value::Bool(l && r),
                (_, _) => Value::None,
            },
            Opcode::LogicalOr => match (l, r) {
                (Value::Bool(l), Value::Bool(r)) => Value::Bool(l || r),
                (_, _) => Value::None,
            },
            Opcode::LogicalXor => match (l, r) {
                (Value::Bool(l), Value::Bool(r)) => Value::Bool((l || r) && !(l && r)),
                (_, _) => Value::None,
            },
            _ => Value::None,
        }
    }

    fn eval(&self, l: Value, r: Value) -> Value {
        match *self {
            Opcode::Add | Opcode::Mul | Opcode::Sub => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Int(self.calc_i(l, r)),
                (Value::Int(l), Value::Real(r)) => Value::Real(self.calc_f(l as f64, r)),
                (Value::Real(l), Value::Int(r)) => Value::Real(self.calc_f(l, r as f64)),
                (Value::Real(l), Value::Real(r)) => Value::Real(self.calc_f(l, r)),
                (_, _) => Value::None,
            },
            Opcode::Div => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Real(self.calc_f(l as f64, r as f64)),
                (Value::Int(l), Value::Real(r)) => Value::Real(self.calc_f(l as f64, r)),
                (Value::Real(l), Value::Int(r)) => Value::Real(self.calc_f(l, r as f64)),
                (Value::Real(l), Value::Real(r)) => Value::Real(self.calc_f(l, r)),
                (_, _) => Value::None,
            },
            Opcode::Mod => match (l, r) {
                (Value::Int(l), Value::Int(r)) => Value::Int(self.calc_i(l, r)),
                (_, _) => Value::None,
            },
            Opcode::Equal
            | Opcode::NotEqual
            | Opcode::LessThan
            | Opcode::GreaterThan
            | Opcode::LessThanOrEqual
            | Opcode::GreaterThanOrEqual
            | Opcode::LogicalAnd
            | Opcode::LogicalOr
            | Opcode::LogicalXor => self.logical(l, r),

            _ => Value::None,
        }
    }

    fn eval_unary(&self, x: Value) -> Value {
        match *self {
            Opcode::Not => match x {
                Value::Bool(x) => Value::Bool(!x),
                _ => Value::None,
            },
            _ => Value::None,
        }
    }
}

// Result of evaluating a language element
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    None,
    Int(isize),
    Real(f64),
    Str(String),
    Bool(bool),
}

// Result of executing an Executable
#[derive(Clone, Debug, PartialEq)]
pub enum ExecResult {
    None,
    Error(&'static str),
    Break,
    Return(Value),
}

#[derive(Clone, Debug)]
pub struct Function {
    args: Vec<Ident>,
    stmts: StmtBlock,
}
impl Function {
    pub fn execute<I: ScriptInterface>(
        &self,
        scopes: &mut ScopeChain,
        args: &Vec<Value>,
        iface: &mut I,
    ) -> Value {
        // Create local scope
        let scope = Scope::from_args(
            &self
                .args
                .iter()
                .zip(args)
                .collect::<Vec<(&Ident, &Value)>>(),
        );

        // Push new function scope onto chain
        scopes.push(scope);

        // Evaluate Function StmtBlock
        let res = match self.stmts.exec(scopes, iface) {
            ExecResult::Return(x) => x,
            _ => Value::None,
        };

        // Pop function Scope from chain
        scopes.pop();

        res
    }
}

// Trait allowing various language elements to be evaluated
pub trait Evaluatable {
    fn eval_default(&self, scopes: &mut ScopeChain) -> Value;
    fn eval<I>(&self, scopes: &mut ScopeChain, &mut I) -> Value
    where
        I: ScriptInterface;
}

// Trait allowing statements to be executed
pub trait Executable {
    fn exec_default(&self, scopes: &mut ScopeChain) -> ExecResult;
    fn exec<I>(&self, scopes: &mut ScopeChain, iface: &mut I) -> ExecResult
    where
        I: ScriptInterface;
}

// Evaluate Expr
impl Evaluatable for Expr {
    fn eval_default(&self, scopes: &mut ScopeChain) -> Value {
        self.eval(scopes, &mut DefaultScriptInterface::new())
    }

    fn eval<I>(&self, scopes: &mut ScopeChain, iface: &mut I) -> Value
    where
        I: ScriptInterface,
    {
        match *self {
            Expr::Int(x) => Value::Int(x),
            Expr::Real(x) => Value::Real(x),
            Expr::Str(ref x) => Value::Str(x.to_string()),
            Expr::Bool(x) => Value::Bool(x),
            Expr::BinOp(ref l, ref opc, ref r) => {
                opc.eval(l.eval(scopes, iface), r.eval(scopes, iface))
            }
            Expr::UnaryOp(ref opc, ref x) => opc.eval_unary(x.eval(scopes, iface)),
            Expr::Id(ref x) => match scopes.resolve_var(x) {
                Some(x) => x.clone(),
                None => Value::None,
            },
            Expr::FuncCall(ref func_id, ref args) => {
                let eval_args = args
                    .iter()
                    .map(|x| x.eval(scopes, iface))
                    .collect::<Vec<Value>>();

                // Create clone of Function as resolving it from ScopeChain returns a reference
                // (and therefore immutably borrows "scopes"), then calling Function::execute()
                // requires a mutable borrow to "scopes" which is not allowed
                // TODO: find solution without clone
                let mut func: Option<Function>;
                {
                    func = match scopes.resolve_func(func_id) {
                        Some(f) => Some(f.clone()),
                        None => None,
                    };
                }
                match func {
                    Some(f) => f.execute(scopes, &eval_args, iface),
                    None => Value::None,
                }
            }
        }
    }
}

// Execute Stmt
impl Executable for Stmt {
    fn exec_default(&self, scopes: &mut ScopeChain) -> ExecResult {
        self.exec::<DefaultScriptInterface>(scopes, &mut DefaultScriptInterface::new())
    }

    fn exec<I>(&self, scopes: &mut ScopeChain, iface: &mut I) -> ExecResult
    where
        I: ScriptInterface,
    {
        match *self {
            // Evaluate "expr" and update variable table (key: "id") with result. Value of the Let
            // is None.
            Stmt::Let(ref id, ref expr) => {
                let eval_res = expr.eval(scopes, iface);
                scopes.insert_var(id, eval_res);
                ExecResult::None
            }

            // Evaluate the Expr and print the result
            Stmt::Print(ref expr, nl) => {
                let eval_res = expr.eval(scopes, iface);
                let string = match eval_res {
                    Value::Int(x) => format!("{}", x),
                    Value::Real(x) => format!("{}", x),
                    Value::Str(x) => format!("{}", x),
                    _ => format!("{:?}", eval_res),
                };
                if nl {
                    iface.println(&string)
                } else {
                    iface.print(&string)
                };
                ExecResult::None
            }

            // Create a new Function in the Scope
            Stmt::FnDef(ref fn_id, ref arg_ids, ref stmts) => {
                scopes.insert_func(
                    fn_id,
                    Function {
                        args: arg_ids.clone(),
                        stmts: stmts.clone(),
                    },
                );
                ExecResult::None
            }

            Stmt::Return(ref expr) => ExecResult::Return(expr.eval(scopes, iface)),

            Stmt::If(ref cond, ref stmts) => {
                if let Value::Bool(b) = cond.eval(scopes, iface) {
                    if b {
                        stmts.exec(scopes, iface)
                    } else {
                        ExecResult::None
                    }
                } else {
                    ExecResult::None
                }
            }

            Stmt::IfElse(ref cond, ref stmts, ref else_stmts) => {
                if let Value::Bool(b) = cond.eval(scopes, iface) {
                    if b {
                        stmts.exec(scopes, iface)
                    } else {
                        else_stmts.exec(scopes, iface)
                    }
                } else {
                    else_stmts.exec(scopes, iface)
                }
            }

            Stmt::Break => ExecResult::Break,

            Stmt::Loop(ref stmts) => loop {
                let res = stmts.exec(scopes, iface);
                if let ExecResult::Break = res {
                    return ExecResult::None;
                }
            },
        }
    }
}

// Execute StmtBlock: execute all Stmts in turn
impl Executable for StmtBlock {
    fn exec_default(&self, scopes: &mut ScopeChain) -> ExecResult {
        self.exec(scopes, &mut DefaultScriptInterface::new())
    }

    fn exec<I>(&self, scopes: &mut ScopeChain, iface: &mut I) -> ExecResult
    where
        I: ScriptInterface,
    {
        for stmt in self {
            let res = stmt.exec(scopes, iface);
            if let ExecResult::Return(_) = res {
                return res;
            }
            if let ExecResult::Break = res {
                return ExecResult::Break;
            }
        }
        ExecResult::None
    }
}
