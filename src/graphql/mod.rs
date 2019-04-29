///! Graphql API module.
mod humans;

use crate::db::PgConn;

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
