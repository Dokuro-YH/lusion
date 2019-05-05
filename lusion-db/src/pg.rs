use diesel::connection::{Connection, TransactionManager};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use crate::error::Result;

pub type PgConn = PgConnection;

pub struct PgPool(Pool<ConnectionManager<PgConn>>);

embed_migrations!("./migrations");

impl PgPool {
    pub fn init(database_url: &str) -> Result<Self> {
        log::debug!("initialize database: {}", database_url);

        let conn = PgConn::establish(&database_url)?;

        embedded_migrations::run(&conn).expect("Failed to initialize database");
        let manager = ConnectionManager::<PgConn>::new(database_url);
        let pool = Pool::new(manager)?;
        Ok(PgPool(pool))
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

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;

    #[test]
    fn test_pg_pool() {
        let database_url = dotenv::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost/lusion".to_owned());
        let pool = PgPool::init(&database_url).unwrap();

        let result = pool.transaction(|conn| Ok(conn.batch_execute("select 1")?));

        assert_matches!(result, Ok(()));
    }
}
