pub trait ScriptInterface {
    fn print(&mut self, s: &str);
    fn println(&mut self, s: &str);
}

pub struct DefaultScriptInterface { }

impl DefaultScriptInterface {
    pub fn new() -> DefaultScriptInterface {
        DefaultScriptInterface {}
    }
}

impl ScriptInterface for DefaultScriptInterface {
    fn print(&mut self, s: &str) {
        print!("{}", s);
    }

    fn println(&mut self, s: &str) {
        println!("{}", s);
    }
}
