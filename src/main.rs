//! An experimental, Web API based on async/await IO implementation.
use std::{env, io};

use lusion_db::pg::PgPool;
use lusion_web::middleware::fs::Static;
use lusion_web::middleware::security::{CookieIdentityPolicy, SecurityMiddleware};

static AUTH_SIGNING_KEY: &[u8] = &[0; 32];

fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "debug,lusion_web=debug");

    dotenv::dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::new(&database_url).expect("Failed to create pool");

    let mut app = tide::App::new(pool);
    app.middleware(SecurityMiddleware::new(
        CookieIdentityPolicy::new(AUTH_SIGNING_KEY)
            .path("/")
            .name("auth-cookie")
            .domain("localhost")
            .secure(false)
            .max_age(3600),
    ));
    app.middleware(Static::new("/images", "./images"));

    app.at("/api").nest(|api| {
        use lusion_web::endpoints::*;

        api.at("/users").get(users::get_users);
        api.at("/users").post(users::post_user);
        api.at("/users/:user_id").get(users::get_user);
        api.at("/users/:user_id").delete(users::delete_user);
        api.at("/users/:user_id/password")
            .put(users::put_user_password);
    });

    Ok(app.serve("127.0.0.1:8000")?)
}
