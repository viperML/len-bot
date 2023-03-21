use std::env;

use async_trait::async_trait;

use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::info;
use tracing::log::warn;
use tracing::{event, span, Level};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_tree::HierarchicalLayer;

#[derive(Debug)]
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let span = span!(Level::INFO, "msg");

        info!(?msg);
        // if msg.content == "!ping" {
        //     // Sending a message can fail, due to a network error, an
        //     // authentication error, or lack of permissions to post in the
        //     // channel, so log to stdout when some error happens, with a
        //     // description of it.
        //     if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
        //         println!("Error sending message: {:?}", why);
        //     }
        // }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let subscriber = Registry::default().with(HierarchicalLayer::new(2));
    // tracing::subscriber::set_global_default(subscriber).unwrap();
    let layer = HierarchicalLayer::default()
        .with_writer(std::io::stdout)
        .with_indent_lines(true)
        .with_indent_amount(2)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_verbose_exit(false)
        .with_verbose_entry(false)
        .with_targets(true);

    let subscriber = Registry::default()
        .with(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
                .add_directive("serenity=warn".parse()?),
        )
        .with(layer);

    tracing::subscriber::set_global_default(subscriber)?;

    let token = env::var("DISCORD_TOKEN")?;

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
