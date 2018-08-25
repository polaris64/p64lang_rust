use std::collections::HashMap;
use std::rc::Rc;

use ast::{
    Evaluatable, ExecResult, Executable, Expr, Function, Ident, NativeFunction, Opcode, Stmt,
    StmtBlock, Value,
};

/// Language scope struct
///
/// Contains HashMaps mapping Idents to Functions, NativeFunctions and Values (variables) in the
/// scope
pub struct Scope {
    pub funcs: HashMap<Ident, Rc<Function>>,
    pub native_funcs: HashMap<Ident, Rc<NativeFunction>>,
    pub vars: HashMap<Ident, Value>,
}
impl Scope {
    /// Create an empty Scope
    pub fn new() -> Scope {
        Scope {
            funcs: HashMap::new(),
            native_funcs: HashMap::new(),
            vars: HashMap::new(),
        }
    }

    /// When creating a Scope for a Function invocation, inserts variables for each of the
    /// Function's arguments with the values passed to the invocation.
    pub fn from_args(args: &Vec<(&Ident, &Value)>) -> Scope {
        let mut scope = Scope::new();
        for arg in args {
            scope.vars.insert(arg.0.clone(), arg.1.clone());
        }
        scope
    }
}

/// Chain of Scopes
///
///   - A stack of Scopes.
///   - Contains methods to resolve variables, Functions, etc and to modify Scope items.
///   - Each function call pushes a new Scope onto the current ScopeChain.
///   - All evaluations/executions require a ScopeChain.
pub struct ScopeChain {
    scopes: Vec<Scope>,
}
impl ScopeChain {
    /// Creates an empty ScopeChain
    pub fn new() -> ScopeChain {
        ScopeChain { scopes: vec![] }
    }

    /// Creates a new ScopeChain with a single root Scope
    pub fn from_scope(scope: Scope) -> ScopeChain {
        ScopeChain {
            scopes: vec![scope],
        }
    }

    /// Pushes a new Scope onto the stack
    pub fn push(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    /// Pops the last Scope from the stack
    pub fn pop(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    /// Inserts a Function into the last Scope with the Ident `key`
    pub fn insert_func(&mut self, key: &str, val: Function) {
        match self.scopes.last_mut() {
            Some(ref mut scope) => scope.funcs.insert(key.clone().to_string(), Rc::new(val)),
            _ => None,
        };
    }

    /// Inserts a Value `val` into the dict identified by `key` at index `idx`
    pub fn insert_dict_item(&mut self, key: &str, idx: &str, val: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(ref mut scope_val) = scope.vars.get_mut(key) {
                if let Value::Dict(ref mut dict) = scope_val {
                    dict.insert(idx.to_string(), val);
                    break;
                }
            }
        }
    }

    /// Inserts a Value `val` into the list identified by `key` at index `idx`
    pub fn insert_list_item(&mut self, key: &str, idx: usize, val: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(ref mut scope_val) = scope.vars.get_mut(key) {
                if let Value::List(ref mut lst) = scope_val {
                    if lst.len() <= idx {
                        lst.resize(idx + 1, Value::None);
                    }
                    lst[idx] = val;
                    break;
                }
            }
        }
    }

    /// Inserts or updates a Value for a variable identified by `key`
    pub fn insert_var(&mut self, key: &str, val: Value) {
        match self.scopes.last_mut() {
            Some(ref mut scope) => scope.vars.insert(key.clone().to_string(), val),
            _ => None,
        };
    }

    /// Searches from last to first Scope for a Function identified by `key` and returns a
    /// reference
    pub fn resolve_func(&self, key: &str) -> Option<Rc<Function>> {
        for scope in self.scopes.iter().rev() {
            match scope.funcs.get(key) {
                Some(x) => return Some(Rc::clone(x)),
                _ => {}
            }
        }
        None
    }

    /// Searches from last to first Scope for a NativeFunction identified by `key` and returns a
    /// reference
    pub fn resolve_native_func(&self, key: &str) -> Option<Rc<NativeFunction>> {
        for scope in self.scopes.iter().rev() {
            match scope.native_funcs.get(key) {
                Some(x) => return Some(Rc::clone(x)),
                _ => {}
            };
        }
        None
    }

    /// Searches from last to first Scope for a variable identified by `key` and returns a
    /// reference to its Value
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

impl Opcode {
    /// Calculates an Opcode's integer result given left and right operands
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

    /// Calculates an Opcode's floating-point result given left and right operands
    fn calc_f(&self, l: f64, r: f64) -> f64 {
        match *self {
            Opcode::Add => l + r,
            Opcode::Div => l / r,
            Opcode::Mul => l * r,
            Opcode::Sub => l - r,
            _ => 0f64,
        }
    }

    /// Evaluates the Opcode given left and right operands according to the operand types
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

    /// Evaluates the unary Opcode given Value of the operand
    fn eval_unary(&self, x: Value) -> Value {
        match *self {
            Opcode::Not => match x {
                Value::Bool(x) => Value::Bool(!x),
                Value::None => Value::Bool(true),
                _ => Value::Bool(false),
            },
            _ => Value::None,
        }
    }

    /// Calculates an Opcode's logical result given left and right operands
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
}

impl Function {
    /// Executes the Function
    ///
    ///   - Creates a new Function Scope
    ///   - Executes the Function's statements (StmtBlock)
    ///   - Removes the Function's Scope
    ///   - Returns the Function result Value
    pub fn execute(&self, scopes: &mut ScopeChain, args: &Vec<Value>) -> Value {
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
        let res = match self.stmts.exec(scopes) {
            ExecResult::Return(x) => x,
            _ => Value::None,
        };

        // Pop function Scope from chain
        scopes.pop();

        res
    }
}

impl Evaluatable for Expr {
    /// Evaluate an Expr
    fn eval(&self, scopes: &mut ScopeChain) -> Value {
        match *self {
            Expr::BinOp(ref l, ref opc, ref r) => opc.eval(l.eval(scopes), r.eval(scopes)),
            Expr::Bool(x) => Value::Bool(x),
            Expr::Dict(ref items) => {
                let mut map = HashMap::<Ident, Value>::new();
                for item in items.iter() {
                    map.insert(item.0.clone(), item.1.eval(scopes));
                }
                Value::Dict(map)
            },
            Expr::FuncCall(ref func_id, ref args) => {
                let eval_args = args.iter().map(|x| x.eval(scopes)).collect::<Vec<Value>>();

                match scopes.resolve_func(func_id) {
                    Some(f) => f.execute(scopes, &eval_args),
                    None => match scopes.resolve_native_func(func_id) {
                        Some(f) => f.execute(scopes, &eval_args),
                        None => Value::None,
                    },
                }
            }
            Expr::Id(ref x) => match scopes.resolve_var(x) {
                Some(x) => x.clone(),
                None => Value::None,
            },
            Expr::Int(x) => Value::Int(x),
            Expr::List(ref exprs) => {
                Value::List(exprs.iter().map(|x| x.eval(scopes)).collect::<Vec<Value>>())
            }
            Expr::ListElement(ref id, ref expr) => {
                
                // Match index: Value::Str for Dict index, Value::Int for List index
                let coll_idx = expr.eval(scopes);
                let var = scopes.resolve_var(id);

                match var {
                    Some(ref val) => match coll_idx {

                        // Int index: val must be a List
                        Value::Int(idx) => match val {
                            Value::List(ref list) => match list.get(idx as usize) {
                                Some(x) => x.clone(),
                                None => Value::None,
                            },
                            _ => Value::None,
                        },

                        // Str index: val must be a Dict
                        Value::Str(ref s) => match val {
                            Value::Dict(ref dict) => match dict.get(s) {
                                Some(x) => x.clone(),
                                None => Value::None,
                            },
                            _ => Value::None,
                        },

                        _ => Value::None,
                    }
                    None => Value::None,
                }
            }
            Expr::Real(x) => Value::Real(x),
            Expr::Str(ref x) => Value::Str(x.to_string()),
            Expr::UnaryOp(ref opc, ref x) => opc.eval_unary(x.eval(scopes)),
        }
    }
}

impl Executable for Stmt {
    /// Execute a Stmt
    fn exec(&self, scopes: &mut ScopeChain) -> ExecResult {
        match *self {
            // Break from a loop
            Stmt::Break => ExecResult::Break,

            // Single Expr (e.g. function call)
            Stmt::Expr(ref exp) => {
                exp.eval(scopes);
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

            // If condition without an else
            Stmt::If(ref cond, ref stmts) => {
                if let Value::Bool(b) = cond.eval(scopes) {
                    if b {
                        stmts.exec(scopes)
                    } else {
                        ExecResult::None
                    }
                } else {
                    ExecResult::None
                }
            }

            // If condition with an else
            Stmt::IfElse(ref cond, ref stmts, ref else_stmts) => {
                if let Value::Bool(b) = cond.eval(scopes) {
                    if b {
                        stmts.exec(scopes)
                    } else {
                        else_stmts.exec(scopes)
                    }
                } else {
                    else_stmts.exec(scopes)
                }
            }

            // Evaluate "expr" and update variable table (key: "id") with result. Value of the Let
            // is None.
            Stmt::Let(ref id, ref expr) => {
                let eval_res = expr.eval(scopes);
                scopes.insert_var(id, eval_res);
                ExecResult::None
            }

            // Assign a Value to a list item (integer index)
            Stmt::ListItemAssignment(ref id, ref idx, ref val) => {
                let idx = idx.eval(scopes);
                let val = val.eval(scopes);
                match idx {
                    Value::Int(x) => scopes.insert_list_item(id, x as usize, val),
                    Value::Str(x) => scopes.insert_dict_item(id, &x, val),
                    _ => {},
                };
                ExecResult::None
            }

            // Execute a loop until the result of executing a loop Stmt is ExecResult::Break
            Stmt::Loop(ref stmts) => loop {
                let res = stmts.exec(scopes);
                if let ExecResult::Break = res {
                    return ExecResult::None;
                }
            },

            // Return from a Function
            Stmt::Return(ref expr) => ExecResult::Return(expr.eval(scopes)),
        }
    }
}

impl Executable for StmtBlock {
    /// Execute StmtBlock: execute all Stmts in turn, stopping prematurely if an ExecResult::Break
    /// or ExecResult::Return is encountered.
    fn exec(&self, scopes: &mut ScopeChain) -> ExecResult {
        for stmt in self {
            let res = stmt.exec(scopes);
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
