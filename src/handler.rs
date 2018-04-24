use serenity::prelude::EventHandler;
use serenity::utils::MessageBuilder;

pub struct Handler;

impl EventHandler for Handler {}

command!(channel_id(_context, message) {
    let msg = MessageBuilder::new()
        .push("The channel ID for ")
        .channel(message.channel_id)
        .push(" is ")
        .push_mono(message.channel_id)
        .build();
    message.reply(&msg)?;
});

command!(ping(_context, message) {
    info!("{:#?}", message);
    message.reply("pong")?;
});
