#![feature(async_await, await_macro)]
use std::io;

use tide::error::{Error, ResultExt};
use tide::{App, Context};

async fn hello(ctx: Context<()>) -> Result<String, Error> {
    let name = ctx.param::<String>("name").client_err()?;
    Ok(format!("Hello, {}", name))
}

fn main() -> io::Result<()> {
    let mut app = App::new(());
    app.at("/*name").get(hello);
    Ok(app.serve("127.0.0.1:8000")?)
}
