use std::{env, io};

use lusion::db::PgPool;
use lusion::graphql::{get_graphiql, post_graphql};
use lusion::middleware::security::{CookieIdentityPolicy, SecurityMiddleware};

static AUTH_SIGNING_KEY: &[u8] = &[0; 32];

fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::new(&database_url);

    let mut app = tide::App::new(pool);
    app.middleware(SecurityMiddleware::new(
        CookieIdentityPolicy::new(AUTH_SIGNING_KEY)
            .path("/")
            .name("auth-cookie")
            .domain("localhost")
            .secure(false)
            .max_age(3600),
    ));

    app.at("/graphiql").get(get_graphiql);
    app.at("/graphql").post(post_graphql);

    Ok(app.serve("127.0.0.1:8000")?)
}
