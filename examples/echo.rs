#![feature(async_await, await_macro)]

use futures::prelude::*;
use lusion_core::prelude::*;

fn main() -> std::io::Result<()> {
    let server = NetServer::new().connect_handler(echo);

    Ok(server.serve("0.0.0.0:1234")?)
}

async fn echo(socket: NetStream) -> std::io::Result<()> {
    let (mut reader, mut writer) = socket.split();
    await!(reader.copy_into(&mut writer))?;
    Ok(())
}
