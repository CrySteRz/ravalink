use std::sync::Arc;

use log::{debug, error};
use rdkafka::Message as KafkaMessage;
use anyhow::Result;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use songbird::Songbird;
use tokio::sync::broadcast::Sender;

use crate::utils::constants::KAFKA_SEND_TIMEOUT;
use crate::worker::types::{ProcessorIPC, ProcessorIPCData};
use crate::utils::config::CONFIG;
use ravalink_interconnect::protocol::{JobRequest, JobEvent, JobResponse};
use crate::utils::helpers::minutes_to_duration;

fn configure_kafka_ssl(mut kafka_config: ClientConfig) -> ClientConfig {
    let config = &CONFIG.kafka;
    if config.kafka_use_ssl.unwrap_or(false) {
        kafka_config
            .set("security.protocol", "ssl")
            .set(
                "ssl.ca.location",
                config.kafka_ssl_ca.as_deref().expect("Kafka CA Not Found"),
            )
            .set(
                "ssl.certificate.location",
                config.kafka_ssl_cert.as_deref().expect("Kafka Cert Not Found"),
            )
            .set(
                "ssl.key.location",
                config.kafka_ssl_key.as_deref().expect("Kafka Key Not Found"),
            );
    } else if config.kafka_use_sasl.unwrap_or(false) {
        kafka_config
            .set("security.protocol", "SASL_PLAINTEXT")
            .set("sasl.mechanisms", "PLAIN")
            .set(
                "sasl.username",
                config.kafka_username.as_deref().expect("Kafka Username Not Found"),
            )
            .set(
                "sasl.password",
                config.kafka_password.as_deref().expect("Kafka Password Not Found"),
            );
    }

    kafka_config
}

pub async fn initialize_producer(brokers: &str) -> FutureProducer {
    let kafka_config = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .clone();

    let kafka_config = configure_kafka_ssl(kafka_config);

    kafka_config.create().expect("Failed to create Producer")
}

pub async fn initialize_consume_generic(
    brokers: &str,
    ipc: &mut ProcessorIPC,
    songbird: Option<Arc<Songbird>>,
    group_id: &str,
    callback: impl Fn(JobRequest, Arc<Sender<ProcessorIPCData>>, Option<Arc<Songbird>>) -> Result<()> + Send + Sync,
    initialized_callback: impl Fn() -> () + Send + Sync,
) {
    let mut kafka_config = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .clone();

    let kafka_config = configure_kafka_ssl(kafka_config);

    let consumer: StreamConsumer = kafka_config.create().expect("Failed to create Consumer");

    consumer
        .subscribe(&[&CONFIG.kafka.kafka_topic])
        .expect("Can't subscribe to specified topic");

    initialized_callback();

    loop {
        match consumer.recv().await {
            Ok(m) => {
                if let Some(payload) = m.payload() {
                    match serde_json::from_slice::<JobRequest>(payload) {
                        Ok(parsed_message) => {
                            if let Err(e) = callback(
                                parsed_message,
                                Arc::clone(&ipc.sender),
                                songbird.clone(),
                            )
                            {
                                error!("Callback execution failed: {}", e);
                            }
                        }
                        Err(e) => error!("Failed to parse message: {}", e),
                    }
                } else {
                    error!("Received empty payload!");
                }
            }
            Err(e) => error!("Failed to receive message: {}", e),
        }
    }
}

pub async fn send_job_request(message: &JobRequest, topic: &str, producer: &FutureProducer) {
    let data = match serde_json::to_string(message) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to serialize JobRequest: {}", e);
            return;
        }
    };

    let record: FutureRecord<'_, String, String> = FutureRecord::to(topic).payload(&data);
    if let Err((e, _)) = producer.send(record, minutes_to_duration(KAFKA_SEND_TIMEOUT)).await {
        error!("Failed to send JobRequest: {}", e);
    } else {
        debug!("Sent JobRequest");
    }
}

pub async fn send_job_event(event: &JobEvent, topic: &str, producer: &FutureProducer) {
    let data = match serde_json::to_string(event) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to serialize JobEvent: {}", e);
            return;
        }
    };

    let record: FutureRecord<'_, String, String> = FutureRecord::to(topic).payload(&data);
    if let Err((e, _)) = producer.send(record, minutes_to_duration(KAFKA_SEND_TIMEOUT)).await {
        error!("Failed to send JobEvent: {}", e);
    } else {
        debug!("Sent JobEvent");
    }
}

pub async fn send_job_response(response: &JobResponse, topic: &str, producer: &FutureProducer) {
    let data = match serde_json::to_string(response) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to serialize JobResponse: {}", e);
            return;
        }
    };

    let record: FutureRecord<'_, String, String> = FutureRecord::to(topic).payload(&data);
    if let Err((e, _)) = producer.send(record, minutes_to_duration(KAFKA_SEND_TIMEOUT)).await {
        error!("Failed to send JobResponse: {}", e);
    } else {
        debug!("Sent JobResponse");
    }
}
