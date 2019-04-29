///! Database access module.
pub mod humans;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use crate::error::{Error, ErrorKind, ResultExt};

pub struct PgConn {
    pub(crate) conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl juniper::Context for PgConn {}

pub struct PgPool(Pool<ConnectionManager<PgConnection>>);

impl PgPool {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::new(manager).expect("Failed to create pool");
        PgPool(pool)
    }

    pub fn get_conn(&self) -> Result<PgConn, Error> {
        let conn = self.0.get().context(ErrorKind::DbPoolError)?;
        Ok(PgConn { conn })
    }
}
