//! Test helpers.
use cookie::Cookie;
use futures::executor::block_on;
use http_service::{Body, Request, Response};
use http_service_mock::{make_server, TestBackend};
use tide::{App, Server};

use crate::db::PgPool;

pub fn init_pool() -> PgPool {
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::new(&database_url)
}

pub fn init_service<AppData: Send + Sync + 'static>(
    app: App<AppData>,
) -> TestBackend<Server<AppData>> {
    make_server(app.into_http_service()).unwrap()
}

pub fn call_service<AppData: Send + Sync + 'static>(
    service: &mut TestBackend<Server<AppData>>,
    req: Request,
) -> Response {
    let res = service.simulate(req).unwrap();
    res
}

pub trait RequestBuilderExt {
    fn cookie<'a>(&mut self, cookie: &Cookie<'a>) -> &mut Self;

    fn json<T: serde::Serialize>(&mut self, payload: T) -> Request;

    fn to_request(&mut self) -> Request;
}

impl RequestBuilderExt for http::request::Builder {
    fn cookie<'a>(&mut self, cookie: &Cookie<'a>) -> &mut Self {
        self.header(http::header::COOKIE, cookie.encoded().to_string());
        self
    }

    fn json<T: serde::Serialize>(&mut self, payload: T) -> Request {
        self.body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap()
    }

    fn to_request(&mut self) -> Request {
        self.body(Body::empty()).unwrap()
    }
}

pub trait ResponseExt {
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>>;

    fn read_body(self) -> String;
}

impl ResponseExt for http::Response<Body> {
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        let cookie_header = self
            .headers()
            .get(http::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        let value = cookie_header
            .split(';')
            .map(str::trim)
            .find(|s| s.starts_with(name))
            .unwrap();
        Cookie::parse_encoded(value.to_owned()).ok()
    }

    fn read_body(self) -> String {
        let bytes = block_on(self.into_body().into_vec()).unwrap();
        String::from_utf8(bytes).unwrap()
    }
}
