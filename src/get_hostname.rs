use futures::{
    future::{ok, Either}, Future, Stream,
};
use hostname::get_hostname as hostname;
use hyper::{
    client::Connect, {Body, Client, Uri},
};
use serde_json::from_slice;
use void::Void;

pub fn get_hostname<C: Connect>(
    client: &Client<C, Body>,
) -> impl Future<Item = String, Error = Void> {
    if let Some(hostname) = hostname() {
        Either::A(ok(hostname))
    } else {
        let url: Uri = "http://ipinfo.io/".parse().unwrap();

        error!("Couldn't get hostname locally; getting from ipinfo.io...");
        Either::B(
            client
                .get(url)
                .and_then(|res| {
                    if res.status().is_success() {
                        Either::A(
                            res.body()
                                .map(|c| c.to_vec())
                                .concat2()
                                .and_then(|body| {
                                    from_slice(&body)
                                        .map(|res: IpInfoResponse| res.hostname)
                                        .or_else(|err| {
                                            error!(
                                                "When fetching hostname, got invalid response: {}",
                                                err
                                            );
                                            Ok("UNKNOWN".to_string())
                                        })
                                })
                                .or_else(|err| {
                                    error!("When fetching hostname, got error: {}", err);
                                    ok("UNKNOWN".to_string())
                                }),
                        )
                    } else {
                        error!("When fetching hostname, got status: {}", res.status());
                        Either::B(ok("UNKNOWN".to_string()))
                    }
                })
                .or_else(|err| {
                    error!("When fetching hostname, got error: {}", err);
                    ok("UNKNOWN".to_string())
                }),
        )
    }
}

#[derive(Debug, Deserialize)]
struct IpInfoResponse {
    pub hostname: String,
}
