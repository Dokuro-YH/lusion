#![feature(async_await, await_macro)]

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

use std::{env, io};

use http::StatusCode;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use tide::{App, Context, Response};

use crate::db::PgPool;
use crate::error::{Error, ErrorKind, ResultExt};
use crate::graphql::{MutationRoot, QueryRoot, Schema};
use crate::middleware::{CookieSecurityPolicy, SecurityMiddleware};

static AUTH_SIGNING_KEY: &[u8] = &[0; 32];

async fn graphiql(_: Context<PgPool>) -> Response {
    let res = graphiql_source("http://localhost:8000/graphql");
    resp::html(StatusCode::OK, res)
}

async fn graphql(mut ctx: Context<PgPool>) -> Result<Response, Error> {
    let pool = ctx.app_data();
    let conn = pool.get_conn()?;
    let schema = Schema::new(QueryRoot, MutationRoot);
    let req: GraphQLRequest = await!(ctx.body_json()).context(ErrorKind::GraphqlError)?;
    let res = req.execute(&schema, &conn);
    let status = if res.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    Ok(resp::json(status, res))
}

fn main() -> io::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = PgPool::new(&database_url);

    let mut app = App::new(db);
    app.middleware(SecurityMiddleware::new(
        CookieSecurityPolicy::new(AUTH_SIGNING_KEY)
            .path("/")
            .name("auth-cookie")
            .domain("localhost")
            .secure(false)
            .max_age(3600),
    ));

    app.at("/graphiql").get(graphiql);
    app.at("/graphql").post(graphql);

    Ok(app.serve("127.0.0.1:8000")?)
}
