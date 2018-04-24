extern crate failure;
extern crate futures;
extern crate hyper;
extern crate kankyo;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serenity;
extern crate stderrlog;
#[macro_use]
extern crate structopt;
extern crate tokio_core;
extern crate tokio_timer;
extern crate void;

mod check;
mod handler;
mod options;

use std::thread::spawn;
use std::time::{Duration, Instant};

use failure::{Error, SyncFailure};
use futures::{Future, Stream};
use futures::future::result;
use hyper::Client as HyperClient;
use serenity::client::{validate_token, Client};
use serenity::framework::standard::StandardFramework;
use structopt::StructOpt;
use tokio_core::reactor::Core;
use tokio_timer::Interval;

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

    let mut core = Core::new()?;
    let http_client = HyperClient::new(&core.handle());

    let interval = Interval::new(Instant::now(), Duration::from_secs(1 * 60));
    let channel = options.channel_id();
    let website_url = options.url();
    core.run(interval.map_err(Error::from).for_each(|_| {
        // Broadcast typing, so we know when it's testing the site.
        channel.broadcast_typing().map_err(SyncFailure::new)?;

        let res = check_website(&http_client, &website_url)
            .map_err(|void| match void {})
            .and_then(|ok| {
                result(if !ok {
                    channel.send_message(|m| m.content("The website looks down..."))
                        .map(|_| ())
                        .map_err(SyncFailure::new)
                } else {
                    Ok(())
                }.map_err(Error::from))
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
