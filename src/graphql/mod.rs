//! Graphql API module.
mod humans;

use http::StatusCode;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use tide::{Context, Response};

use crate::db::{PgConn, PgPool};
use crate::error::{Error, ErrorKind, ResultExt};
use crate::resp;

/// Graphql schema.
pub type Schema = juniper::RootNode<'static, QueryRoot, MutationRoot>;

/// Graphql query.
pub struct QueryRoot;

/// Graphql mutations.
pub struct MutationRoot;

graphql_object!(QueryRoot: PgConn |&self| {
    field humans() -> humans::QueryHuman { humans::QueryHuman }
});

graphql_object!(MutationRoot: PgConn |&self| {
    field humans() -> humans::MutationHuman { humans::MutationHuman }
});

pub async fn get_graphiql(_: Context<PgPool>) -> Response {
    let res = graphiql_source("http://localhost:8000/graphql");
    resp::html(StatusCode::OK, res)
}

pub async fn post_graphql(mut ctx: Context<PgPool>) -> Result<Response, Error> {
    let req: GraphQLRequest = await!(ctx.body_json()).context(ErrorKind::BadRequest)?;
    let schema = Schema::new(QueryRoot, MutationRoot);

    let pool = ctx.app_data();
    let res = pool.transaction(|conn| Ok(req.execute(&schema, &conn)))?;

    let status = if res.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    Ok(resp::json(status, res))
}
