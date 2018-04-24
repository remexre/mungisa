use std::borrow::Cow;
use std::env::var;

use hyper::Uri;
use serenity::model::id::ChannelId;

#[derive(Debug, StructOpt)]
#[structopt(raw(global_setting = "::structopt::clap::AppSettings::ColoredHelp"))]
pub struct Options {
    /// The channel to message. Overrides the `CHANNEL_ID` environment
    /// variable.
    #[structopt(short = "c", long = "channel")]
    channel_id: Option<String>,

    /// The Discord API token. Overrides the `DISCORD_TOKEN` environment
    /// variable.
    #[structopt(long = "discord-token")]
    discord_token: Option<String>,

    /// Turns off message output.
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// The URL of the website to check. Overrides the `WEBSITE_URL`
    /// environment variable.
    #[structopt(long = "url")]
    url: Option<String>,

    /// Increases the verbosity. Default verbosity is errors only.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,
}

impl Options {
    /// Gets the ID of the channel to message from the environment variable
    /// `CHANNEL_ID` or from the `--channel-id` flag.
    pub fn channel_id(&self) -> ChannelId {
        let id = match self.channel_id {
            Some(ref id) => Cow::Borrowed(id),
            None => Cow::Owned(var("CHANNEL_ID").expect("Missing or invalid Channel ID")),
        };
        ChannelId(id.parse().expect("Invalid Channel ID"))
    }

    /// Gets the Discord API token from the environment variable
    /// `DISCORD_TOKEN` or from the `--discord-token` flag.
    pub fn discord_token(&self) -> String {
        self.discord_token
            .clone()
            .unwrap_or_else(|| var("DISCORD_TOKEN").expect("Missing or invalid Discord token"))
    }

    /// Sets up logging as specified by the `-q` and `-v` flags.
    pub fn start_logger(&self) {
        if !self.quiet {
            let r = ::stderrlog::new().verbosity(self.verbose).init();
            if let Err(err) = r {
                error!("Logging couldn't start: {}", err);
            }
        }
    }

    /// Gets the URL to check from the environment variable `WEBSITE_URL`, the
    /// `--url` flag, or the compiled-in default.
    pub fn url(&self) -> Uri {
        self.url
            .as_ref()
            .map(|s| Cow::Borrowed(s as &str))
            .or_else(|| var("WEBSITE_URL").ok().map(Cow::Owned))
            .unwrap_or(Cow::Borrowed("http://acm.umn.edu"))
            .parse()
            .expect("Invalid URL")
    }
}
