use crate::worker::types::ServerIPCData;
use anyhow::{Context, Result};
use ravalink_interconnect::protocol::Request;
use rdkafka::producer;
use rdkafka::producer::FutureProducer;
use songbird::events::TrackEvent;
use songbird::id::ChannelId;
use songbird::id::GuildId;
use songbird::Songbird;
use tokio::sync::Mutex;
use std::fmt;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use crate::handlers::voice::{TrackEndNotifier, TrackErrorNotifier};

#[allow(clippy::enum_variant_names)]
pub enum ChannelControlError {
    ManagerAcquisitionFailed
}

impl fmt::Display for ChannelControlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelControlError::ManagerAcquisitionFailed => write!(f, "Failed to acquire manager"),
        }
    }
}

pub async fn run(
    request: &Request,
    manager: &mut Option<Arc<Songbird>>,
    ipc: Arc<Sender<ServerIPCData>>,
    producer : Arc<Mutex<FutureProducer>>
) -> Result<()> {
    let gid = request.guild_id;
    let vcid = request.voice_channel_id.expect("Voice Channel not provided");

    if let Ok(handler_lock) = manager
        .as_mut()
        .context(ChannelControlError::ManagerAcquisitionFailed.to_string())?
        .join(GuildId(gid), ChannelId(vcid))
        .await
    {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(
            TrackEvent::Error.into(),
            TrackErrorNotifier {
                job_id: request.job_id.clone(),
                guild_id: request.guild_id.clone(),
                ipc : ipc.clone(),
                producer : producer.clone(),

            },
        );
        handler.add_global_event(TrackEvent::End.into(), TrackEndNotifier {
            job_id: request.job_id.clone(),
            guild_id: request.guild_id.clone(),
            ipc: ipc.clone(),
            producer : producer.clone(),
        });
    }
    Ok(())
}