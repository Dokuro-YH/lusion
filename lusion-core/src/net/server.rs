use crate::handler::Handler;
use crate::net::NetStream;

use futures::executor::{self, ThreadPool};
use futures::future::Future;
use futures::stream::StreamExt;
use futures::task::SpawnExt;
use romio::tcp::TcpListener;

use std::io;
use std::net::ToSocketAddrs;
use std::sync::Arc;

pub struct NetServer<H> {
    pool_size: usize,
    connect_handler: Option<Arc<H>>,
}

impl<H> NetServer<H>
where
    H: Handler<NetStream> + Send + Sync + 'static,
    H::Future: Future<Output = io::Result<()>> + Send + 'static,
{
    pub fn new() -> Self {
        Self {
            pool_size: num_cpus::get(),
            connect_handler: None,
        }
    }

    pub fn connect_handler(mut self, h: H) -> Self {
        self.connect_handler = Some(Arc::new(h));
        self
    }

    pub fn serve<A: ToSocketAddrs>(mut self, addr: A) -> io::Result<()> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(io::ErrorKind::InvalidInput)?;
        let connect_handler = self
            .connect_handler
            .take()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "connect handler must be set"))?;

        executor::block_on(async {
            let mut threadpool = ThreadPool::builder().pool_size(self.pool_size).create()?;
            let mut listener = TcpListener::bind(&addr)?;
            let mut incoming = listener.incoming();

            while let Some(stream) = await!(incoming.next()) {
                let stream = stream.map(NetStream::new)?;
                let connect_handler = connect_handler.clone();
                threadpool
                    .spawn(async move {
                        match await!(connect_handler.handle(stream)) {
                            Ok(()) => {}
                            Err(e) => log::error!("connect handler error: {:?}", e),
                        }
                    })
                    .map_err(|e| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Thread pool execute error: {:?}", e),
                        )
                    })?;
            }
            Ok(())
        })
    }
}
