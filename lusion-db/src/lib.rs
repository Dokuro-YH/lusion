//! Lusion Database Library.

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
pub mod users;

mod schema;

#[cfg(test)]
mod test_helpers;

pub use self::error::{Error, Result};
pub use self::pg::{PgConn, PgPool};
