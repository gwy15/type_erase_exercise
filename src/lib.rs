#![allow(unused)]

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
}

/// 要求 T 可解析
trait FromRequest {}
impl FromRequest for () {}
impl FromRequest for String {}
impl FromRequest for u32 {}
impl FromRequest for u64 {}
impl<T1> FromRequest for (T1,) where T1: FromRequest {}
impl<T1, T2> FromRequest for (T1, T2)
where
    T1: FromRequest,
    T2: FromRequest,
{
}

/// 设置 T 为 Handler 接受的类型
trait Handler<T>: Clone + 'static {}
impl<F> Handler<()> for F where F: Fn() -> () + Clone + 'static {}
impl<F, T> Handler<(T,)> for F
where
    F: Fn(T) -> () + Clone + 'static,
    T: FromRequest,
{
}
impl<F, T1, T2> Handler<(T1, T2)> for F
where
    F: Fn(T1, T2) -> () + Clone + 'static,
    T1: FromRequest,
    T2: FromRequest,
{
}

trait Service {}

struct ServiceWrapper<F> {
    f: F,
}
impl<F> ServiceWrapper<F> {
    pub fn new<T>(f: F) -> Self
    where
        F: Handler<T>,
    {
        Self { f }
    }
}
impl<T> Service for ServiceWrapper<T> {}

#[test]
fn test_add_handlers() {
    fn none() {}

    fn one(s: String) {}

    fn two(n1: u32, n2: u64) {}

    App::new().handler(none).handler(one).handler(two);
}
