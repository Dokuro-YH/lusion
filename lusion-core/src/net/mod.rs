mod server;
mod stream;

pub use self::server::NetServer;
pub use self::stream::NetStream;

pub mod prelude {
    pub use super::server::NetServer;
    pub use super::stream::NetStream;
}
