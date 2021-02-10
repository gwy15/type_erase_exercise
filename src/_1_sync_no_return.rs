use std::marker::PhantomData;

struct App {
    // handlers: Vec<Box<dyn Handler>>,
    services: Vec<Box<dyn Service>>,
}
impl App {
    pub fn new() -> Self {
        Self { services: vec![] }
    }
    pub fn handler<F, T>(mut self, f: F) -> Self
    where
        F: Handler<T>,
        T: FromRequest + 'static,
    {
        self.services.push(Box::new(ServiceWrapper::new(f)));
        self
    }
    pub fn dispatch(&self, req: String) {
        for service in self.services.iter() {
            service.handle_request(&req);
        }
    }
}

/// 要求 T 可解析
trait FromRequest {
    fn from_request(req: &str) -> Self;
}
impl FromRequest for () {
    fn from_request(req: &str) -> Self {
        ()
    }
}
impl FromRequest for String {
    fn from_request(req: &str) -> Self {
        req.to_string()
    }
}
impl FromRequest for u32 {
    fn from_request(req: &str) -> Self {
        req.parse().unwrap()
    }
}
impl FromRequest for u64 {
    fn from_request(req: &str) -> Self {
        req.parse().unwrap()
    }
}
impl<T1> FromRequest for (T1,)
where
    T1: FromRequest,
{
    fn from_request(req: &str) -> Self {
        (T1::from_request(req),)
    }
}
impl<T1, T2> FromRequest for (T1, T2)
where
    T1: FromRequest,
    T2: FromRequest,
{
    fn from_request(req: &str) -> Self {
        (T1::from_request(req), T2::from_request(req))
    }
}

/// 设置 T 为 Handler 接受的类型
trait Handler<T>: Clone + 'static {
    fn call(&self, param: T);
}
impl<F> Handler<()> for F
where
    F: Fn() -> () + Clone + 'static,
{
    fn call(&self, param: ()) {
        (self)()
    }
}
impl<F, T> Handler<(T,)> for F
where
    F: Fn(T) -> () + Clone + 'static,
    T: FromRequest,
{
    fn call(&self, param: (T,)) {
        (self)(param.0)
    }
}
impl<F, T1, T2> Handler<(T1, T2)> for F
where
    F: Fn(T1, T2) -> () + Clone + 'static,
    T1: FromRequest,
    T2: FromRequest,
{
    fn call(&self, param: (T1, T2)) {
        (self)(param.0, param.1)
    }
}

trait Service {
    fn handle_request(&self, req: &str);
}

struct ServiceWrapper<F, T> {
    f: F,
    _t: PhantomData<T>,
}
impl<F, T> ServiceWrapper<F, T> {
    pub fn new(f: F) -> Self
    where
        F: Handler<T>,
        T: FromRequest,
    {
        Self { f, _t: PhantomData }
    }
}
impl<F, T> Service for ServiceWrapper<F, T>
where
    F: Handler<T>,
    T: FromRequest,
{
    fn handle_request(&self, req: &str) {
        let params = T::from_request(req);
        self.f.call(params);
    }
}

#[test]
fn test_add_handlers() {
    fn none() {
        eprintln!("print from none");
    }

    fn one(s: String) {
        eprintln!("print from one: s = {}", s);
    }

    fn two(n1: u32, n2: u64) {
        eprintln!("print from two: n1 = {}, n2 = {}", n1, n2);
    }

    let app = App::new().handler(none).handler(one).handler(two);
    app.dispatch("1234".to_string());
}
