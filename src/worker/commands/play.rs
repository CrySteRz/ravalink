use anyhow::Result;
use ravalink_interconnect::protocol::Request;
use songbird::input::YoutubeDl;
use songbird::tracks::TrackHandle;
use songbird::Songbird;
use std::fmt;
use std::sync::Arc;
use crate::worker::commands::get_manager_call;
use reqwest::Client;
use songbird::input::Compose;

#[derive(Debug)]
enum PlaybackError {
    MissingAudioURL,
}

impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlaybackError::MissingAudioURL => write!(f, "Missing Audio URL"),
        }
    }
}

impl std::error::Error for PlaybackError {}

pub async fn run(
    request: &Request,
    manager: &mut Option<Arc<Songbird>>,
    client: Client,
    url: String,
) -> Result<TrackHandle> {
    let handler_lock = get_manager_call(request.guild_id, manager).await?;
    let mut handler = handler_lock.lock().await;
    let mut source = YoutubeDl::new(client, url);

    match source.aux_metadata().await {
        Ok(metadata) => {
            println!("AuxMetadata: {:?}", metadata);
        }
        Err(e) => {
            eprintln!("Error fetching metadata: {:?}", e);
        }
    }

    let track_handle = handler.play_input(source.into());
    Ok(track_handle)
}