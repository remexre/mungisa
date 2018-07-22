extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate hostname;
extern crate hyper;
extern crate kankyo;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serenity;
#[macro_use]
extern crate structopt;
extern crate tokio_core;
extern crate tokio_timer;
extern crate void;

mod check;
mod get_hostname;
mod handler;
mod options;

use std::thread::spawn;
use std::time::{Duration, Instant};

use failure::{Error, SyncFailure};
use futures::{future::result, Future, Stream};
use hyper::Client as HyperClient;
use serenity::{
    client::{validate_token, Client}, framework::standard::StandardFramework,
};
use structopt::StructOpt;
use tokio_core::reactor::Core;
use tokio_timer::Interval;
use void::ResultVoidExt;

use check::check_website;
use handler::Handler;
use options::Options;

fn main() {
    kankyo::load().expect("Failed to load .env");
    let options = Options::from_args();
    options.start_logger();

    if let Err(err) = run(options) {
        error!("{}", err);
    }
}

fn run(options: Options) -> Result<(), Error> {
    let mut core = Core::new()?;
    let http_client = HyperClient::new(&core.handle());

    let hostname = core.run(options.hostname(&http_client)).void_unwrap();

    let token = options.discord_token();
    validate_token(&token).map_err(SyncFailure::new)?;

    let mut discord_client = Client::new(&token, Handler).map_err(SyncFailure::new)?;
    discord_client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .cmd("channel-id", handler::channel_id)
            .cmd("ping", handler::ping),
    );
    let listen = spawn(move || listen_thread(discord_client));

    let interval = Interval::new(Instant::now(), Duration::from_secs(1 * 60));
    let channel = options.channel_id();
    let website_url = options.url();
    core.run(interval.map_err(Error::from).for_each(|_| {
        // Broadcast typing, so we know when it's testing the site.
        channel.broadcast_typing().map_err(SyncFailure::new)?;

        let res = check_website(&http_client, &website_url)
            .map_err(|void| match void {})
            .and_then(|ok| {
                result(
                    if !ok {
                        channel
                            .send_message(|m| {
                                m.content(format!("[{}] The website looks down...", hostname))
                            })
                            .map(|_| ())
                            .map_err(SyncFailure::new)
                    } else {
                        Ok(())
                    }.map_err(Error::from),
                )
            })
            .wait();

        if let Err(err) = res {
            error!("When checking for uptime: {}", err);
        }
        Ok(())
    }))?;

    let () = listen.join().unwrap()?;
    Ok(())
}

fn listen_thread(mut client: Client) -> Result<(), Error> {
    client.start().map_err(SyncFailure::new)?;
    Ok(())
}
