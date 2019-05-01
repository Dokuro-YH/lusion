//! Middleware-based security context.
use cookie::{Cookie, CookieJar, Key};
use futures::future::FutureObj;
use http::header::{self, HeaderValue};
use http_service::{Request, Response};
use tide::error::StringError;
use tide::middleware::{Middleware, Next};
use tide::Context;
use time::Duration;

use crate::security::{SecurityContext, SecuritySubject};

pub struct SecurityMiddleware {
    policy: Box<dyn SecurityPolicy>,
}

impl SecurityMiddleware {
    pub fn new<T: SecurityPolicy>(policy: T) -> Self {
        Self {
            policy: Box::new(policy),
        }
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self {
            policy: Box::new(CookieSecurityPolicy::default()),
        }
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for SecurityMiddleware {
    fn handle<'a>(
        &'a self,
        mut cx: Context<Data>,
        next: Next<'a, Data>,
    ) -> FutureObj<'a, Response> {
        let subject = self.policy.from_request(cx.request()).unwrap();
        let sc = SecurityContext::new(subject);
        box_async! {
            cx.extensions_mut().insert(sc.clone());

            let resp = await!(next.run(cx));

            if sc.is_changed() {
                self.policy.write_response(sc.subject(), resp).unwrap()
            } else {
                resp
            }
        }
    }
}

/// An `SecuritySubject` storage policy.
pub trait SecurityPolicy: 'static + Send + Sync {
    /// Load `SecuritySubject` from `Request`.
    fn from_request(&self, req: &Request) -> Result<Option<SecuritySubject>, StringError>;

    fn write_response(
        &self,
        subject: Option<SecuritySubject>,
        resp: Response,
    ) -> Result<Response, StringError>;
}

pub struct CookieSecurityPolicy {
    key: Key,
    path: String,
    name: String,
    domain: Option<String>,
    secure: bool,
    max_age: Option<Duration>,
}

impl CookieSecurityPolicy {
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

impl Default for CookieSecurityPolicy {
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

impl SecurityPolicy for CookieSecurityPolicy {
    fn from_request(&self, req: &Request) -> Result<Option<SecuritySubject>, StringError> {
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
            let subject = serde_json::from_str(auth_cookie.value())
                .map_err(|e| StringError(format!("Failed to deserialize: {}", e)))?;

            Ok(Some(subject))
        } else {
            Ok(None)
        }
    }

    fn write_response(
        &self,
        subject: Option<SecuritySubject>,
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

        if let Some(subject) = subject {
            let value = serde_json::to_string(&subject)
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
    use crate::resp;
    use crate::security::SecurityExt;
    use crate::test_helpers::*;
    use http::StatusCode;
    use http_service::Body;

    async fn retrieve_user_info(mut ctx: Context<()>) -> Response {
        let res = ctx
            .principal()
            .unwrap()
            .unwrap_or_else(|| "anonymous".to_owned());
        resp::json(StatusCode::OK, res)
    }

    async fn check_user_authority(mut ctx: Context<()>) -> Response {
        let res = ctx.check_authority("user").unwrap();
        resp::json(StatusCode::OK, res)
    }

    async fn remember_user_info(mut ctx: Context<()>) {
        ctx.remember("remembered", vec!["user".to_owned()]).unwrap();
    }

    async fn forget_user_info(mut ctx: Context<()>) {
        ctx.forget().unwrap();
    }

    fn named_cookie_app(cookie_name: &str) -> tide::App<()> {
        let mut app = tide::App::new(());
        app.middleware(SecurityMiddleware::new(
            CookieSecurityPolicy::new(&[0; 32]).name(cookie_name),
        ));

        app.at("/get").get(retrieve_user_info);
        app.at("/remember").get(remember_user_info);
        app.at("/check").get(check_user_authority);
        app.at("/forget").get(forget_user_info);
        app
    }

    fn app() -> tide::App<()> {
        let mut app = tide::App::new(());
        app.middleware(SecurityMiddleware::default());

        app.at("/get").get(retrieve_user_info);
        app.at("/remember").get(remember_user_info);
        app.at("/check").get(check_user_authority);
        app.at("/forget").get(forget_user_info);
        app
    }

    #[test]
    fn successfully_retrieve_request_user_info() {
        let mut server = init_service(app());
        let req = http::Request::get("/get").body(Body::empty()).unwrap();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "\"anonymous\"");
    }

    #[test]
    fn successfully_remember_user_info() {
        let mut server = init_service(app());

        let req = http::Request::get("/remember").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let auth_cookie = res.get_cookie("tide-auth").unwrap();

        let req = http::Request::get("/get").cookie(&auth_cookie).to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "\"remembered\"");
    }

    #[test]
    fn successfully_check_user_authority() {
        let mut server = init_service(app());

        let req = http::Request::get("/remember").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let auth_cookie = res.get_cookie("tide-auth").unwrap();

        let req = http::Request::get("/check")
            .cookie(&auth_cookie)
            .to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "true");
    }

    #[test]
    fn successfully_forget_user_info() {
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
        assert_eq!(res.read_body(), "\"remembered\"");
    }

    #[test]
    fn successfully_set_cookie_security_policy_cookie_name() {
        let mut server = init_service(named_cookie_app("test-cookie123"));

        let req = http::Request::get("/remember").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert!(res.headers().contains_key(header::SET_COOKIE));

        let auth_cookie = res.get_cookie("test-cookie123");
        assert!(auth_cookie.is_some());
    }
}
