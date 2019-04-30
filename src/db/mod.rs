//! Database access module.
pub mod humans;

use std::ops::Deref;

use diesel::connection::{Connection, TransactionManager};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::error::{Error, ErrorKind, ResultExt};

pub struct PgPool(Pool<ConnectionManager<PgConnection>>);

impl PgPool {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::new(manager).expect("Failed to create pool");
        PgPool(pool)
    }

    pub fn get_conn(&self) -> Result<PgConn, Error> {
        let conn = self.0.get().context(ErrorKind::DbPoolError)?;
        Ok(PgConn::new(conn))
    }

    pub fn transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&PgConn) -> Result<T, Error>,
    {
        let conn = self.get_conn()?;
        let transaction_manager = conn.transaction_manager();
        transaction_manager
            .begin_transaction(&*conn)
            .context(ErrorKind::DbTransaction)?;
        match f(&conn) {
            Ok(value) => {
                transaction_manager
                    .commit_transaction(&*conn)
                    .context(ErrorKind::DbTransaction)?;
                Ok(value)
            }
            Err(e) => {
                transaction_manager
                    .rollback_transaction(&*conn)
                    .context(ErrorKind::DbTransaction)?;
                Err(e)
            }
        }
    }

    #[cfg(test)]
    pub fn test_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&PgConn) -> Result<T, Error>,
    {
        let conn = self.get_conn()?;
        let transaction_manager = conn.transaction_manager();

        transaction_manager
            .begin_transaction(&*conn)
            .context(ErrorKind::DbTransaction)?;

        let result = f(&conn);

        transaction_manager
            .rollback_transaction(&*conn)
            .context(ErrorKind::DbTransaction)?;

        result
    }
}

pub struct PgConn(PooledConnection<ConnectionManager<PgConnection>>);

impl PgConn {
    pub(crate) fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self(conn)
    }

    pub fn get_conn(&self) -> &PgConnection {
        &*self.0
    }
}

impl Deref for PgConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl juniper::Context for PgConn {}
