pub mod ast;
mod p64lang;
pub mod runtime;

use ast::{ExecResult, Executable, Scope, ScopeChain};
use p64lang::ProgramParser;
use runtime::ScriptInterface;

pub fn interpret(src: &str, global_scope: Scope, iface: &mut impl ScriptInterface) -> ExecResult {
    let mut scopes = ScopeChain::new();
    scopes.push(global_scope);
    match ProgramParser::new().parse(src) {
        Ok(block) => block.exec(&mut scopes, iface),
        Err(_) => ExecResult::Error("Unable to parse program source"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::{Evaluatable, Executable, Scope, ScopeChain, Value};
    use p64lang::{ExprParser, ProgramParser};

    // Mock ScriptInterface for testing
    struct TestingScriptInterface {
        print_count: usize,
        println_count: usize,
    }
    impl TestingScriptInterface {
        pub fn new() -> TestingScriptInterface {
            TestingScriptInterface {
                print_count: 0,
                println_count: 0,
            }
        }
    }
    impl ScriptInterface for TestingScriptInterface {
        fn print(&mut self, s: &str) {
            self.print_count += 1;
            print!("{}", s);
        }
        fn println(&mut self, s: &str) {
            self.println_count += 1;
            println!("{}", s);
        }
    }

    #[test]
    fn let_stmt() {
        let mut scopes = ScopeChain::from_scope(Scope::new());

        // Test parsing
        assert_eq!(
            "[Let(\"a\", BinOp(Int(1), Add, Int(2)))]",
            format!("{:?}", ProgramParser::new().parse("let a = 1 + 2").unwrap())
        );

        // Test evaluation of expression using an undefined variable
        assert_eq!(
            "None",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("a + 1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );

        // Test evaluation of a Let statement
        assert_eq!(None, scopes.resolve_var("a"));
        assert_eq!(
            "None",
            format!(
                "{:?}",
                ProgramParser::new()
                    .parse("let a = 1 + 2")
                    .unwrap()
                    .exec_default(&mut scopes)
            )
        );
        assert_eq!(Some(&Value::Int(3)), scopes.resolve_var("a"));

        // Test evaluation of expressions using variable "a" (now defined in "scope")
        assert_eq!(
            "Int(4)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("a + 1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Int(9)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("a * a")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(1.5)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("a / 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
    }

    #[test]
    fn literals() {
        // Bools
        assert_eq!(
            "Bool(true)",
            format!("{:?}", ExprParser::new().parse("true").unwrap())
        );
        assert_eq!(
            "Bool(false)",
            format!("{:?}", ExprParser::new().parse("false").unwrap())
        );

        // Ints
        assert_eq!(
            "Int(42)",
            format!("{:?}", ExprParser::new().parse("42").unwrap())
        );
        assert_eq!(
            "Int(-42)",
            format!("{:?}", ExprParser::new().parse("-42").unwrap())
        );

        // Reals
        assert_eq!(
            "Real(1.618)",
            format!("{:?}", ExprParser::new().parse("1.618").unwrap())
        );
        assert_eq!(
            "Real(-1.618)",
            format!("{:?}", ExprParser::new().parse("-1.618").unwrap())
        );
        assert_eq!(
            "Real(0.618)",
            format!("{:?}", ExprParser::new().parse(".618").unwrap())
        );
        assert_eq!(
            "Real(-0.618)",
            format!("{:?}", ExprParser::new().parse("-.618").unwrap())
        );

        // Strings
        assert_eq!(
            "Str(\"Hello\")",
            format!("{:?}", ExprParser::new().parse(r#""Hello""#).unwrap())
        );
        assert_eq!(
            "Str(\"Hello world!\")",
            format!(
                "{:?}",
                ExprParser::new().parse(r#""Hello world!""#).unwrap()
            )
        );
        assert_eq!(
            "Str(\"Hello\\'world!\")",
            format!(
                "{:?}",
                ExprParser::new().parse(r#""Hello'world!""#).unwrap()
            )
        );
        // TODO: escaped " in Strings
        //assert_eq!("Str(\"Hello\"world!\")", format!("{:?}", ExprParser::new().parse(r#""Hello\"world!""#).unwrap()));

        // Ids
        assert_eq!(
            "Id(\"a\")",
            format!("{:?}", ExprParser::new().parse("a").unwrap())
        );
        assert_eq!(
            "Id(\"_a\")",
            format!("{:?}", ExprParser::new().parse("_a").unwrap())
        );
        assert_eq!(
            "Id(\"a123\")",
            format!("{:?}", ExprParser::new().parse("a123").unwrap())
        );
        assert_eq!(
            "Id(\"a123_45\")",
            format!("{:?}", ExprParser::new().parse("a123_45").unwrap())
        );
    }

    #[test]
    fn operator_precedence() {
        // Test language expression precedence
        // 1 + (2 * 3 / 4) + 42 = 1 + 1.5 + 42 = Real(44.5)
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse("fn test(b) { return b; }; let a = 1 + 2 * 3 / 4 + test(42);")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Real(44.5)), scopes.resolve_var("a"));
    }

    #[test]
    fn scope_inheritance() {
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse("let a = 1; fn test(z) { return a + z; }; let b = test(2); let c = a;")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(1)), scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Int(3)), scopes.resolve_var("b"));
        assert_eq!(Some(&Value::Int(1)), scopes.resolve_var("c"));
    }

    #[test]
    fn bin_ops() {
        // Test evaluation of relational expressions
        let mut scopes = ScopeChain::from_scope(Scope::new());
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 == 1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 != 1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 == 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 > 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 > 1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 < 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 < 1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 >= 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 >= 3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 <= 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 <= 3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("\"a\" == \"a\"")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("\"a\" == \"b\"")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("(1 + 3) > 3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );

        // Test evaluation of arithmetic expressions
        let mut scopes = ScopeChain::from_scope(Scope::new());

        // +
        assert_eq!(
            "Int(3)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 + 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(3.3)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 + 2.3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(3.2)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1.2 + 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(3.5)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1.2 + 2.3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );

        // -
        assert_eq!(
            "Int(-1)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 - 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(-1.5)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1 - 2.5")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(-0.8)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1.2 - 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(-1.3)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("1.2 - 2.5")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );

        // *
        assert_eq!(
            "Int(6)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 * 3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(6.8)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2 * 3.4")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(7.5)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2.5 * 3")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(3.75)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("2.5 * 1.5")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );

        // /
        assert_eq!(
            "Real(3.0)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("6 / 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(3.35)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("6.7 / 2")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(2.4)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("6 / 2.5")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Real(2.68)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("6.7 / 2.5")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );

        // %
        assert_eq!(
            "Int(4)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("16 % 12")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "None",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("16 % 12.1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "None",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("16.1 % 12")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "None",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("16.1 % 12.1")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
    }

    #[test]
    fn logical_truth_tables() {
        // Assert logical truth tables
        let mut scopes = ScopeChain::from_scope(Scope::new());
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("true  && true")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("true  && false")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("false && true")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("false && false")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("true  || true")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("true  || false")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("false || true")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("false || false")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("true  ^  true")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("true  ^  false")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(true)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("false ^  true")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
        assert_eq!(
            "Bool(false)",
            format!(
                "{:?}",
                ExprParser::new()
                    .parse("false ^  false")
                    .unwrap()
                    .eval_default(&mut scopes)
            )
        );
    }

    #[test]
    fn stmt_block() {
        // Test evaluation of a full StmtBlock with a new Scope
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse("let abc = 1 + 2; let bcd = 3 + 4; let cde = abc * bcd;")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(3)), scopes.resolve_var("abc"));
        assert_eq!(Some(&Value::Int(7)), scopes.resolve_var("bcd"));
        assert_eq!(Some(&Value::Int(21)), scopes.resolve_var("cde"));
    }

    #[test]
    fn functions() {
        // Test function definitions and calls
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse(
                "fn add(a, b) { let c = a + b; return c; let c = 123; }; let res = add(1, 2 + 3);",
            ).unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("res"));

        // Functions without arguments
        ProgramParser::new()
            .parse("fn test() { return 42; }; let res = test();")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(42)), scopes.resolve_var("res"));
    }

    #[test]
    fn conditionals() {
        // Test conditional If/IfElse statements
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse(
                "let a = 1; if 1 == 1 { let a = 2; } else { let a = 3; }; if 1 != 2 { let a = 4; }",
            ).unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(4)), scopes.resolve_var("a"));
        ProgramParser::new()
            .parse("if (1 == 2) || (1 == 1) { let a = 5; };")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(5)), scopes.resolve_var("a"));
        ProgramParser::new()
            .parse("if (1 == 1) && (2 == 2) { let a = 6; };")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
        ProgramParser::new()
            .parse("if (1 == 1) ^ (2 == 2) { let a = 7; };")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
        ProgramParser::new()
            .parse("if 1 == 1 ^ 2 == 2 { let a = 8; };")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
    }

    #[test]
    fn loops() {
        // Test loop
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse("let a = 0; let b = 1; loop { let a = a + 1; let b = b * 2; if a > 5 { break; }; };")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Int(6)), scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Int(64)), scopes.resolve_var("b"));
    }

    #[test]
    fn unary_ops() {
        // Test unary operators
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse("let a = !(1 == 1); let b = !(2 < 1);")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Bool(true)), scopes.resolve_var("b"));

        // Test unary operators and Boolean literals
        let mut scopes = ScopeChain::from_scope(Scope::new());
        ProgramParser::new()
            .parse("let a = true; let b = false; let c = !a; let d = !a && !b;")
            .unwrap()
            .exec_default(&mut scopes);
        assert_eq!(Some(&Value::Bool(true)), scopes.resolve_var("a"));
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("b"));
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("c"));
        assert_eq!(Some(&Value::Bool(false)), scopes.resolve_var("d"));
    }

    #[test]
    fn lib_interpret() {
        let mut iface = TestingScriptInterface::new();
        let scope = Scope::new();
        let res = interpret("return 42", scope, &mut iface);
        match res {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(x) => assert_eq!(Value::Int(42), x),
            ExecResult::Error(e) => assert!(false, e),
        };
        assert_eq!(0, iface.print_count);
        assert_eq!(0, iface.println_count);
        let scope = Scope::new();
        let res = interpret("!&*", scope, &mut iface);
        match res {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(_) => assert!(false, "interpret() should not have returned Return"),
            ExecResult::Error(e) => assert_eq!("Unable to parse program source", e),
        };
        assert_eq!(0, iface.print_count);
        assert_eq!(0, iface.println_count);

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
                    print res;
                    print ", ";
                    let count = count - 1;
                    if count <= 1 {
                        break;
                    };
                };
                println "";
                return res;
            };

            return fib(8);
        "#;
        let scope = Scope::new();
        let res = interpret(src, scope, &mut iface);
        match res {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(x) => assert_eq!(Value::Int(21), x),
            ExecResult::Error(e) => assert!(false, e),
        };

        // print should have been invoked twice per loop (=14)
        // println should have been invoked once (after loop)
        assert_eq!(14, iface.print_count);
        assert_eq!(1, iface.println_count);

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
        let scope = Scope::new();
        let res = interpret(src, scope, &mut iface);
        match res {
            ExecResult::None => assert!(false, "interpret() should not have returned None"),
            ExecResult::Break => assert!(false, "interpret() should not have returned Break"),
            ExecResult::Return(x) => assert_eq!(Value::Int(24), x),
            ExecResult::Error(e) => assert!(false, e),
        };
    }
}
