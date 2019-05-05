//! Lusion Database Library.

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

pub mod humans;
pub mod pg;
pub mod test;
pub mod users;

mod error;
mod schema;

#[cfg(test)]
mod test_helpers;

pub use self::error::{Error, Result};

use diesel::connection::{Connection, TransactionManager};

/// A database connection pool.
pub trait DbPool {
    type Connection: Connection;

    /// Executes the given function
    fn with<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Self::Connection) -> Result<T>;

    /// Executes the given function inside of a database transaction
    fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Self::Connection) -> Result<T>,
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
