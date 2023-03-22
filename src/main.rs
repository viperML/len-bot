use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::Arc;

use async_trait::async_trait;

use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Mutex;
use tracing::{debug, info};
use tracing::{span, Level};
use tracing::{trace, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_tree::HierarchicalLayer;

#[derive(Debug)]
struct Handler {
    msg_queue: Arc<Mutex<HashMap<ChannelId, VecDeque<Message>>>>,
    ai_handler: async_openai::Client,
}

impl Handler {
    fn new() -> Self {
        Self {
            msg_queue: Arc::new(Mutex::new(HashMap::new())),
            ai_handler: async_openai::Client::new(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} ready", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let _span = span!(Level::INFO, "msg");
        info!(?msg);


        let ai_input = match self.msg_queue.clone().lock() {
            Err(err) => {
                warn!(?err, "Mutex poisoned");
                None
            }
            Ok(mut msg_queue) => {
                match msg_queue.entry(msg.channel_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(VecDeque::from([msg.clone()]));
                        None
                    }
                    Entry::Occupied(mut entry) => {
                        let vec = entry.get_mut();
                        vec.push_back(msg.clone());
                        if vec.len() >= 10 {
                            vec.pop_front();
                        }
                        // let vec = *&vec;
                        build_msg(vec.clone())
                    }
                }
            }
        };

        if msg.author.bot {
            return;
        }

        if msg.content.contains("bot") {
            if let Some(ai_input) = ai_input {
                match self.ai_handler.chat().create(ai_input).await {
                    Err(err) => warn!(?err, "Didn't get openai response"),
                    Ok(responses) => {
                        info!(?responses, "Ok response");
                        for r in responses.choices {
                            match r.message.role {
                                async_openai::types::Role::Assistant => {
                                    msg.channel_id
                                        .say(&ctx.http, r.message.content)
                                        .await
                                        .unwrap();
                                }
                                _ => warn!("Didn't receive an assistant response"),
                            }
                        }
                    }
                };
            }
        };
    }
}

#[tracing::instrument]
fn build_msg<I: IntoIterator<Item = Message> + std::fmt::Debug>(
    stack: I,
) -> Option<async_openai::types::CreateChatCompletionRequest> {
    use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs};

    let msgs: Result<Vec<ChatCompletionRequestMessage>, _> = stack
        .into_iter()
        .map(|m| {
            ChatCompletionRequestMessageArgs::default()
                .role(async_openai::types::Role::User)
                .content(m.content)
                .build()
        })
        .collect();

    let context = vec![ChatCompletionRequestMessageArgs::default()
        .role(async_openai::types::Role::System)
        .content("Eres un bot muy útil en una sala de chat online. Estás en una conversación entre varios usuarios y tienes que responderlos")
        .build()
        .unwrap()];

    match msgs {
        Err(_) => return None,
        Ok(msgs) => async_openai::types::CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .messages(context)
            .messages(msgs)
            .build()
            .ok(),
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
