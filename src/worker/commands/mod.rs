use anyhow::{Context, Result};
use songbird::id::GuildId;
use songbird::{Call, Songbird};
use std::num::NonZero;
use std::sync::Arc;

pub mod play;
pub mod connect;
pub mod stop;
pub mod pause;
pub mod resume;
pub mod skip;

pub async fn get_manager_call(
    guild_id: NonZero<u64>,
    manager: &mut Option<Arc<Songbird>>,
) -> Result<Arc<tokio::sync::Mutex<Call>>> {
    let h = manager
        .as_mut()
        .context("Manager not initialized")?
        .get(GuildId(
            guild_id
        ))
        .context("Failed to retrieve manager Call")?;
    Ok(h)
}