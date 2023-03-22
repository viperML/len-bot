use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::Arc;

use async_trait::async_trait;

use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Mutex;
use tracing::info;
use tracing::warn;
use tracing::{span, Level};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_tree::HierarchicalLayer;

#[derive(Debug)]
struct Handler {
    msg_queue: Arc<Mutex<HashMap<ChannelId, VecDeque<Message>>>>,
}

impl Handler {
    fn new() -> Self {
        Self {
            msg_queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} ready", ready.user.name);
    }

    async fn message(&self, _: Context, msg: Message) {
        let _span = span!(Level::INFO, "msg");
        info!(?msg);

        match self.msg_queue.clone().lock() {
            Err(err) => {
                warn!(?err, "Mutex poisoned");
            }
            Ok(mut guard) => {
                match guard.entry(msg.channel_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(VecDeque::from([msg]));
                    }
                    Entry::Occupied(mut entry) => {
                        let entry = entry.get_mut();
                        entry.push_back(msg);
                        if entry.len() >= 5 {
                            entry.pop_front();
                        }
                    }
                };
                info!(?guard);
            }
        };
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        .event_handler(Handler::new())
        .await?;

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
