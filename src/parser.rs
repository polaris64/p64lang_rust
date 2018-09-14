#[cfg(not(feature = "no_std"))]
use std::ops::Neg;
#[cfg(feature = "no_std")]
use core::ops::Neg;

#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use nom::{alpha, digit, digit0};
use nom::types::CompleteStr;

use ast::{Expr, Ident, Opcode, Stmt, StmtBlock};

/**
 * Takes an optional sign (&str, "+" or "-") and a number and returns the correct signed number
 * according to the sign.
 *
 * Generic over any type implementing std::Ops::Neg (allows the unary "-" operator to be applied).
 * The unary "-" operator returns a Neg::Output, so we specify that T must be bound to the Neg
 * trait where its Output type is also T.
 */
fn signed_number<T: Neg<Output = T>>(sign: Option<CompleteStr>, num: T) -> T {
    match sign {
        None => num,
        Some(c) => match c.0 {
            "-" => -num,
            _ => num,
        },
    }
}

/// Parser for a number's sign: either "+" or "-"
named!(number_sign<CompleteStr, CompleteStr>, alt!(tag!("+") | tag!("-")));

/**
 * Parser for a single real number: optional number_sign followed by a real number (optional
 * integer component, period, decimal digits).
 */
named!(real<CompleteStr, f64>,
    do_parse!(
        sign: opt!(number_sign) >>
        num: map_res!(

            // recognize! returns the consumed output if the inner parser was successful.  So, the
            // entire input parsed by tuple! (e.g. "123.456") should be returned.
            recognize!(

                // Build a resulting tuple such as ("123", ".", "456") for "123.456".
                tuple!(digit0, tag!("."), digit)
            ),

            // The result will be a string like "123.456" as recognize! returned all matching
            // chars, so parse this as an f64.
            |s: CompleteStr| s.0.parse::<f64>()
        ) >>
        ( signed_number(sign, num) )
    )
);

/// Parser for a single integer number: optional number_sign followed by an integer number
named!(int<CompleteStr, isize>,
    do_parse!(
        sign: opt!(number_sign) >>
        num: map_res!(
            digit,
            |s: CompleteStr| s.0.parse::<isize>()
        ) >>
        ( signed_number(sign, num) )
    )
);

/// Returns true if the char is valid for an identifier (not in first position)
fn is_ident_char(c: char) -> bool {
    match c {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => true,
        _ => false
    }
}

/// Parser for a single language identifier (e.g. "name1")
named!(ident<CompleteStr, Ident>,
    map!(
        recognize!(pair!(alt!(alpha | tag!("_")), take_while!(is_ident_char))),
        |s: CompleteStr| s.0
    )
);


// --- Expressions ---

/// Parser for logical (&&, ||, ^) Opcodes
named!(logical_opcode<CompleteStr, Opcode>,
    alt!(
        map!(tag!("&&"), |_| Opcode::LogicalAnd) |
        map!(tag!("||"), |_| Opcode::LogicalOr)  |
        map!(tag!("^"),  |_| Opcode::LogicalXor)
    )
);

/// Parser for relational Opcodes (e.g. <, >=, !=)
named!(relational_opcode<CompleteStr, Opcode>,
    alt!(
        map!(tag!("<="), |_| Opcode::LessThanOrEqual)    |
        map!(tag!(">="), |_| Opcode::GreaterThanOrEqual) |
        map!(tag!("=="), |_| Opcode::Equal)              |
        map!(tag!("!="), |_| Opcode::NotEqual)           |
        map!(tag!("<"),  |_| Opcode::LessThan)           |
        map!(tag!(">"),  |_| Opcode::GreaterThan)
    )
);

/// Parser for "*", "/", "%" Opcodes
named!(product_opcode<CompleteStr, Opcode>,
    alt!(
        map!(tag!("*"), |_| Opcode::Mul) |
        map!(tag!("/"), |_| Opcode::Div) |
        map!(tag!("%"), |_| Opcode::Mod)
    )
);

/// Parser for "+", "-" Opcodes
named!(sum_opcode<CompleteStr, Opcode>,
    alt!(
        map!(tag!("+"), |_| Opcode::Add) |
        map!(tag!("-"), |_| Opcode::Sub)
    )
);

/// Parser for an expression term: parses either an "expr" delimited by parentheses (recursion) or
/// another language value type
named!(term<CompleteStr, Expr>,
    alt!(
        ws!(delimited!(tag!("("), expr, tag!(")"))) |
        ws!(value_expr)
    )
);

/// Parser for logical expressions (e.g. true && false)
named!(logical_expr<CompleteStr, Expr>,
    alt!(
        do_parse!(
            lhs: term >>
            op:  ws!(logical_opcode) >>
            rhs: logical_expr >>
            ( Expr::BinOp(Box::new(lhs), op, Box::new(rhs)) )
        ) |
        term
    )
);

/// Parser for relational expressions (e.g. 1 < 2)
named!(relational_expr<CompleteStr, Expr>,
    alt!(
        do_parse!(
            lhs: logical_expr >>
            op:  ws!(relational_opcode) >>
            rhs: relational_expr >>
            ( Expr::BinOp(Box::new(lhs), op, Box::new(rhs)) )
        ) |
        logical_expr
    )
);

/// Parser for product expressions (e.g. 2 * 3)
named!(product_expr<CompleteStr, Expr>,
    alt!(
        do_parse!(
            lhs: relational_expr >>
            op:  ws!(product_opcode) >>
            rhs: product_expr >>
            ( Expr::BinOp(Box::new(lhs), op, Box::new(rhs)) )
        ) |
        relational_expr
    )
);

/// Parser for sum expressions (e.g. 1 + 2)
named!(sum_expr<CompleteStr, Expr>,
    alt!(
        do_parse!(
            lhs: product_expr >>
            op:  ws!(sum_opcode) >>
            rhs: sum_expr >>
            ( Expr::BinOp(Box::new(lhs), op, Box::new(rhs)) )
        ) |
        product_expr
    )
);

/// Parser for any language expression
named!(expr<CompleteStr, Expr>,
    call!(sum_expr)
);

/// Parser for Boolean literals
named!(bool_literal<CompleteStr, bool>,
    alt!(
        tag_no_case!("true") => { |_| true } |
        tag_no_case!("false") => { |_| false }
    )
);

/// Parser for Dict literals
named!(dict_literal<CompleteStr, Expr>,
    map!(
        delimited!(
            ws!(tag!("{")),
            separated_list!(ws!(tag!(",")), map!(key_val_pair, |(k, v)| (k, Box::new(v)))),
            ws!(tag!("}"))
        ),
        Expr::Dict
    )
);

/// Parser for float literals (calls real)
named!(float_literal<CompleteStr, f64>, call!(real));

/// Parser for function call expressions
named!(func_call<CompleteStr, Expr>,
    do_parse!(
        id: ident >>
        args: delimited!(
            ws!(tag!("(")),
            separated_list!(ws!(tag!(",")), map!(expr, Box::new)),
            ws!(tag!(")"))
        ) >>
        ( Expr::FuncCall(id, args) )
    )
);

/// Parser for int literals
named!(int_literal<CompleteStr, isize>,
   call!(int)
);

/// Parser for a key (string) / value (expr) pair
named!(key_val_pair<CompleteStr, (Ident, Expr)>,
    do_parse!(
        key: str_literal >>
        ws!(tag!(":")) >>
        val: expr >>
        (key, val)
    )
);

/// Parser for list elements: list identifier and index
named!(list_element<CompleteStr, Expr>,
    do_parse!(
        id: ident >>
        idx: delimited!(
            ws!(tag!("[")),
            map!(expr, Box::new),
            ws!(tag!("]"))
        ) >>
        ( Expr::ListElement(id, idx) )
    )
);

/// Parser for a List literal
named!(list_literal<CompleteStr, Expr>,
    map!(
        delimited!(
            ws!(tag!("[")),
            separated_list!(ws!(tag!(",")), map!(expr, Box::new)),
            ws!(tag!("]"))
        ),
        Expr::List
    )
);

/// Parser for string literals (characters enclosed by '"' characters)
named!(str_literal<CompleteStr, &str>,
    alt!(
        map!(
            delimited!(char!('"'), is_not!("\""), char!('"')),
            |x: CompleteStr| x.0
        ) |
        map!(tag!(r#""""#), |_| "")
    )
);

/// Parser for a unary Opcode (e.g. "!")
named!(unary_opcode<CompleteStr, Opcode>,
    alt!(
        tag!("!") => { |_| Opcode::Not }
    )
);

/// Parser for any unary operation (e.g. "!true")
named!(unary_op<CompleteStr, Expr>,
    do_parse!(
        op: unary_opcode >>
        t:  expr >>
        ( Expr::UnaryOp(op, Box::new(t)) )
    )
);

/// Parser for any language expression that results in a single value
named!(value_expr<CompleteStr, Expr>,
    alt!(
        map!(float_literal,     Expr::Real) |
        map!(int_literal,       Expr::Int)  |
        map!(bool_literal,      Expr::Bool) |
        map!(str_literal,       Expr::Str)  |
        map!(tag!("null"),  |_| Expr::None) |
        func_call                           |
        dict_literal                        |
        list_literal                        |
        list_element                        |
        unary_op                            |
        map!(ident,             Expr::Id)
    )
);


// --- Statements ---

named!(break_statement<CompleteStr, Stmt>,
    map!(ws!(tag!("break")), |_| Stmt::Break)
);

named!(expr_statement<CompleteStr, Stmt>,
    map!(expr, Stmt::Expr)
);

named!(fndef_statement<CompleteStr, Stmt>,
    do_parse!(
        ws!(tag!("fn")) >>
        id: ident >>
        args: delimited!(
            ws!(tag!("(")),
            separated_list!(ws!(tag!(",")), ident),
            ws!(tag!(")"))
        ) >>
        stmts: statement_block >>
        ( Stmt::FnDef(id, args, stmts) )
    )
);

named!(if_statement<CompleteStr, Stmt>,
    do_parse!(
        ws!(tag!("if")) >>
        cond: expr >>
        stmts: statement_block >>
        ( Stmt::If(cond, stmts) )
    )
);

named!(if_else_statement<CompleteStr, Stmt>,
    do_parse!(
        ws!(tag!("if")) >>
        cond: expr >>
        stmts_t: statement_block >>
        ws!(tag!("else")) >>
        stmts_f: statement_block >>
        ( Stmt::IfElse(cond, stmts_t, stmts_f) )
    )
);

named!(let_statement<CompleteStr, Stmt>,
    do_parse!(
        ws!(tag!("let")) >>
        id: ident >>
        ws!(tag!("=")) >> 
        val: ws!(expr) >>
        ( Stmt::Let(id, val) )
    )
);

named!(list_assignment_statement<CompleteStr, Stmt>,
    do_parse!(
        id: ident >>
        idx: delimited!(ws!(tag!("[")), expr, ws!(tag!("]"))) >>
        ws!(tag!("=")) >>
        val: ws!(expr) >>
        ( Stmt::ListItemAssignment(id, idx, val) )
    )
);

named!(loop_statement<CompleteStr, Stmt>,
    do_parse!(
        ws!(tag!("loop")) >>
        stmts: statement_block >>
        ( Stmt::Loop(stmts) )
    )
);

named!(return_statement<CompleteStr, Stmt>,
    do_parse!(
        ws!(tag!("return")) >>
        val: ws!(expr) >>
        ( Stmt::Return(val) )
    )
);

/// Parser for a single supported statement of any type
named!(statement<CompleteStr, Stmt>,
    alt!(
        break_statement           |
        fndef_statement           |
        if_else_statement         |
        if_statement              |
        let_statement             |
        list_assignment_statement |
        loop_statement            |
        return_statement          |
        expr_statement
    )
);

/// Parser for a list of "statement" separated by ";" with an optional trailing ";"
named!(statements<CompleteStr, Vec<Stmt>>,
    do_parse!(
        list: separated_list!(ws!(tag!(";")), statement) >>
        opt!(tag!(";")) >>
        ( list )
    )
);

/// Parser for "statements" enclosed within braces
named!(statement_block<CompleteStr, StmtBlock>,
    delimited!(ws!(tag!("{")), statements, ws!(tag!("}")))
);


/// Axiom rule: parses an entire program
named!(program_parser<CompleteStr, StmtBlock>,
    call!(statements)
);

/**
 * Main parser function: takes source code and returns a Result containing either the AST or a
 * string error.
 */
pub fn parse<'s>(source: &'s str) -> Result<StmtBlock, &'static str> {
    // TODO: obtain error from Nom
    match program_parser(CompleteStr(source)) {
        Ok((_, stmts)) => Ok(stmts),
        Err(_) => Err("Unable to parse source"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_sign_test_valid() {
        assert_eq!(Ok((CompleteStr(""), CompleteStr("+"))), number_sign(CompleteStr("+")));
        assert_eq!(Ok((CompleteStr(""), CompleteStr("-"))), number_sign(CompleteStr("-")));
    }

    #[test]
    #[should_panic]
    fn number_sign_test_invalid() {
        number_sign(CompleteStr("*")).unwrap();
    }

    #[test]
    fn real_test_valid() {
        assert_eq!(Ok((CompleteStr(""),  123f64)),    real(CompleteStr("123.0")));
        assert_eq!(Ok((CompleteStr(""),  123.45f64)), real(CompleteStr("123.45")));
        assert_eq!(Ok((CompleteStr(""),  123f64)),    real(CompleteStr("+123.0")));
        assert_eq!(Ok((CompleteStr(""), -123f64)),    real(CompleteStr("-123.0")));
        assert_eq!(Ok((CompleteStr(""),    0.5f64)),  real(CompleteStr(".5")));
        assert_eq!(Ok((CompleteStr(""),    0.5f64)),  real(CompleteStr("+.5")));
        assert_eq!(Ok((CompleteStr(""), -  0.5f64)),  real(CompleteStr("-.5")));
    }

    #[test]
    #[should_panic]
    fn real_test_invalid() {
        real(CompleteStr("123")).unwrap();
    }

    #[test]
    fn int_test_valid() {
        assert_eq!(Ok((CompleteStr(""),  123)), int(CompleteStr("123")));
        assert_eq!(Ok((CompleteStr(""),  123)), int(CompleteStr("+123")));
        assert_eq!(Ok((CompleteStr(""), -123)), int(CompleteStr("-123")));

        assert_eq!(Ok((CompleteStr(".45"), 123)), int(CompleteStr("123.45")));
    }

    #[test]
    fn ident_test_valid() {
        assert_eq!(Ok((CompleteStr(""),   "abc123")), ident(CompleteStr("abc123")));
        assert_eq!(Ok((CompleteStr(""),   "a")),      ident(CompleteStr("a")));
        assert_eq!(Ok((CompleteStr(""),   "aa")),     ident(CompleteStr("aa")));
        assert_eq!(Ok((CompleteStr(" a"), "a")),      ident(CompleteStr("a a")));
    }

    #[test]
    #[should_panic]
    fn ident_test_invalid() {
        ident(CompleteStr("123abc")).unwrap();
    }

    #[test]
    fn logical_opcode_test_valid() {
        assert_eq!(Ok((CompleteStr(""), Opcode::LogicalAnd)), logical_opcode(CompleteStr("&&")));
        assert_eq!(Ok((CompleteStr(""), Opcode::LogicalOr)),  logical_opcode(CompleteStr("||")));
        assert_eq!(Ok((CompleteStr(""), Opcode::LogicalXor)), logical_opcode(CompleteStr("^")));
    }

    #[test]
    fn product_opcode_test_valid() {
        assert_eq!(Ok((CompleteStr(""), Opcode::Div)), product_opcode(CompleteStr("/")));
        assert_eq!(Ok((CompleteStr(""), Opcode::Mul)), product_opcode(CompleteStr("*")));
        assert_eq!(Ok((CompleteStr(""), Opcode::Mod)), product_opcode(CompleteStr("%")));
    }

    #[test]
    #[should_panic]
    fn product_opcode_test_invalid() {
        product_opcode(CompleteStr("+")).unwrap();
        product_opcode(CompleteStr("-")).unwrap();
    }

    #[test]
    fn relational_opcode_test_valid() {
        assert_eq!(Ok((CompleteStr(""), Opcode::LessThan)),           relational_opcode(CompleteStr("<")));
        assert_eq!(Ok((CompleteStr(""), Opcode::GreaterThan)),        relational_opcode(CompleteStr(">")));
        assert_eq!(Ok((CompleteStr(""), Opcode::LessThanOrEqual)),    relational_opcode(CompleteStr("<=")));
        assert_eq!(Ok((CompleteStr(""), Opcode::GreaterThanOrEqual)), relational_opcode(CompleteStr(">=")));
        assert_eq!(Ok((CompleteStr(""), Opcode::Equal)),              relational_opcode(CompleteStr("==")));
        assert_eq!(Ok((CompleteStr(""), Opcode::NotEqual)),           relational_opcode(CompleteStr("!=")));
    }

    #[test]
    fn sum_opcode_test_valid() {
        assert_eq!(Ok((CompleteStr(""), Opcode::Add)), sum_opcode(CompleteStr("+")));
        assert_eq!(Ok((CompleteStr(""), Opcode::Sub)), sum_opcode(CompleteStr("-")));
    }

    #[test]
    #[should_panic]
    fn sum_opcode_test_invalid() {
        sum_opcode(CompleteStr("*")).unwrap();
        sum_opcode(CompleteStr("/")).unwrap();
    }

    #[test]
    fn logical_expr_test_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LogicalAnd, Box::new(Expr::Int(2))))),
            logical_expr(CompleteStr("1 && 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LogicalOr, Box::new(Expr::Int(2))))),
            logical_expr(CompleteStr("1 || 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LogicalXor, Box::new(Expr::Int(2))))),
            logical_expr(CompleteStr("1 ^ 2"))
        );
    }

    #[test]
    fn relational_expr_test_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LessThan, Box::new(Expr::Int(2))))),
            relational_expr(CompleteStr("1 < 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::GreaterThan, Box::new(Expr::Int(2))))),
            relational_expr(CompleteStr("1 > 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LessThanOrEqual, Box::new(Expr::Int(2))))),
            relational_expr(CompleteStr("1 <= 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::GreaterThanOrEqual, Box::new(Expr::Int(2))))),
            relational_expr(CompleteStr("1 >= 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Equal, Box::new(Expr::Int(2))))),
            relational_expr(CompleteStr("1 == 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::NotEqual, Box::new(Expr::Int(2))))),
            relational_expr(CompleteStr("1 != 2"))
        );
    }

    #[test]
    fn product_expr_test_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Mul, Box::new(Expr::Int(2))))),
            product_expr(CompleteStr("1*2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Mul, Box::new(Expr::Int(2))))),
            product_expr(CompleteStr("1 *2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Mul, Box::new(Expr::Int(2))))),
            product_expr(CompleteStr("1* 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Mul, Box::new(Expr::Int(2))))),
            product_expr(CompleteStr("1 * 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Real(1.23f64)), Opcode::Div, Box::new(Expr::Int(2))))),
            product_expr(CompleteStr("1.23 / 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Mod, Box::new(Expr::Int(2))))),
            product_expr(CompleteStr("1 % 2"))
        );
    }

    #[test]
    fn sum_expr_test_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2))))),
            sum_expr(CompleteStr("1+2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2))))),
            sum_expr(CompleteStr("1 +2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2))))),
            sum_expr(CompleteStr("1+ 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2))))),
            sum_expr(CompleteStr("1 + 2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Real(1.23f64)), Opcode::Sub, Box::new(Expr::Int(2))))),
            sum_expr(CompleteStr("1.23 - 2"))
        );
    }

    #[test]
    fn expr_valid() {
        assert_eq!(Ok((CompleteStr(""), Expr::Real(1.23f64))), expr(CompleteStr("1.23")));
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2))))),
            expr(CompleteStr("1+2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Mul, Box::new(Expr::Int(2))))),
            expr(CompleteStr("1*2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LessThan, Box::new(Expr::Int(2))))),
            expr(CompleteStr("1<2"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::LogicalOr, Box::new(Expr::Int(2))))),
            expr(CompleteStr("1 || 2"))
        );
        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::BinOp(
                    Box::new(Expr::Real(1.23f64)),
                    Opcode::Add,
                    Box::new(Expr::BinOp(
                        Box::new(Expr::Real(2.34f64)),
                        Opcode::Mul,
                        Box::new(Expr::Real(3.45f64)),
                    ))
                )
            )),
            expr(CompleteStr("1.23 + 2.34 * 3.45"))
        );
    }

    #[test]
    fn term_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2))))),
            term(CompleteStr("(1+2)"))
        );
        assert_eq!(Ok((CompleteStr(""), Expr::Real(1.23f64))), term(CompleteStr("1.23")));
    }

    #[test]
    fn bool_literal_valid() {
        assert_eq!(Ok((CompleteStr(""), true)),  bool_literal(CompleteStr("true")));
        assert_eq!(Ok((CompleteStr(""), true)),  bool_literal(CompleteStr("True")));
        assert_eq!(Ok((CompleteStr(""), true)),  bool_literal(CompleteStr("TRUE")));
        assert_eq!(Ok((CompleteStr(""), false)), bool_literal(CompleteStr("false")));
        assert_eq!(Ok((CompleteStr(""), false)), bool_literal(CompleteStr("False")));
        assert_eq!(Ok((CompleteStr(""), false)), bool_literal(CompleteStr("FALSE")));
    }

    #[test]
    fn dict_literal_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::Dict(vec![
                   ("a",   Box::new(Expr::Int(1))),
                   ("bcd", Box::new(Expr::Real(23.45f64)))
                ])
            )),
            dict_literal(CompleteStr(r#"{"a":1,"bcd":23.45}"#))
        );
    }

    #[test]
    fn float_literal_valid() {
        assert_eq!(Ok((CompleteStr(""), 12.34f64)), float_literal(CompleteStr("12.34")));
    }

    #[test]
    fn func_call_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::FuncCall(
                    "testFun",
                    vec![
                        Box::new(Expr::Int(1)),
                        Box::new(Expr::Int(2)),
                        Box::new(Expr::Int(3)),
                    ]
                )
            )),
            func_call(CompleteStr("testFun(1, 2, 3)"))
        );
    }
    
    #[test]
    fn int_literal_valid() {
        assert_eq!(Ok((CompleteStr(""), 123)),  int_literal(CompleteStr("123")));
        assert_eq!(Ok((CompleteStr(""), 123)),  int_literal(CompleteStr("+123")));
        assert_eq!(Ok((CompleteStr(""), -123)), int_literal(CompleteStr("-123")));
    }

    #[test]
    fn key_val_pair_valid() {
        assert_eq!(Ok((CompleteStr(""), ("a", Expr::Int(1)))), key_val_pair(CompleteStr(r#""a":1"#)));
        assert_eq!(Ok((CompleteStr(""), ("a", Expr::Int(1)))), key_val_pair(CompleteStr(r#""a" :1"#)));
        assert_eq!(Ok((CompleteStr(""), ("a", Expr::Int(1)))), key_val_pair(CompleteStr(r#""a": 1"#)));
        assert_eq!(Ok((CompleteStr(""), ("a", Expr::Int(1)))), key_val_pair(CompleteStr(r#""a" : 1"#)));

        assert_eq!(
            Ok((CompleteStr(""), ("abc", Expr::Str("def")))),
            key_val_pair(CompleteStr(r#""abc":"def""#))
        );
    }

    #[test]
    fn list_element_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::ListElement("a", Box::new(Expr::Int(1))))),
            list_element(CompleteStr("a[1]"))
        );
    }

    #[test]
    fn list_literal_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::List(vec![
                   Box::new(Expr::Int(1)),
                   Box::new(Expr::Str("two")),
                   Box::new(Expr::Bool(true)),
                   Box::new(Expr::Real(4.56f64)),
                ])
            )),
            list_literal(CompleteStr(r#"[1, "two", true, 4.56]"#))
        );
    }

    #[test]
    fn str_literal_valid() {
        assert_eq!(Ok((CompleteStr(""), "")),        str_literal(CompleteStr(r#""""#)));
        assert_eq!(Ok((CompleteStr(""), "a")),       str_literal(CompleteStr(r#""a""#)));
        assert_eq!(Ok((CompleteStr(""), "abc")),     str_literal(CompleteStr(r#""abc""#)));
        assert_eq!(Ok((CompleteStr(""), "abc 123")), str_literal(CompleteStr(r#""abc 123""#)));
    }

    #[test]
    fn unary_opcode_valid() {
        assert_eq!(Ok((CompleteStr(""), Opcode::Not)), unary_opcode(CompleteStr("!")));
    }

    #[test]
    fn unary_op_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Expr::UnaryOp(Opcode::Not, Box::new(Expr::Id("a"))))),
            unary_op(CompleteStr("!a"))
        );
        assert_eq!(
            Ok((CompleteStr(""), Expr::UnaryOp(Opcode::Not, Box::new(Expr::Bool(true))))),
            unary_op(CompleteStr("!true"))
        );
    }

    #[test]
    fn value_expr_valid() {
        assert_eq!(Ok((CompleteStr(""), Expr::Real(1.23f64))),          value_expr(CompleteStr("1.23")));
        assert_eq!(Ok((CompleteStr(""), Expr::Int(123))),               value_expr(CompleteStr("123")));
        assert_eq!(Ok((CompleteStr(""), Expr::Bool(true))),             value_expr(CompleteStr("true")));
        assert_eq!(Ok((CompleteStr(""), Expr::Str("abc"))), value_expr(CompleteStr(r#""abc""#)));
        assert_eq!(Ok((CompleteStr(""), Expr::None)),                   value_expr(CompleteStr("null")));
        assert_eq!(Ok((CompleteStr(""), Expr::Id("abc"))),  value_expr(CompleteStr("abc")));

        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::List(vec![
                   Box::new(Expr::Int(1)),
                   Box::new(Expr::Str("two")),
                   Box::new(Expr::Bool(true)),
                   Box::new(Expr::Real(4.56f64)),
                ])
            )),
            value_expr(CompleteStr(r#"[1, "two", true, 4.56]"#))
        );

        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::Dict(vec![
                   ("a",   Box::new(Expr::Int(1))),
                   ("bcd", Box::new(Expr::Real(23.45f64)))
                ])
            )),
            value_expr(CompleteStr(r#"{"a":1,"bcd":23.45}"#))
        );

        assert_eq!(
            Ok((
                CompleteStr(""),
                Expr::FuncCall(
                    "testFun",
                    vec![
                        Box::new(Expr::Int(1)),
                        Box::new(Expr::Int(2)),
                        Box::new(Expr::Int(3)),
                    ]
                )
            )),
            value_expr(CompleteStr("testFun(1, 2, 3)"))
        );

        assert_eq!(
            Ok((CompleteStr(""), Expr::ListElement("a", Box::new(Expr::Int(1))))),
            value_expr(CompleteStr("a[1]"))
        );

        assert_eq!(
            Ok((CompleteStr(""), Expr::UnaryOp(Opcode::Not, Box::new(Expr::Id("a"))))),
            value_expr(CompleteStr("!a"))
        );
    }

    #[test]
    fn break_statement_valid() {
        assert_eq!(Ok((CompleteStr(""), Stmt::Break)), break_statement(CompleteStr("break")));
        assert_eq!(Ok((CompleteStr(""), Stmt::Break)), break_statement(CompleteStr(" break")));
        assert_eq!(Ok((CompleteStr(""), Stmt::Break)), break_statement(CompleteStr("break ")));
        assert_eq!(Ok((CompleteStr(""), Stmt::Break)), break_statement(CompleteStr(" break ")));

        assert_eq!(Ok((CompleteStr(";"), Stmt::Break)), break_statement(CompleteStr("break;")));
    }

    #[test]
    fn expr_statement_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Stmt::Expr(Expr::Id("a")))),
            expr_statement(CompleteStr("a"))
        );
    }

    #[test]
    fn fndef_statement_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Stmt::FnDef(
                    "abc",
                    vec![
                        "a",
                        "b",
                        "c",
                    ],
                    vec![
                        Stmt::Return(Expr::Id("a")),
                    ]
                )
            )),
            fndef_statement(CompleteStr("fn abc(a,b,c) { return a;}"))
        );
    }

    #[test]
    fn if_statement_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Stmt::If(
                    Expr::Bool(true),
                    vec![
                        Stmt::Expr(
                            Expr::FuncCall(
                                "print",
                                vec![Box::new(Expr::Int(1))],
                            ),
                        ),
                    ]
                )
            )),
            if_statement(CompleteStr(r#"if true { print(1); }"#))
        );
    }

    #[test]
    fn if_else_statement_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Stmt::IfElse(
                    Expr::Bool(true),
                    vec![
                        Stmt::Expr(
                            Expr::FuncCall(
                                "print",
                                vec![Box::new(Expr::Int(1))],
                            ),
                        ),
                    ],
                    vec![
                        Stmt::Expr(
                            Expr::FuncCall(
                                "print",
                                vec![Box::new(Expr::Int(0))],
                            ),
                        ),
                    ]
                )
            )),
            if_else_statement(CompleteStr(r#"if true { print(1); } else { print(0); }"#))
        );
    }

    #[test]
    fn let_statement_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Stmt::Let("a", Expr::Int(123)))),
            let_statement(CompleteStr("let a = 123"))
        );
    }

    #[test]
    fn list_assignment_statement_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Stmt::ListItemAssignment(
                    "a",
                    Expr::Int(1),
                    Expr::Int(2)
                )
            )),
            list_assignment_statement(CompleteStr("a[1] = 2"))
        );
        assert_eq!(
            Ok((
                CompleteStr(""),
                Stmt::ListItemAssignment(
                    "a",
                    Expr::Str("idx"),
                    Expr::Int(2)
                )
            )),
            list_assignment_statement(CompleteStr(r#"a["idx"] = 2"#))
        );
    }

    #[test]
    fn loop_statement_valid() {
        assert_eq!(
            Ok((
                CompleteStr(""),
                Stmt::Loop(
                    vec![
                        Stmt::Expr(
                            Expr::FuncCall(
                                "print",
                                vec![Box::new(Expr::Int(1))]
                            )
                        ),
                    ]
                )
            )),
            loop_statement(CompleteStr("loop { print(1); }"))
        );
    }

    #[test]
    fn return_statement_valid() {
        assert_eq!(
            Ok((CompleteStr(""), Stmt::Return(Expr::Int(123)))),
            return_statement(CompleteStr("return 123"))
        );
    }

    #[test]
    fn statement_valid() {
        match statement(CompleteStr("break")) {
            Err(_) => assert!(false, "statement(): Break: returned error"),
            Ok(s) => match s.1 {
                Stmt::Break => {},
                _ => assert!(false, "statement(): Break: not Stmt::Break"),
            },
        }
        match statement(CompleteStr("fn a(b) { return a; }")) {
            Err(_) => assert!(false, "statement(): FnDef: returned error"),
            Ok(s) => match s.1 {
                Stmt::FnDef(_, _, _) => {},
                _ => assert!(false, "statement(): FnDef: not Stmt::FnDef"),
            },
        }
        match statement(CompleteStr("if true { print(1); }")) {
            Err(_) => assert!(false, "statement(): If: returned error"),
            Ok(s) => match s.1 {
                Stmt::If(_, _) => {},
                _ => assert!(false, "statement(): If: not Stmt::If"),
            },
        }
        match statement(CompleteStr("if true { print(1); } else { print(0); }")) {
            Err(_) => assert!(false, "statement(): IfElse: returned error"),
            Ok(s) => match s.1 {
                Stmt::IfElse(_, _, _) => {},
                _ => assert!(false, "statement(): IfElse: not Stmt::IfElse"),
            },
        }
        match statement(CompleteStr("let a = 1")) {
            Err(_) => assert!(false, "statement(): Let: returned error"),
            Ok(s) => match s.1 {
                Stmt::Let(_, _) => {},
                _ => assert!(false, "statement(): Let: not Stmt::Let"),
            },
        }
        match statement(CompleteStr("a[1] = 2")) {
            Err(_) => assert!(false, "statement(): ListItemAssignment: returned error"),
            Ok(s) => match s.1 {
                Stmt::ListItemAssignment(_, _, _) => {},
                _ => assert!(false, "statement(): ListItemAssignment: not Stmt::ListItemAssignment"),
            },
        }
        match statement(CompleteStr("loop { print(1); }")) {
            Err(_) => assert!(false, "statement(): Loop: returned error"),
            Ok(s) => match s.1 {
                Stmt::Loop(_) => {},
                _ => assert!(false, "statement(): Loop: not Stmt::Loop"),
            },
        }
        match statement(CompleteStr("return 1")) {
            Err(_) => assert!(false, "statement(): Return: returned error"),
            Ok(s) => match s.1 {
                Stmt::Return(_) => {},
                _ => assert!(false, "statement(): Return: not Stmt::Return"),
            },
        }
        match statement(CompleteStr("print(1)")) {
            Err(_) => assert!(false, "statement(): Expr: returned error"),
            Ok(s) => match s.1 {
                Stmt::Expr(_) => {},
                _ => assert!(false, "statement(): Expr: not Stmt::Expr"),
            },
        }
    }
}
