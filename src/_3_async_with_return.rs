use std::{error::Error, future::Future, marker::PhantomData, num::ParseIntError, pin::Pin};

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
            service.handle_request(&req).await;
        }
    }
}

/// 要求 T 可解析
type FromRequestError = Box<dyn Error>;

trait FromRequest: Sized {
    fn from_request(req: &str) -> Result<Self, FromRequestError>;
}
impl FromRequest for () {
    fn from_request(req: &str) -> Result<Self, FromRequestError> {
        Ok(())
    }
}
impl FromRequest for String {
    fn from_request(req: &str) -> Result<Self, FromRequestError> {
        Ok(req.to_string())
    }
}
impl FromRequest for u32 {
    fn from_request(req: &str) -> Result<Self, FromRequestError> {
        Ok(req.parse()?)
    }
}
impl FromRequest for u64 {
    fn from_request(req: &str) -> Result<Self, FromRequestError> {
        Ok(req.parse()?)
    }
}
impl<T1> FromRequest for (T1,)
where
    T1: FromRequest,
{
    fn from_request(req: &str) -> Result<Self, FromRequestError> {
        Ok((T1::from_request(req)?,))
    }
}
impl<T1, T2> FromRequest for (T1, T2)
where
    T1: FromRequest,
    T2: FromRequest,
{
    fn from_request(req: &str) -> Result<Self, FromRequestError> {
        Ok((T1::from_request(req)?, T2::from_request(req)?))
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
        if let Ok(params) = T::from_request(req) {
            let f = self.f.clone();
            Box::pin(async move { f.call(params).await })
        } else {
            Box::pin(async {})
        }
    }
}

#[tokio::test]
async fn test_add_handlers() {
    async fn none() {
        eprintln!("[0] print from none");
    }

    async fn one(s: String) {
        eprintln!("[1] print from one: s = {}", s);
    }

    async fn two(n1: u32, n2: u64) {
        eprintln!("[2] print from two: n1 = {}, n2 = {}", n1, n2);
    }

    async fn three(n1: u32, n2: u64) {
        eprintln!("[3] print from three: n1 = {}, n2 = {}", n1, n2);
    }

    async fn mixed(s: String, n: u64) {
        eprintln!("[4] print from mixed: s = {}, n = {}", s, n);
    }

    let app = App::new()
        .handler(none)
        .handler(one)
        .handler(two)
        .handler(three)
        .handler(mixed);
    app.dispatch("3333".to_string()).await;
    app.dispatch("333a3".to_string()).await;
}
