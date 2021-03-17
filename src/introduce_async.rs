use std::marker::PhantomData;

struct App {
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
        let f = FunctionWrapper::new(f);
        self.services.push(Box::new(f));
        self
    }

    pub fn dispatch(&self, req: Request) {
        for f in self.services.iter() {
            f.handle_request(&req);
        }
    }
}

trait Service {
    fn handle_request(&self, req: &Request);
}
/// `Handler<(T1, T2)>`基本上等价于 `F(T1, T2)->()`。
trait Handler<T>: 'static {
    fn call(&self, params: T);
}
#[rustfmt::skip]
mod _impl_handler {
    use super::*;
    // delegate
    impl<F> Handler<()> for F where F: Fn() -> () + 'static {
        fn call(&self, params: ()) {
            (self)()
        }
    }
    macro_rules! f {
        (($($Ts:ident),*), ($($Ns:tt),*)) => {
            impl<F, $($Ts,)*> Handler<( $($Ts, )* )> for F
            where
                F: Fn( $($Ts,)* ) -> () + 'static
            {
                fn call(&self, params: ( $($Ts,)* )) {
                    (self)(
                        $(params.$Ns, )*
                    )
                }
            }
        };
    }
    f!((T1), (0));
    f!((T1, T2), (0, 1));
    f!((T1, T2, T3), (0, 1, 2));
    f!((T1, T2, T3, T4), (0, 1, 2, 3));
    f!((T1, T2, T3, T4, T5), (0, 1, 2, 3, 4));
    f!((T1, T2, T3, T4, T5, T6), (0, 1, 2, 3, 4, 5));
    f!((T1, T2, T3, T4, T5, T6, T7), (0, 1, 2, 3, 4, 5, 6));
    f!((T1, T2, T3, T4, T5, T6, T7, T8), (0, 1, 2, 3, 4, 5, 6, 7));
    f!((T1, T2, T3, T4, T5, T6, T7, T8, T9), (0, 1, 2, 3, 4, 5, 6, 7, 8));
    f!((T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), (0, 1, 2, 3, 4, 5, 6, 7, 8, 9));
}

/// 这里将函数指针的 T 提到类型参数中
struct FunctionWrapper<F, T> {
    f: F,
    _t: PhantomData<T>,
}
impl<F, T> FunctionWrapper<F, T>
where
    F: Handler<T>,
    T: FromRequest,
{
    pub fn new(f: F) -> Self {
        Self { f, _t: PhantomData }
    }
}
/// 将 `Service` 逻辑实现给函数指针
impl<F, T> Service for FunctionWrapper<F, T>
where
    F: Handler<T>,
    T: FromRequest,
{
    fn handle_request(&self, req: &Request) {
        // 在这里从请求中提取参数
        let params = T::from_request(req);
        self.f.call(params)
    }
}

struct Request {
    s: String,
}
trait FromRequest {
    fn from_request(req: &Request) -> Self;
}

#[rustfmt::skip]
mod _impl_from_request {
    use super::*;

    impl FromRequest for String {
        fn from_request(req: &Request) -> Self {
            req.s.clone()
        }
    }
    impl FromRequest for u32 {
        fn from_request(req: &Request) -> Self {
            req.s.parse().unwrap()
        }
    }
    impl FromRequest for () {
        fn from_request(req: &Request) -> Self {
            ()
        }
    }
    // propagate
    macro_rules! f {
        ($($Ts:tt),*) => {
            impl< $($Ts,)* > FromRequest for ( $($Ts,)* )
            where
                $(
                    $Ts: FromRequest,
                )*
            {
                fn from_request(req: &Request) -> Self {
                    (
                        $(
                            $Ts::from_request(req),
                        )*
                    )
                }
            }
        };
    }
    f!(T1);
    f!(T1, T2);
    f!(T1, T2, T3);
    f!(T1, T2, T3, T4);
    f!(T1, T2, T3, T4, T5);
    f!(T1, T2, T3, T4, T5, T6);
    f!(T1, T2, T3, T4, T5, T6, T7);
    f!(T1, T2, T3, T4, T5, T6, T7, T8);
    f!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
    f!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
}

impl Request {
    pub fn new(s: impl Into<String>) -> Self {
        Self { s: s.into() }
    }
}

#[test]
fn test_second_try() {
    use mockall::*;

    #[automock]
    pub trait Handler {
        // 测试无参数
        fn f0();
        // 测试获取一个 String
        fn f1(s: String);
        // 测试获取两个参数
        fn f2(n: u32, s: String);
        // 反过来也可以
        fn f3(t: (), s: String, n: u32);
    }

    let f0_ctx = MockHandler::f0_context();
    f0_ctx.expect().times(1).returning(|| {});

    let f1_ctx = MockHandler::f1_context();
    f1_ctx.expect().times(1).returning(|s: String| {
        assert_eq!(s, "123");
    });

    let f2_ctx = MockHandler::f2_context();
    f2_ctx.expect().times(1).returning(|n: u32, s: String| {
        assert_eq!(n, 123);
        assert_eq!(s, "123");
    });

    let f3_ctx = MockHandler::f3_context();
    f3_ctx
        .expect()
        .times(1)
        .returning(|t: (), s: String, n: u32| {
            assert_eq!(s, "123");
            assert_eq!(n, 123);
        });

    let app = App::new()
        .handler(MockHandler::f0)
        .handler(|s: String| MockHandler::f1(s))
        .handler(MockHandler::f2)
        .handler(MockHandler::f3);
    app.dispatch(Request::new("123"));
}
