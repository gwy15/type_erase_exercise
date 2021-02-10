use std::{future::Future, marker::PhantomData, pin::Pin};

struct App {
    // handlers: Vec<Box<dyn Handler>>,
    services: Vec<Box<dyn Service>>,
}
impl App {
    pub fn new() -> Self {
        Self { services: vec![] }
    }
    pub fn handler<F, T, R>(mut self, f: F) -> Self
    where
        F: Handler<T, R>,
        T: FromRequest + 'static,
        R: Future<Output = ()> + 'static,
    {
        self.services.push(Box::new(ServiceWrapper::new(f)));
        self
    }
    pub async fn dispatch(&self, req: String) {
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
trait Handler<T, R>: Clone + 'static
where
    R: Future<Output = ()>,
{
    fn call(&self, param: T) -> R;
}
impl<F, R> Handler<(), R> for F
where
    F: Fn() -> R + Clone + 'static,
    R: Future<Output = ()>,
{
    fn call(&self, param: ()) -> R {
        (self)()
    }
}
impl<F, T, R> Handler<(T,), R> for F
where
    F: Fn(T) -> R + Clone + 'static,
    T: FromRequest,
    R: Future<Output = ()>,
{
    fn call(&self, param: (T,)) -> R {
        (self)(param.0)
    }
}
impl<F, T1, T2, R> Handler<(T1, T2), R> for F
where
    F: Fn(T1, T2) -> R + Clone + 'static,
    T1: FromRequest,
    T2: FromRequest,
    R: Future<Output = ()>,
{
    fn call(&self, param: (T1, T2)) -> R {
        (self)(param.0, param.1)
    }
}

trait Service {
    fn handle_request(&self, req: &str) -> Pin<Box<dyn Future<Output = ()>>>;
}

struct ServiceWrapper<F, T, R> {
    f: F,
    _t: PhantomData<(T, R)>,
}
impl<F, T, R> ServiceWrapper<F, T, R> {
    pub fn new(f: F) -> Self
    where
        F: Handler<T, R>,
        T: FromRequest,
        R: Future<Output = ()>,
    {
        Self { f, _t: PhantomData }
    }
}
impl<F, T, R> Service for ServiceWrapper<F, T, R>
where
    F: Handler<T, R>,
    T: FromRequest + 'static,
    R: Future<Output = ()>,
{
    fn handle_request(&self, req: &str) -> Pin<Box<dyn Future<Output = ()>>> {
        let params = T::from_request(req);
        let f = self.f.clone();
        Box::pin(async move { f.call(params).await })
    }
}

#[tokio::test]
async fn test_add_handlers() {
    async fn none() {
        println!("print from none");
    }

    async fn one(s: String) {
        println!("print from one: s = {}", s);
    }

    async fn two(n1: u32, n2: u64) {
        println!("print from two: n1 = {}, n2 = {}", n1, n2);
    }

    let app = App::new().handler(none).handler(one).handler(two);
    app.dispatch("1234".to_string()).await;
}
