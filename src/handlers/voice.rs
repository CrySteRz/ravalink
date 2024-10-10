use std::num::NonZero;
use std::sync::Arc;
use rdkafka::producer::FutureProducer;
use songbird::events::EventHandler as VoiceEventHandler;
use songbird::{Event, EventContext};
use serenity::async_trait;
use tokio::sync::broadcast::Sender;
use log::error;
use tokio::sync::Mutex;

use crate::worker::types::{ServerIPCData, ServerEventType, ServerMessage};

pub struct TrackErrorNotifier {
    pub job_id: String,
    pub guild_id: NonZero<u64>,
    pub ipc: Arc<Sender<ServerIPCData>>,
    pub producer : Arc<Mutex<FutureProducer>>,
}

pub struct TrackEndNotifier {
    pub job_id: String,
    pub guild_id: NonZero<u64>,
    pub ipc: Arc<Sender<ServerIPCData>>,
    pub producer : Arc<Mutex<FutureProducer>>,
}

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                let error_message = format!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );

                let notification = self.ipc.send(ServerIPCData {
                    message: ServerMessage::Event(ServerEventType::TrackError {
                        error: error_message,
                    }),
                    guild_id: self.guild_id,
                    job_id: self.job_id.clone(),
                    producer : Some(self.producer.clone()),
                });

                if let Err(e) = notification {
                    error!(
                        "Failed to notify job: {} about track error. Error: {}",
                        self.job_id, e
                    );
                }
            }
        }

        None
    }
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let notification = self.ipc.send(ServerIPCData {
            message: ServerMessage::Event(ServerEventType::TrackEnded),
            guild_id: self.guild_id,
            job_id: self.job_id.clone(),
            producer : Some(self.producer.clone()),
        });


        match notification {
            Ok(_) => {
                println!(
                    "Notified job: {} that track has ended.",
                    self.job_id.to_string()
                );
            }
            Err(e) => {
                error!(
                    "Failed to notify job: {} that track has ended. Error: {}",
                    self.job_id.to_string(), e
                );
            }
        }

        None
    }
}
