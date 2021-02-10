#![allow(unused)]

struct App {
    handlers: Vec<Box<dyn Handler>>,
}
impl App {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }
    pub fn handler<F>(mut self, f: F) -> Self
    where
        F: Handler + 'static,
    {
        self.handlers.push(Box::new(f));
        self
    }
}

trait Handler {}
impl<F> Handler for F where F: Fn() -> () {}
impl<F> Handler for F where F: Fn(String) -> () {}

#[test]
fn test_add_handlers() {
    fn none() {}

    fn one(s: String) {}

    App::new().handler(none).handler(one);
}
