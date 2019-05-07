//! PostgreSQL module.
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::error::DbError;
use crate::DbPool;

/// A PostgreSQL connection.
pub type PgConn = PgConnection;

/// A PostgreSQL connection pool.
pub struct PgPool(Pool<ConnectionManager<PgConn>>);

impl PgPool {
    pub fn new(database_url: &str) -> Result<Self, DbError> {
        log::debug!("initialize database: {}", database_url);

        let manager = ConnectionManager::<PgConn>::new(database_url);
        let pool = Pool::new(manager)?;
        Ok(PgPool(pool))
    }
}

impl DbPool for PgPool {
    type Connection = PgConn;

    fn with<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&Self::Connection) -> Result<T, DbError>,
    {
        let conn = self.0.get().map_err(DbError::Pool)?;
        f(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    #[test]
    fn test_pg_pool() {
        let database_url = dotenv::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost/lusion".to_owned());
        let pool = PgPool::new(&database_url).unwrap();
        let result = pool.transaction(|conn| Ok(conn.batch_execute("select 1")?));

        assert!(result.is_ok());
    }
}
