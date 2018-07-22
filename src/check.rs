use futures::{Async, Future};
use hyper::{
    client::{Connect, FutureResponse}, {Body, Client, Uri},
};
use void::Void;

pub fn check_website<C: Connect>(client: &Client<C, Body>, uri: &Uri) -> CheckFuture {
    info!("Checking that {} is up...", uri);
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
