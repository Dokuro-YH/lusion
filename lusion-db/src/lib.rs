//! Lusion Database Library.

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

pub mod humans;
pub mod users;

mod error;
mod pg;
mod schema;

#[cfg(test)]
mod test_helpers;

pub use self::error::{Error, Result};
pub use self::pg::{PgConn, PgPool};
