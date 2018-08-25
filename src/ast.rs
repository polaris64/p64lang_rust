use std::any::Any;
use std::collections::HashMap;

use interpreter::ScopeChain;

// --- Types ---

/// Result of executing an Executable
#[derive(Clone, Debug, PartialEq)]
pub enum ExecResult {
    Break,
    Error(&'static str),
    None,
    Return(Value),
}

/// Language expression
///
/// Numbers, strings, lists, function calls, identifiers and operations thereon. Anything that can
/// be evaluated to a Value.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    BinOp(Box<Expr>, Opcode, Box<Expr>),
    Bool(bool),
    Dict(Vec<(Ident, Box<Expr>)>),
    FuncCall(Ident, Vec<Box<Expr>>),
    Id(Ident),
    Int(isize),
    ListElement(Ident, Box<Expr>),
    List(Vec<Box<Expr>>),
    Real(f64),
    Str(String),
    UnaryOp(Opcode, Box<Expr>),
}

/// Script-defined functions
///
/// Contains a list of statements (StmtBlock) that are executed when the Function is called, and a
/// list of argument Idents that will be assigned to actual values during the call.
#[derive(Debug)]
pub struct Function {
    pub args: Vec<Ident>,
    pub stmts: StmtBlock,
}

/// Language identifier
///
/// Used to represent a variable or function name.
pub type Ident = String;

/// Operation codes
///
/// Contains variants representing various operations that can be performed on expressions, such as
/// arithmetic, logical and relational.
#[derive(Clone, Debug, PartialEq)]
pub enum Opcode {
    Add,
    Div,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    LogicalAnd,
    LogicalOr,
    LogicalXor,
    Mod,
    Mul,
    Not,
    NotEqual,
    Sub,
}

/// Language statements
///
/// Any single program instruction, such as a variable assignment, function call, conditional,
/// loop.
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Break,
    Expr(Box<Expr>),
    FnDef(Ident, Vec<Ident>, StmtBlock),
    If(Box<Expr>, StmtBlock),
    IfElse(Box<Expr>, StmtBlock, StmtBlock),
    Let(Ident, Box<Expr>),
    ListItemAssignment(Ident, Box<Expr>, Box<Expr>),
    Loop(StmtBlock),
    Return(Box<Expr>),
}

/// Statement block
///
/// A block of zero or more Stmts
pub type StmtBlock = Vec<Box<Stmt>>;

/// Result of evaluating an Evaluatable
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Dict(HashMap<Ident, Value>),
    Int(isize),
    List(Vec<Value>),
    None,
    Real(f64),
    Str(String),
}

// --- Traits ---

/// Trait allowing various language elements to be evaluated
pub trait Evaluatable {
    fn eval(&self, scopes: &mut ScopeChain) -> Value;
}

/// Trait allowing various language elements to be executed
pub trait Executable {
    fn exec(&self, scopes: &mut ScopeChain) -> ExecResult;
}

/// Trait used to allow structs to be called from a script
///
/// The `execute()` method will be called via the script interpreter with the current ScopeChain
/// and a list of argument values.
pub trait NativeFunction {
    fn execute(&self, scopes: &mut ScopeChain, args: &Vec<Value>) -> Value;
    fn as_any(&self) -> &Any;
}
