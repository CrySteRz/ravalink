use std::sync::Arc;

use log::{debug, error};
use rdkafka::Message as KafkaMessage;

use crate::utils::constants::KAFKA_SEND_TIMEOUT;
use crate::worker::types::{ProcessorIPC, ProcessorIPCData};
use anyhow::Result;
use async_fn_traits::{AsyncFn1, AsyncFn4};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use songbird::Songbird;
use tokio::sync::broadcast::Sender;
use crate::utils::config::Config;
use ravalink_interconnect::protocol::{JobRequest, JobEvent, JobResponse};
use crate::utils::helpers::minutes_to_duration;

fn configure_kafka_ssl(mut kafka_config: ClientConfig, config: &Config) -> ClientConfig {
    if config.kafka.kafka_use_ssl.unwrap_or(false) {
        kafka_config
            .set("security.protocol", "ssl")
            .set(
                "ssl.ca.location",
                config
                    .kafka
                    .kafka_ssl_ca
                    .clone()
                    .expect("Kafka CA Not Found"),
            )
            .set(
                "ssl.certificate.location",
                config
                    .kafka
                    .kafka_ssl_cert
                    .clone()
                    .expect("Kafka Cert Not Found"),
            )
            .set(
                "ssl.key.location",
                config
                    .kafka
                    .kafka_ssl_key
                    .clone()
                    .expect("Kafka Key Not Found"),
            );
    } else if config.kafka.kafka_use_sasl.unwrap_or(false) {
        kafka_config
            .set("security.protocol", "SASL_PLAINTEXT")
            .set("sasl.mechanisms", "PLAIN")
            .set(
                "sasl.username",
                config.kafka.kafka_username.as_ref().unwrap(),
            )
            .set(
                "sasl.password",
                config.kafka.kafka_password.as_ref().unwrap(),
            );
    }

    kafka_config
}

pub fn initialize_producer(brokers: &String, config: &Config) -> FutureProducer {
    let mut kafka_config = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .clone();

    kafka_config = configure_kafka_ssl(kafka_config, config);

    let producer: FutureProducer = kafka_config.create().expect("Failed to create Producer");

    producer
}

pub async fn initialize_consume_generic(
    brokers: &String,
    config: &Config,
    callback: impl AsyncFn4<
        JobRequest,
        Config,
        Arc<Sender<ProcessorIPCData>>,
        Option<Arc<Songbird>>,
        Output = Result<()>,
    >,
    ipc: &mut ProcessorIPC,
    initialized_callback: impl AsyncFn1<Config, Output = ()>,
    songbird: Option<Arc<Songbird>>,
    group_id: &String,
) {
    let mut kafka_config = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("security.protocol", "ssl")
        .clone();

    kafka_config = configure_kafka_ssl(kafka_config, config);

    let consumer: StreamConsumer = kafka_config.create().expect("Failed to create Consumer");

    consumer
        .subscribe(&[&config.kafka.kafka_topic])
        .expect("Can't subscribe to specified topic");

    initialized_callback(config.clone()).await; // Unfortunate clone because of Async trait

    loop {
        let msg = consumer.recv().await;
        match msg {
            Ok(m) => {
                let payload = m.payload();

                match payload {
                    Some(payload) => {
                        let parsed_message: Result<JobRequest, serde_json::Error> =
                            serde_json::from_slice(payload);

                        match parsed_message {
                            Ok(m) => {
                                let _parse = callback(
                                    m,
                                    config.clone(),
                                    ipc.sender.clone(),
                                    songbird.clone(),
                                )
                                .await; // More Unfortunate clones because of Async trait. At least most of these implement Arc so it's not the worst thing in the world
                            }
                            Err(e) => error!("Failed to parse message: {}", e),
                        }
                    }
                    None => {
                        error!("Received No Payload!");
                    }
                }
            }
            Err(e) => error!("Failed to receive message: {}", e),
        }
    }
}

pub async fn send_job_request(message: &JobRequest, topic: &str, producer: &mut FutureProducer) {
    let data = serde_json::to_string(message).unwrap();
    let record: FutureRecord<String, String> = FutureRecord::to(topic).payload(&data);
    producer.send(record, minutes_to_duration(KAFKA_SEND_TIMEOUT)).await.unwrap();
    debug!("Sent JobRequest");
}

pub async fn send_job_event(event: &JobEvent, topic: &str, producer: &mut FutureProducer) {
    let data = serde_json::to_string(event).unwrap();
    let record: FutureRecord<String, String> = FutureRecord::to(topic).payload(&data);
    producer.send(record, minutes_to_duration(KAFKA_SEND_TIMEOUT)).await.unwrap();
    debug!("Sent JobEvent");
}

pub async fn send_job_response(response: &JobResponse, topic: &str, producer: &mut FutureProducer) {
    let data = serde_json::to_string(response).unwrap();
    let record: FutureRecord<String, String> = FutureRecord::to(topic).payload(&data);
    producer.send(record, minutes_to_duration(KAFKA_SEND_TIMEOUT)).await.unwrap();
    debug!("Sent JobResponse");
}
