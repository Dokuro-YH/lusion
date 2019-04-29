#![feature(async_await, await_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;

pub mod schema;
pub mod error;
pub mod db;
pub mod graphql;
pub mod resp;

use std::{io, env};

use http::StatusCode;
use tide::{App, Context, Response};
use juniper::http::{GraphQLRequest};
use juniper::http::graphiql::graphiql_source;

use crate::db::{PgPool};
use crate::graphql::{Schema, QueryRoot, MutationRoot};
use crate::error::{ErrorKind, Error, ResultExt};

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
    app.at("/graphiql").get(graphiql);
    app.at("/graphql").post(graphql);

    Ok(app.serve("127.0.0.1:8000")?)
}
