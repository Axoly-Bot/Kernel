use std::{error::Error, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use logger::GenericLogger;
use tokio::{signal, task::JoinHandle};
use twilight_gateway::{create_recommended, CloseFrame, Config, Event, EventTypeFlags, Intents, MessageSender, Shard, ShardId, StreamExt};
use twilight_http::Client as HttpCLient;
use redis_cache::Cache;


static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub struct Kernel {
    token: String,
    pub http: Arc<HttpCLient>,
    pub senders: Vec<MessageSender>,
    pub tasks: Vec<JoinHandle<()>>,
    pub shards: Vec<Shard>, // Cambiamos a Vec para poder tomar posesión
    pub cache: Cache,
}

impl Kernel {
    pub async fn new(token: String, intents: Intents, redis_uri: String) -> Result<Self, Box<dyn Error>> {
        let http = Arc::new(HttpCLient::new(token.clone()));
        let config = Config::new(token.clone(), intents);

        let shards = create_recommended(&http, config, |_, builder| builder.build()).await?;

        let senders = Vec::with_capacity(shards.len());
        let tasks = Vec::with_capacity(shards.len());

        let cache = Cache::new(redis_uri).await;

        Ok(Self {
            token,
            http,
            senders,
            tasks,
            shards: shards.into_iter().collect(), // Convertimos a Vec
            cache,
        })
    }

    async fn handle_events(mut shard: Shard, _cache: Cache, _http: Arc<HttpCLient>) {
        // Aquí iría la lógica de manejo de eventos
        // Por ahora, para que compile, un loop simple que verifica SHUTDOWN
        while !SHUTDOWN.load(Ordering::Relaxed) {
            while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
                let event = match item {
                    Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
                    Ok(event) => event,
                    Err(source) => {
                        tracing::warn!(?source, "error receiving event");

                        continue;
                    }
                };

                // You'd normally want to spawn a new tokio task for each event and
                // handle the event there to not block the shard.
                tracing::debug!(?event, shard = ?shard.id(), "received event");
            }
        }
    }

    pub async fn run(&mut self) {
        // Tomamos posesión de los shards, reemplazando con un Vec vacío
        let shards = std::mem::take(&mut self.shards);
        let cache = self.cache.clone(); // Asumiendo que Cache implementa Clone
        let http = Arc::clone(&self.http);

        for shard in shards.into_iter() {
            let sender = shard.sender();
            self.senders.push(sender);

            let cache_clone = cache.clone();
            let http_clone = Arc::clone(&http);

            let task = tokio::spawn(async move {
                Self::handle_events(shard, cache_clone, http_clone).await;
            });
            self.tasks.push(task);
        }
    }

    pub async fn handle_close(&self) {
        signal::ctrl_c().await.unwrap();
        SHUTDOWN.store(true, Ordering::Relaxed);

        for sender in &self.senders {
            _ = sender.close(CloseFrame::NORMAL);
        }

        for jh in &self.tasks {
            _ = jh;
        }
    }

}

