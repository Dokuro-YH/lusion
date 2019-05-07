//! Error module.

pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as DieselError;

#[derive(Debug, Fail)]
pub enum DbError {
    #[fail(display = "diesel error: {}", _0)]
    Diesel(DieselError),

    #[fail(display = "pool error: {}", _0)]
    Pool(PoolError),
}

impl From<DieselError> for DbError {
    fn from(err: DieselError) -> Self {
        DbError::Diesel(err)
    }
}

impl From<PoolError> for DbError {
    fn from(err: PoolError) -> Self {
        DbError::Pool(err)
    }
}
