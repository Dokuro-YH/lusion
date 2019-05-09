//! Lusion Database Library.

#[macro_use]
extern crate failure;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

pub mod error;
pub mod humans;
pub mod pg;
pub mod pool;
pub mod test;
pub mod users;

pub mod prelude {
    pub use crate::error::DbError;
    pub use crate::pg::{PgConn, PgPool};
    pub use crate::pool::DbPool;
}

mod schema;

#[cfg(test)]
mod test_helpers;
