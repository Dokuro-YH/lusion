use std::cmp;
use std::fs::{File, Metadata};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::marker::Unpin;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use bytes::Bytes;
use futures::{future::FutureObj, stream::Stream, task::Context, Poll};
use tide::middleware::{Middleware, Next};

use crate::response::{self, Response};

pub struct NamedFile {
    path: PathBuf,
    file: File,
    md: Metadata,
}

impl NamedFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let md = file.metadata()?;
        Ok(NamedFile { path, file, md })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn into_response(self) -> Response {
        let chunk = ChunkedReadFile {
            size: self.md.len(),
            offset: 0,
            file: self.file,
            counter: 0,
        };
        response::stream(http::StatusCode::OK, chunk)
    }
}

pub struct ChunkedReadFile {
    size: u64,
    offset: u64,
    file: File,
    counter: u64,
}

impl Stream for ChunkedReadFile {
    type Item = Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let size = self.size;
        let offset = self.offset;
        let counter = self.counter;
        let file = self.file.by_ref();

        if size == counter {
            Poll::Ready(None)
        } else {
            let max_bytes = cmp::min(size.saturating_sub(counter), 65_536) as usize;
            let mut buf = Vec::with_capacity(max_bytes);

            file.seek(SeekFrom::Start(offset))?;
            let n = file.take(max_bytes as u64).read_to_end(&mut buf)?;

            if n == 0 {
                return Poll::Ready(Some(Err(ErrorKind::UnexpectedEof.into())));
            }

            self.offset += n as u64;
            self.counter += n as u64;

            Poll::Ready(Some(Ok(Bytes::from(buf))))
        }
    }
}

impl Unpin for ChunkedReadFile {}

pub struct Static {
    path: String,
    directory: PathBuf,
}

impl Static {
    pub fn new<T: Into<PathBuf>>(path: &str, dir: T) -> Self {
        Self {
            path: path.to_owned(),
            directory: dir.into(),
        }
    }

    fn read_file(&self, path: &str) -> Result<Option<NamedFile>> {
        let buf = self.get_path_buf(path)?;
        let file_path = self.directory.join(&buf);

        if file_path.exists() && file_path.is_file() {
            return Ok(Some(NamedFile::open(file_path)?));
        }

        Ok(None)
    }

    fn get_path_buf(&self, path: &str) -> Result<PathBuf> {
        let mut buf = PathBuf::new();
        for segment in path.split('/') {
            if segment == ".." {
                buf.pop();
            } else if segment.starts_with('.') {
                return Err(Error::new(ErrorKind::Other, "bad segment start '.'"));
            } else {
                buf.push(segment);
            }
        }

        Ok(buf)
    }
}

impl<Data: Send + Sync + 'static> Middleware<Data> for Static {
    fn handle<'a>(
        &'a self,
        cx: tide::Context<Data>,
        next: Next<'a, Data>,
    ) -> FutureObj<'a, Response> {
        box_async! {
            let path = cx.uri().path();
            if path.starts_with(&self.path) {
                let file_path = &path[self.path.len()..];

                let res = match self.read_file(&file_path) {
                    Ok(file) => file
                        .map(|file| file.into_response())
                        .unwrap_or_else(|| response::empty(http::StatusCode::NOT_FOUND)),
                    Err(e) => {
                        log::debug!("Failed to read file: {}", e);
                        response::empty(http::StatusCode::INTERNAL_SERVER_ERROR)
                    }
                };

                return res;
            }

            await!(next.run(cx))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    fn app() -> tide::App<()> {
        let mut app = tide::App::new(());
        app.middleware(Static::new("/static", "./tests/resources"));

        app
    }

    #[test]
    fn test_static_middleware() {
        let mut server = init_service(app());
        let req = http::Request::get("/static/a.txt").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "aaa\n");

        let req = http::Request::get("/static/b.txt").to_request();
        let res = call_service(&mut server, req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.read_body(), "bbb\n");
    }

}
