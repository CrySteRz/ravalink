use anyhow::{Context, Result};
use log::error;
use ravalink_interconnect::protocol::Request;
use songbird::id::GuildId;
use songbird::tracks::TrackHandle;
use songbird::Songbird;
use std::fmt;
use std::sync::Arc;

#[allow(clippy::enum_variant_names)]
pub enum ChannelControlError {
    ManagerAcquisitionFailed,
    ChannelLeaveFailed,
}

impl fmt::Display for ChannelControlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelControlError::ManagerAcquisitionFailed => write!(f, "Failed to acquire manager"),
            ChannelControlError::ChannelLeaveFailed => write!(f, "Failed to leave channel"),
        }
    }
}

pub async fn run(
    request: &Request,
    manager: &mut Option<Arc<Songbird>>,

) -> Result<()> {
    manager
        .as_mut()
        .context(ChannelControlError::ManagerAcquisitionFailed.to_string())?
        .remove(GuildId(request.guild_id))
        .await
        .context(ChannelControlError::ChannelLeaveFailed.to_string())?;
    Ok(())
}
//Add track_handle