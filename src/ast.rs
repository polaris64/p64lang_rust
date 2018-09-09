#[cfg(not(feature = "no_std"))]
use std::any::Any;
#[cfg(feature = "no_std")]
use core::any::Any;

#[cfg(not(feature = "no_std"))]
use std::collections::HashMap;
#[cfg(feature = "no_std")]
use alloc::collections::BTreeMap;

#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use interpreter::ScopeChain;

// --- Types ---

/// Result of executing an Executable
#[derive(Clone, Debug, PartialEq)]
pub enum ExecResult<'src> {
    Break,
    Error(&'static str),
    None,
    Return(Value<'src>),
}

/// Language expression
///
/// Numbers, strings, lists, function calls, identifiers and operations thereon. Anything that can
/// be evaluated to a Value.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'src> {
    BinOp(Box<Expr<'src>>, Opcode, Box<Expr<'src>>),
    Bool(bool),
    Dict(Vec<(Ident<'src>, Box<Expr<'src>>)>),
    FuncCall(Ident<'src>, Vec<Box<Expr<'src>>>),
    Id(Ident<'src>),
    Int(isize),
    ListElement(Ident<'src>, Box<Expr<'src>>),
    List(Vec<Box<Expr<'src>>>),
    None,
    Real(f64),
    Str(&'src str),
    UnaryOp(Opcode, Box<Expr<'src>>),
}

/// Script-defined functions
///
/// Contains a list of statements (StmtBlock) that are executed when the Function is called, and a
/// list of argument Idents that will be assigned to actual values during the call.
#[derive(Debug)]
pub struct Function<'src> {
    pub args:  Vec<Ident<'src>>,
    pub stmts: StmtBlock<'src>,
}

/// Language identifier
///
/// Used to represent a variable or function name.
pub type Ident<'src> = &'src str;

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
pub enum Stmt<'src> {
    Break,
    Expr(Expr<'src>),
    FnDef(Ident<'src>, Vec<Ident<'src>>, StmtBlock<'src>),
    If(Expr<'src>, StmtBlock<'src>),
    IfElse(Expr<'src>, StmtBlock<'src>, StmtBlock<'src>),
    Let(Ident<'src>, Expr<'src>),
    ListItemAssignment(Ident<'src>, Expr<'src>, Expr<'src>),
    Loop(StmtBlock<'src>),
    Return(Expr<'src>),
}

/// Statement block
///
/// A block of zero or more Stmts
pub type StmtBlock<'src> = Vec<Stmt<'src>>;

/// Result of evaluating an Evaluatable
#[derive(Clone, Debug, PartialEq)]
pub enum Value<'src> {
    Bool(bool),

    #[cfg(feature = "no_std")]
    Dict(BTreeMap<Ident<'src>, Value<'src>>),

    #[cfg(not(feature = "no_std"))]
    Dict(HashMap<Ident<'src>, Value<'src>>),

    Int(isize),
    List(Vec<Value<'src>>),
    None,
    Real(f64),
    Str(&'src str),
}

// --- Traits ---

/// Trait allowing various language elements to be evaluated
pub trait Evaluatable<'src> {
    fn eval(&self, scopes: &mut ScopeChain<'src>) -> Value<'src>;
}

/// Trait allowing various language elements to be executed
pub trait Executable<'src> {
    fn exec(&self, scopes: &mut ScopeChain<'src>) -> ExecResult<'src>;
}

/// Trait used to allow structs to be called from a script
///
/// The `execute()` method will be called via the script interpreter with the current ScopeChain
/// and a list of argument values.
pub trait NativeFunction {
    fn execute<'src>(&self, scopes: &mut ScopeChain<'src>, args: &Vec<Value<'src>>) -> Value<'src>;
    fn as_any(&self) -> &Any;
}
