use diesel::connection::{Connection, TransactionManager};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::error::Result;

pub type PgConn = PgConnection;

pub struct PgPool(Pool<ConnectionManager<PgConn>>);

impl PgPool {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConn>::new(database_url);
        let pool = Pool::new(manager).expect("Failed to create pool");
        PgPool(pool)
    }

    pub fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&PgConn) -> Result<T>,
    {
        let conn = self.0.get()?;
        let transaction_manager = conn.transaction_manager();
        transaction_manager.begin_transaction(&conn)?;
        match f(&conn) {
            Ok(value) => {
                transaction_manager.commit_transaction(&conn)?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager.rollback_transaction(&conn)?;
                Err(e)
            }
        }
    }
}
