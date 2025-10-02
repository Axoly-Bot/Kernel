use std::{error::Error, sync::{atomic::{AtomicBool, Ordering}, Arc}};


use tokio::{signal, task::JoinHandle};
use twilight_gateway::{create_recommended, CloseFrame, Config, Intents, MessageSender, Shard, ShardId};
use twilight_http::Client as HttpCLient;
use Logger::Logger;
use RedisCache::Cache;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);
pub struct Kernel<T = ()> { // Tipo por defecto ()
    token: String,
    pub http: Arc<HttpCLient>,
    pub senders: Vec<MessageSender>,
    pub tasks: Vec<JoinHandle<T>>,
    pub shards: Box<dyn ExactSizeIterator<Item = Shard>>,
    pub cache: Cache,
}

impl<T> Kernel<T> {
    pub async fn new(token: String, intents: Intents) -> Result<Self, Box<dyn Error>> 
    where
        T: Send + 'static, // Requerido para JoinHandle
    {
        let http = Arc::new(HttpCLient::new(token.clone()));
        let config = Config::new(token.clone(), intents);

        let shards = create_recommended(&http, config, |_, builder| builder.build()).await?;

        let senders = Vec::with_capacity(shards.len());
        let tasks = Vec::with_capacity(shards.len());

        let cache = Cache::new().await;

        Ok(Self {
            token,
            http,
            senders: senders,
            tasks,
            shards: Box::new(shards.into_iter()),
            cache,
        })
    }

    pub async fn handle_close(&self){
        signal::ctrl_c().await.unwrap();
        SHUTDOWN.store(true, Ordering::Relaxed);

        for sender in &self.senders {
            // Ignore error if shard's already shutdown.
            _ = sender.close(CloseFrame::NORMAL);
        }

        for jh in &self.tasks {
            _ = jh;
        }
    }

    pub async fn get_logger(id: ShardId) -> Logger {
        Logger { 
            shard_id: id.number(), 
            name: "Axoly".to_string(),
            log_directory: "logs/".to_string()
        }
    }
}