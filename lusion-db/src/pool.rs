use diesel::connection::{Connection, TransactionManager};

use crate::error::DbError;

/// A database connection pool.
pub trait DbPool {
    type Connection: Connection;

    /// Executes the given function
    fn with<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Self::Connection) -> Result<T, DbError>;

    /// Executes the given function inside of a database transaction
    fn transaction<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Self::Connection) -> Result<T, DbError>,
    {
        self.with(|conn| {
            let transaction_manager = conn.transaction_manager();
            transaction_manager.begin_transaction(conn)?;
            match f(&conn) {
                Ok(value) => {
                    transaction_manager.commit_transaction(conn)?;
                    Ok(value)
                }
                Err(e) => {
                    transaction_manager.rollback_transaction(conn)?;
                    Err(e)
                }
            }
        })
    }
}
