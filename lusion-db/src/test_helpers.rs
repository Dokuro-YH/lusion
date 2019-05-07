//! Test helpers.
use diesel::connection::{Connection, TransactionManager};

use crate::error::DbError;
use crate::pg::PgConn;

pub fn with_transaction<F, T>(f: F) -> Result<T, DbError>
where
    F: FnOnce(&PgConn) -> Result<T, DbError>,
{
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = PgConn::establish(&database_url).unwrap();
    let transaction_manager = conn.transaction_manager();

    transaction_manager.begin_transaction(&conn)?;

    let result = f(&conn);

    transaction_manager.rollback_transaction(&conn)?;

    result
}
