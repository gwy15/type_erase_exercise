struct App {
    handlers: Vec<Box<dyn Fn() -> ()>>,
}
impl App {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }
    pub fn handler(mut self, f: impl Fn() -> () + 'static) -> Self {
        self.handlers.push(Box::new(f));
        self
    }
    pub fn dispatch(&self) {
        for handler in self.handlers.iter() {
            (handler)()
        }
    }
}

#[test]
fn test_start_simple() {
    use mockall::*;

    #[automock]
    pub trait Handler {
        fn f1();
        fn f2();
        fn f3();
    }

    let f1_ctx = MockHandler::f1_context();
    f1_ctx.expect().times(1).returning(|| {});

    let f2_ctx = MockHandler::f2_context();
    f2_ctx.expect().times(1).returning(|| {});

    let f3_ctx = MockHandler::f3_context();
    f3_ctx.expect().times(1).returning(|| {});

    let app = App::new()
        .handler(MockHandler::f1)
        .handler(|| MockHandler::f2())
        .handler(MockHandler::f3);
    app.dispatch();
}
