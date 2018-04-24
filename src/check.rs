use futures::{Async, Future};
use hyper::{Body, Client, Uri};
use hyper::client::{Connect, FutureResponse};
use void::Void;

pub fn check_website<C: Connect>(client: &Client<C, Body>, uri: &Uri) -> CheckFuture {
    CheckFuture(client.get(uri.clone()))
}

pub struct CheckFuture(FutureResponse);

impl Future for CheckFuture {
    type Item = bool;
    type Error = Void;

    fn poll(&mut self) -> Result<Async<bool>, Void> {
        match self.0.poll() {
            Ok(Async::Ready(res)) => {
                let status = res.status();
                let ok = status.is_success() || status.is_redirection();
                Ok(Async::Ready(ok))
            }
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => {
                error!("{}", err);
                Ok(Async::Ready(false))
            }
        }
    }
}