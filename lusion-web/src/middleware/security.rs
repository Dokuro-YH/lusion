//! Middleware-based security context.
use cookie::{Cookie, CookieJar, Key};
use futures::future::BoxFuture;
use http::header::{self, HeaderValue};
use tide::error::StringError;
use tide::middleware::{Middleware, Next};
use tide::Context;
use time::Duration;

use crate::request::Request;
use crate::response::Response;
use crate::security::{Identity, SecurityContext};

pub struct SecurityMiddleware {
    policy: Box<dyn SecurityIdentityPolicy>,
}

impl SecurityMiddleware {
    pub fn new<T: SecurityIdentityPolicy>(policy: T) -> Self {
        Self {
            policy: Box::new(policy),
        }
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self {
            policy: Box::new(CookieIdentityPolicy::default()),
        }
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for SecurityMiddleware {
    fn handle<'a>(
        &'a self,
        mut cx: Context<Data>,
        next: Next<'a, Data>,
    ) -> BoxFuture<'a, Response> {
        let identity = self.policy.from_request(cx.request()).unwrap();
        let sc = SecurityContext::new(identity);
        box_async! {
            cx.extensions_mut().insert(sc.clone());

            let resp = await!(next.run(cx));

            if sc.is_changed() {
                self.policy.write_response(sc.identity(), resp).unwrap()
            } else {
                resp
            }
        }
    }
}

/// An `Identity` storage policy.
pub trait SecurityIdentityPolicy: 'static + Send + Sync {
    /// Load `Identity` from `Request`.
    fn from_request(&self, req: &Request) -> Result<Option<Identity>, StringError>;

    fn write_response(
        &self,
        identity: Option<Identity>,
        resp: Response,
    ) -> Result<Response, StringError>;
}

pub struct CookieIdentityPolicy {
    key: Key,
    path: String,
    name: String,
    domain: Option<String>,
    secure: bool,
    max_age: Option<Duration>,
}

impl CookieIdentityPolicy {
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: Key::from_master(key),
            ..Self::default()
        }
    }

    pub fn path<S: Into<String>>(mut self, value: S) -> Self {
        self.path = value.into();
        self
    }

    pub fn name<S: Into<String>>(mut self, value: S) -> Self {
        self.name = value.into();
        self
    }

    pub fn domain<S: Into<String>>(mut self, value: S) -> Self {
        self.domain = Some(value.into());
        self
    }

    pub fn secure(mut self, value: bool) -> Self {
        self.secure = value;
        self
    }

    pub fn max_age(self, seconds: i64) -> Self {
        self.max_age_time(Duration::seconds(seconds))
    }

    pub fn max_age_time(mut self, value: Duration) -> Self {
        self.max_age = Some(value);
        self
    }
}

impl Default for CookieIdentityPolicy {
    fn default() -> Self {
        Self {
            key: Key::generate(),
            name: "tide-auth".to_owned(),
            path: "/".to_owned(),
            domain: None,
            secure: false,
            max_age: None,
        }
    }
}

impl SecurityIdentityPolicy for CookieIdentityPolicy {
    fn from_request(&self, req: &Request) -> Result<Option<Identity>, StringError> {
        let mut jar = CookieJar::new();

        for hdr in req.headers().get_all(http::header::COOKIE) {
            let s = hdr
                .to_str()
                .map_err(|e| StringError(format!("Failed to parse header value: {}", e)))?;

            for cookie_str in s.split(';').map(str::trim) {
                if !cookie_str.is_empty() {
                    let cookie = Cookie::parse_encoded(cookie_str.to_owned())
                        .map_err(|e| StringError(format!("Failed to parse cookie: {}", e)))?;
                    jar.add_original(cookie);
                }
            }
        }

        if let Some(auth_cookie) = jar.private(&self.key).get(&self.name) {
            let identity = serde_json::from_str(auth_cookie.value())
                .map_err(|e| StringError(format!("Failed to deserialize: {}", e)))?;

            Ok(Some(identity))
        } else {
            Ok(None)
        }
    }

    fn write_response(
        &self,
        identity: Option<Identity>,
        mut resp: Response,
    ) -> Result<Response, StringError> {
        let mut jar = CookieJar::new();
        let mut cookie = Cookie::named(self.name.clone());
        cookie.set_path(self.path.clone());
        cookie.set_secure(self.secure);
        cookie.set_http_only(true);

        if let Some(ref domain) = self.domain {
            cookie.set_domain(domain.clone());
        }

        if let Some(max_age) = self.max_age {
            cookie.set_max_age(max_age);
        }

        if let Some(identity) = identity {
            let value = serde_json::to_string(&identity)
                .map_err(|e| StringError(format!("Failed to serialize: {}", e)))?;
            cookie.set_value(value);

            jar.private(&self.key).add(cookie);
        } else {
            jar.add_original(cookie.clone());
            jar.private(&self.key).remove(cookie);
        }

        for cookie in jar.delta() {
            let hv = HeaderValue::from_str(&cookie.to_string());
            if let Ok(val) = hv {
                resp.headers_mut().append(header::SET_COOKIE, val);
            } else {
                return Ok(http::Response::builder()
                    .status(http::status::StatusCode::INTERNAL_SERVER_ERROR)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(http_service::Body::empty())
                    .unwrap());
            }
        }

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::{self, StatusCode};
    use crate::security::SecurityExt;
    use crate::test_helpers::*;

    async fn retrieve(mut ctx: Context<()>) -> Response {
        let res = ctx
            .identity()
            .unwrap()
            .unwrap_or_else(|| Identity::new("anonymous"));
        response::json(StatusCode::OK, res)
    }

    async fn remember(mut ctx: Context<()>) {
        ctx.remember(Identity::new("user")).unwrap();
    }

    async fn forget(mut ctx: Context<()>) {
        ctx.forget().unwrap();
    }

    fn named_cookie_app(cookie_name: &str) -> tide::App<()> {
        let mut app = tide::App::new(());
        app.middleware(SecurityMiddleware::new(
            CookieIdentityPolicy::new(&[0; 32]).name(cookie_name),
        ));

        app.at("/get").get(retrieve);
        app.at("/remember").get(remember);
        app.at("/forget").get(forget);
        app
    }

    fn app() -> tide::App<()> {
        let mut app = tide::App::new(());
        app.middleware(SecurityMiddleware::default());

        app.at("/get").get(retrieve);
        app.at("/remember").get(remember);
        app.at("/forget").get(forget);
        app
    }

    #[test]
    fn test_retrieve_should_be_200() {
        let mut server = init_service(app());
        let req = http::Request::get("/get").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "\"anonymous\"");
    }

    #[test]
    fn test_remember_should_be_200() {
        let mut server = init_service(app());

        let req = http::Request::get("/remember").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let auth_cookie = res.get_cookie("tide-auth").unwrap();

        let req = http::Request::get("/get").cookie(&auth_cookie).to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "\"user\"");
    }

    #[test]
    fn test_forget_should_be_200() {
        let mut server = init_service(app());

        let req = http::Request::get("/remember").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let auth_cookie = res.get_cookie("tide-auth").unwrap();

        let req = http::Request::get("/forget")
            .cookie(&auth_cookie)
            .to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let req = http::Request::get("/get").cookie(&auth_cookie).to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "\"user\"");
    }

    #[test]
    fn test_set_cookie_identity_policy_cookie_name() {
        let mut server = init_service(named_cookie_app("test-cookie123"));

        let req = http::Request::get("/remember").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let auth_cookie = res.get_cookie("test-cookie123");
        assert!(auth_cookie.is_some());
    }
}
