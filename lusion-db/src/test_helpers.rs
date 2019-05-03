//! Test helpers.
use crate::{Error, PgConn};
use diesel::connection::{Connection, TransactionManager};

pub fn with_transaction<F, T>(f: F) -> Result<T, Error>
where
    F: FnOnce(&PgConn) -> Result<T, Error>,
{
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = PgConn::establish(&database_url).unwrap();
    let transaction_manager = conn.transaction_manager();

    transaction_manager.begin_transaction(&conn)?;

    let result = f(&conn);

    transaction_manager.rollback_transaction(&conn)?;

    result
}
