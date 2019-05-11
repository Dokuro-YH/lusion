#![feature(async_await, await_macro)]

pub mod handler;
pub mod net;

pub mod prelude {
    pub use super::handler::Handler;
    pub use super::net::{self, NetServer, NetStream};
}
