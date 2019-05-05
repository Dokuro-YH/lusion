use diesel::connection::{Connection, TransactionManager};

use crate::{DbPool, Result};

/// A test connection pool.
pub struct TestPool<Pool>(Pool);

impl<Pool> TestPool<Pool>
where
    Pool: DbPool,
    Pool::Connection: Connection,
{
    pub fn with(pool: Pool) -> Self {
        TestPool(pool)
    }
}

impl<Pool> DbPool for TestPool<Pool>
where
    Pool: DbPool,
    Pool::Connection: Connection,
{
    type Connection = Pool::Connection;

    fn with<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Self::Connection) -> Result<T>,
    {
        self.0.with(|conn| {
            let transaction_manager = conn.transaction_manager();
            transaction_manager.begin_transaction(conn)?;
            let value = f(&conn)?;
            transaction_manager.rollback_transaction(conn)?;
            Ok(value)
        })
    }

    fn transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Self::Connection) -> Result<T>,
    {
        self.with(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg::PgPool;
    use diesel::connection::SimpleConnection;

    #[test]
    fn test_pool() {
        let database_url = dotenv::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres@localhost/lusion".to_owned());
        let pool = PgPool::new(&database_url).unwrap();
        let test_pool = TestPool::with(pool);
        let result = test_pool.transaction(|conn| Ok(conn.batch_execute("select 1")?));

        assert!(result.is_ok());
    }
}
