#![feature(async_await, await_macro)]

///! An experimental, Web API based on async/await IO implementation.

macro_rules! box_async {
    {$($t:tt)*} => {
        FutureObj::new(Box::new(async move { $($t)* }))
    };
}
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;

pub mod db;
pub mod error;
pub mod graphql;
pub mod middleware;
pub mod resp;
pub mod schema;
pub mod security;

#[cfg(test)]
mod test_helpers;

use std::{env, io};

use crate::db::PgPool;
use crate::graphql::{get_graphiql, post_graphql};
use crate::middleware::{CookieSecurityPolicy, SecurityMiddleware};

static AUTH_SIGNING_KEY: &[u8] = &[0; 32];

fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPool::new(&database_url);

    let mut app = tide::App::new(db);
    app.middleware(SecurityMiddleware::new(
        CookieSecurityPolicy::new(AUTH_SIGNING_KEY)
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
