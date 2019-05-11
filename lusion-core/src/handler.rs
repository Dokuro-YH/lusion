use futures::future::Future;

pub trait Handler<E> {
    type Future: Future;

    fn handle(&self, event: E) -> Self::Future;
}

impl<F, E, Fut> Handler<E> for F
where
    F: Fn(E) -> Fut + Send + Sync + 'static,
    Fut: Future + Send + 'static,
{
    type Future = Fut;

    fn handle(&self, event: E) -> Self::Future {
        (self)(event)
    }
}
