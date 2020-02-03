#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate nom;

pub mod ast;
pub mod interpreter;
mod parser;
pub mod runtime;

use ast::{ExecResult, Executable};
use interpreter::{Scope, ScopeChain};
use parser::parse;
use runtime::insert_native_functions;

/// Result of parsing and executing code
///
///   - `exec_result`: actual resulting value from execution
///   - `scope_chain`: ScopeChain after execution
pub struct InterpretResult<'src> {
    pub exec_result: ExecResult<'src>,
    pub scope_chain: ScopeChain<'src>,
}

/// Gets a Scope containing the runtime module's default NativeFunctions
pub fn get_default_global_scope<'src>() -> Scope<'src> {
    let mut scope = Scope::new();
    insert_native_functions(&mut scope);
    scope
}

/// Interprets given source code under a Scope
///
/// # Params
///
///   - `src: &str`: source code to parse and execute
///   - `global_scope: Scope`: root scope under which to execute the code
///
pub fn interpret<'src>(src: &'src str, global_scope: Scope<'src>) -> InterpretResult<'src> {
    let mut scopes = ScopeChain::from_scope(global_scope);
    let er = match parse(src) {
        Ok(stmts) => stmts.exec(&mut scopes),
        Err(s)    => ExecResult::Error(s),
    };
    InterpretResult {
        exec_result: er,
        scope_chain: scopes,
    }
}

#[cfg(all(test, not(feature = "no_std")))]
mod tests {
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;

    use ast::{Executable, Expr, Ident, Opcode, NativeFunction, Stmt, Value};
    use interpreter::{Scope, ScopeChain};
    use parser::parse;

    struct TestPrint {
        calls: RefCell<usize>,
    }
    impl TestPrint {
        pub fn get_calls(&self) -> usize {
            self.calls.borrow().clone()
        }
        pub fn assert_calls(&self, num: usize) {
            assert_eq!(num, self.get_calls());
        }
    }
    impl NativeFunction for TestPrint {
        fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, _args: &[Value<'src>]) -> Value<'src> {
            self.calls.replace(self.get_calls() + 1);
            Value::None
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }
    struct TestPrintLn {
        calls: RefCell<usize>,
    }
    impl TestPrintLn {
        pub fn get_calls(&self) -> usize {
            self.calls.borrow().clone()
        }
        pub fn assert_calls(&self, num: usize) {
            assert_eq!(num, self.get_calls());
        }
    }
    impl NativeFunction for TestPrintLn {
        fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, _args: &[Value<'src>]) -> Value<'src> {
            self.calls.replace(self.get_calls() + 1);
            Value::None
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn insert_test_functions(scope: &mut Scope) {
        let test_print = Rc::new(TestPrint {
            calls: RefCell::new(0),
        });
        let test_println = Rc::new(TestPrintLn {
            calls: RefCell::new(0),
        });
        scope.native_funcs.insert("print",   test_print);
        scope.native_funcs.insert("println", test_println);
    }

    #[test]
    fn let_stmt() {

        // Test parsing
        assert_eq!(
            Ok(vec![
                Stmt::Let(
                    "a",
                    Expr::BinOp(Box::new(Expr::Int(1)), Opcode::Add, Box::new(Expr::Int(2)))
                )
            ]),
            parse("let a = 1 + 2;")
        );

        let mut scopes = ScopeChain::from_scope(Scope::new());

        // Test evaluation of expression using an undefined variable
        assert_eq!(
            ExecResult::Return(Value::None),
            parse("return a + 1").unwrap().exec(&mut scopes)
        );

        // Test evaluation of a Let statement
        assert_eq!(None, scopes.resolve_var("a"));
        assert_eq!(
            ExecResult::Return(Value::Int(3)),
            parse("let a = 1 + 2; return a;").unwrap().exec(&mut scopes)
        );
        assert_eq!(Some(&Value::Int(3)), scopes.resolve_var("a"));

        // Test evaluation of expressions using variable "a" (now defined in "scope")
        assert_eq!(
            ExecResult::Return(Value::Int(4)),
            parse("let b = a + 1; return b;").unwrap().exec(&mut scopes)
        );
        assert_eq!(
            ExecResult::Return(Value::Int(9)),
            parse("let b = a * a; return b;").unwrap().exec(&mut scopes)
        );
        assert_eq!(
            ExecResult::Return(Value::Real(1.5f64)),
            parse("let b = a / 2; return b;").unwrap().exec(&mut scopes)
        );
    }

    #[test]
    fn literals() {

        // Bools
        assert_eq!(
            ExecResult::Return(Value::Bool(true)),
            interpret("return true;", Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Bool(false)),
            interpret("return false;", Scope::new()).exec_result
        );

        // Ints
        assert_eq!(
            ExecResult::Return(Value::Int(42)),
            interpret("return 42;", Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Int(-42)),
            interpret("return -42;", Scope::new()).exec_result
        );

        // Reals
        assert_eq!(
            ExecResult::Return(Value::Real(1.618f64)),
            interpret("return 1.618;", Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Real(-1.618f64)),
            interpret("return -1.618;", Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Real(0.618f64)),
            interpret("return .618;", Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Real(-0.618f64)),
            interpret("return -.618;", Scope::new()).exec_result
        );

        // Strings
        assert_eq!(
            ExecResult::Return(Value::Str("Hello")),
            interpret(r#"return "Hello";"#, Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Str("Hello world!")),
            interpret(r#"return "Hello world!";"#, Scope::new()).exec_result
        );
        assert_eq!(
            ExecResult::Return(Value::Str("Hello'world!")),
            interpret(r#"return "Hello'world!";"#, Scope::new()).exec_result
        );
        // TODO: escaped " in Strings
        //assert_eq!("Str(\"Hello\"world!\")", format!("{:?}", ExprParser::new().parse(r#""Hello\"world!""#).unwrap()));

        // Ids
        assert_eq!(
            Ok(vec![Stmt::Expr(Expr::Id("a"))]),
            parse("a")
        );
        assert_eq!(
            Ok(vec![Stmt::Expr(Expr::Id("_a"))]),
            parse("_a")
        );
        assert_eq!(
            Ok(vec![Stmt::Expr(Expr::Id("a123"))]),
            parse("a123")
        );
        assert_eq!(
            Ok(vec![Stmt::Expr(Expr::Id("a123_45"))]),
            parse("a123_45")
        );
    }

    #[test]
    fn operator_precedence() {
        // Test language expression precedence
        // 1 + (2 * 3 / 4) + 42 = 1 + 1.5 + 42 = Real(44.5)
        let scopes = interpret("fn test(b) { return b; }; let a = 1 + 2 * 3 / 4 + test(42);", Scope::new()).scope_chain;
        assert_eq!(Some(&Value::Real(44.5)), scopes.resolve_var("a"));
    }

    #[test]
    fn scope_inheritance() {
        let scopes = interpret("let a = 1; fn test(z) { return a + z; }; let b = test(2); let c = a;", Scope::new()).scope_chain;
        assert_eq!(Some(&Value::Int(1)), scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Int(3)), scopes.resolve_var("b"));
        assert_eq!(Some(&Value::Int(1)), scopes.resolve_var("c"));
    }

    #[test]
    fn lib_interpret() {
        let mut scope = Scope::new();
        insert_test_functions(&mut scope);
        let res = interpret("return 42", scope);
        match res.exec_result {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(x) => assert_eq!(Value::Int(42), x),
            ExecResult::Error(e) => assert!(false, e),
        };
        res.scope_chain
            .resolve_native_func("print")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPrint>()
            .unwrap()
            .assert_calls(0);
        res.scope_chain
            .resolve_native_func("println")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPrintLn>()
            .unwrap()
            .assert_calls(0);

        // Complex function (Fibonacci)
        let src = r#"
            fn fib(n) {
                if n <= 0 { return 0; };
                let count = n;
                let prev  = 0;
                let res   = 1;
                loop {
                    let temp = res;
                    let res = res + prev;
                    let prev = temp;
                    print(res);
                    print(", ");
                    let count = count - 1;
                    if count <= 1 {
                        break;
                    };
                };
                println("");
                return res;
            };

            return fib(8);
        "#;
        let mut scope = Scope::new();
        insert_test_functions(&mut scope);
        let res = interpret(src, scope);
        match res.exec_result {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(x) => assert_eq!(Value::Int(21), x),
            ExecResult::Error(e) => assert!(false, e),
        };

        // print should have been invoked twice per loop (=14)
        // println should have been invoked once (after loop)
        res.scope_chain
            .resolve_native_func("print")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPrint>()
            .unwrap()
            .assert_calls(14);
        res.scope_chain
            .resolve_native_func("println")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPrintLn>()
            .unwrap()
            .assert_calls(1);

        // Complex function (recursive factorial)
        let src = r#"
            fn fact(n) {
                if n <= 0 {
                    return 0;
                };
                if n == 1 {
                    return 1;
                } else {
                    return n * fact(n - 1);
                };
            };

            return fact(4);
        "#;
        let mut scope = Scope::new();
        insert_test_functions(&mut scope);
        let res = interpret(src, scope);
        match res.exec_result {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(x) => assert_eq!(Value::Int(24), x),
            ExecResult::Error(e) => assert!(false, e),
        };
        res.scope_chain
            .resolve_native_func("print")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPrint>()
            .unwrap()
            .assert_calls(0);
        res.scope_chain
            .resolve_native_func("println")
            .unwrap()
            .as_any()
            .downcast_ref::<TestPrintLn>()
            .unwrap()
            .assert_calls(0);
    }

    #[test]
    fn bin_ops() {

        // Test evaluation of relational expressions
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return 1 == 1;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return 1 != 1;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return 1 == 2;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return 1  > 2;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return 2  > 1;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return 1  < 2;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return 2  < 1;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return 2 >= 2;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return 2 >= 3;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return 2 <= 2;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return 2 <= 3;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret(r#"return "a" == "a";"#, Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret(r#"return "a" == "b";"#, Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return (1 + 3) > 3", Scope::new()).exec_result);

        // Test evaluation of arithmetic expressions

        // +
        assert_eq!(ExecResult::Return(Value::Int(3)),       interpret("return 1   + 2;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(3.3f64)), interpret("return 1   + 2.3;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(3.2f64)), interpret("return 1.2 + 2;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(3.5f64)), interpret("return 1.2 + 2.3;", Scope::new()).exec_result);

        // -
        assert_eq!(ExecResult::Return(Value::Int(-1)),       interpret("return 1   - 2;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(-1.5f64)), interpret("return 1   - 2.5;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(-0.8f64)), interpret("return 1.2 - 2;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(-1.3f64)), interpret("return 1.2 - 2.5;", Scope::new()).exec_result);

        // *
        assert_eq!(ExecResult::Return(Value::Int(6)),        interpret("return 2   * 3;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(6.8f64)),  interpret("return 2   * 3.4;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(7.5f64)),  interpret("return 2.5 * 3;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(3.75f64)), interpret("return 2.5 * 1.5;", Scope::new()).exec_result);

        // /
        assert_eq!(ExecResult::Return(Value::Real(3f64)),    interpret("return 6   / 2;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(3.35f64)), interpret("return 6.7 / 2;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(2.4f64)),  interpret("return 6   / 2.5;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Real(2.68f64)), interpret("return 6.7 / 2.5;", Scope::new()).exec_result);

        // %
        assert_eq!(ExecResult::Return(Value::Int(4)), interpret("return 16   % 6;",    Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::None),   interpret("return 16   % 12.1;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::None),   interpret("return 16.1 % 12;",   Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::None),   interpret("return 16.1 % 12.1;", Scope::new()).exec_result);
    }

    #[test]
    fn logical_truth_tables() {
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return true  && true;",  Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return true  && false;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return false && true;",  Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return false && false;", Scope::new()).exec_result);

        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return true  || true;",  Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return true  || false;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return false || true;",  Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return false || false;", Scope::new()).exec_result);

        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return true  ^ true;",  Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return true  ^ false;", Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(true)),  interpret("return false ^ true;",  Scope::new()).exec_result);
        assert_eq!(ExecResult::Return(Value::Bool(false)), interpret("return false ^ false;", Scope::new()).exec_result);
    }

    #[test]
    fn stmt_block() {
        // Test evaluation of a full StmtBlock with a new Scope
        let scopes = interpret("let abc = 1 + 2; let bcd = 3 + 4; let cde = abc * bcd;", Scope::new()).scope_chain;
        assert_eq!(Some(&Value::Int(3)),  scopes.resolve_var("abc"));
        assert_eq!(Some(&Value::Int(7)),  scopes.resolve_var("bcd"));
        assert_eq!(Some(&Value::Int(21)), scopes.resolve_var("cde"));
    }

    #[test]
    fn functions() {
        // Test function definitions and calls
        let scopes = interpret(
            "fn add(a, b) { let c = a + b; return c; let c = 123; }; let res = add(1, 2 + 3);",
            Scope::new()
        ).scope_chain;
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("res"));

        // Functions without arguments
        let scopes = interpret(
            "fn test() { return 42; }; let res = test();",
            Scope::new()
        ).scope_chain;
        assert_eq!(Some(&Value::Int(42)), scopes.resolve_var("res"));
    }

    #[test]
    fn conditionals() {
        // Test conditional If/IfElse statements
        let mut scopes = interpret(
            "let a = 1; if 1 == 1 { let a = 2; } else { let a = 3; }; if 1 != 2 { let a = 4; }",
            Scope::new()
        ).scope_chain;
        assert_eq!(Some(&Value::Int(4)), scopes.resolve_var("a"));
        let mut scopes = interpret("if (1 == 2) || (1 == 1) { let a = 5; };", scopes.pop().unwrap()).scope_chain;
        assert_eq!(Some(&Value::Int(5)), scopes.resolve_var("a"));
        let mut scopes = interpret("if (1 == 1) && (2 == 2) { let a = 6; };", scopes.pop().unwrap()).scope_chain;
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
        let mut scopes = interpret("if (1 == 1) ^ (2 == 2) { let a = 7; };", scopes.pop().unwrap()).scope_chain;
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
        let scopes = interpret("if 1 == 1 ^ 2 == 2 { let a = 8; };", scopes.pop().unwrap()).scope_chain;
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
    }

    #[test]
    fn loops() {
        // Test loop
        let scopes = interpret(
            "let a = 0; let b = 1; loop { let a = a + 1; let b = b * 2; if a > 5 { break; }; };",
            Scope::new()
        ).scope_chain;
        assert_eq!(Some(&Value::Int(6)),  scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Int(64)), scopes.resolve_var("b"));
    }

    #[test]
    fn unary_ops() {
        // Test unary operators
        let scopes = interpret("let a = !(1 == 1); let b = !(2 < 1);", Scope::new()).scope_chain;
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Bool(true)), scopes.resolve_var("b"));

        // Test unary operators and Boolean literals
        let scopes = interpret("let a = true; let b = false; let c = !a; let d = !a && !b;", Scope::new()).scope_chain;
        assert_eq!(Some(&Value::Bool(true)),  scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("b"));
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("c"));
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("d"));
    }

    #[test]
    fn native_functions() {
        struct TestFunc {};
        impl NativeFunction for TestFunc {
            fn execute<'src>(&self, _scopes: &mut ScopeChain<'src>, args: &[Value<'src>]) -> Value<'src> {
                match args[0] {
                    Value::Int(x) => Value::Int(x + 40),
                    _ => Value::None,
                }
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        };
        let test_func = TestFunc {};
        let mut scope = Scope::new();
        scope
            .native_funcs
            .insert("test_func", Rc::new(test_func));

        let scopes = interpret("let a = test_func(1) + 1; let b = test_func(12) * 3;", scope).scope_chain;
        assert_eq!(Some(&Value::Int(42)),  scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Int(156)), scopes.resolve_var("b"));
    }

    #[test]
    fn lists() {
        let scopes = interpret("let a = [1, \"test\", 2]; let b = a[1];", Scope::new()).scope_chain;
        assert_eq!(
            Some(&Value::List(vec![
                Value::Int(1),
                Value::Str("test"),
                Value::Int(2)
            ])),
            scopes.resolve_var("a")
        );
        assert_eq!(
            Some(&Value::Str("test")),
            scopes.resolve_var("b")
        );

        let scopes = interpret(
            "let a = [1, \"test\", 2]; a[0] = 40 + 2; a[4] = \"test2\"; let b = a[0]; let c = a[3]; let d = a[4];",
            Scope::new()
        ).scope_chain;
        assert_eq!(
            Some(&Value::List(vec![
                Value::Int(42),
                Value::Str("test"),
                Value::Int(2),
                Value::None,
                Value::Str("test2"),
            ])),
            scopes.resolve_var("a")
        );
        assert_eq!(Some(&Value::Int(42)), scopes.resolve_var("b"));
        assert_eq!(Some(&Value::None),    scopes.resolve_var("c"));
        assert_eq!(
            Some(&Value::Str("test2")),
            scopes.resolve_var("d")
        );
    }

    #[test]
    fn dicts() {
        let scopes = interpret(
            "let a = {\"d1\": 1 + 2, \"d2\": \"second\"}; let b = a[\"d1\"]; a[\"d2\"] = \"third\"; a[\"d3\"] = \"fourth\";",
            Scope::new()
        ).scope_chain;
        let mut expected = HashMap::<Ident, Value>::new();
        expected.insert("d1", Value::Int(3));
        expected.insert("d2", Value::Str("third"));
        expected.insert("d3", Value::Str("fourth"));
        assert_eq!(&Value::Dict(expected), scopes.resolve_var("a").unwrap());
        assert_eq!(Some(&Value::Int(3)),   scopes.resolve_var("b"));
    }
}
