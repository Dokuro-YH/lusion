#![feature(async_await, await_macro)]

//! An experimental, Web API based on async/await IO implementation.

macro_rules! box_async {
    {$($t:tt)*} => {
        FutureObj::new(Box::new(async move { $($t)* }))
    };
}
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate juniper;

pub mod db;
pub mod error;
pub mod graphql;
pub mod middleware;
pub mod resp;
pub mod schema;
pub mod security;
pub mod validator;

#[cfg(test)]
mod test_helpers;
