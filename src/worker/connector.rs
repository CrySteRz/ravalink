use crate::utils::config::CONFIG;
use crate::utils::generic_connector::{initialize_producer, send_generic_message, initialize_consume_generic};

use crate::worker::types::ServerIPC;
use anyhow::Result;
use ravalink_interconnect::protocol::Message;
use log::error;
use rdkafka::producer::FutureProducer;
use songbird::Songbird;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use crate::worker::pool::WorkerPool;

pub static WORKER_PRODUCER: OnceLock<Mutex<Option<FutureProducer>>> = OnceLock::new();


pub async fn initialize_api(
    ipc: &mut ServerIPC,
    songbird: Option<Arc<Songbird>>,
    group_id: &String,
) {
    let broker = CONFIG.kafka.kafka_uri.to_string();
    let _x = CONFIG.clone();

    initialize_worker_consume(broker, ipc, songbird, group_id).await;
}

async fn parse_message_callback(
    message: Message,
    worker_pool: Arc<WorkerPool>,
    songbird: Option<Arc<Songbird>>,
) -> Result<()> {
    if let Some(producer_mutex) = WORKER_PRODUCER.get() {
        let producer_guard = producer_mutex.lock().await;
        if let Some(producer) = &*producer_guard {
            let producer = Arc::new(Mutex::new(producer.clone()));
            worker_pool.send_job(message, producer, songbird).await?;
        } else {
            error!("Kafka producer is not initialized");
        }
    } else {
        error!("Kafka producer is not initialized");
    }

    Ok(())
}


pub async fn initialize_worker_consume(
    brokers: String,
    ipc: &mut ServerIPC,
    songbird: Option<Arc<Songbird>>,
    group_id: &String,
) {
    let producer: FutureProducer = initialize_producer(&brokers).await;
    let _ = WORKER_PRODUCER.set(Mutex::new(Some(producer)));


    let worker_pool = Arc::new(WorkerPool::new(ipc));

    initialize_consume_generic(
        &brokers,
        ipc,
        songbird,
        group_id,
        |message, _sender, songbird| {
            let worker_pool = Arc::clone(&worker_pool);
            parse_message_callback(message, worker_pool, songbird)
        },
    )
    .await;
}


pub async fn send_message(message: &Message, topic: &str, producer: &mut FutureProducer) {
    send_generic_message(message, topic, producer).await;
}
