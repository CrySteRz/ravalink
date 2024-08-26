use crate::handlers::default::Handler;
use crate::utils::config::Config;
use crate::worker::types::ProcessorIPC;
use log::info;
use serenity::prelude::GatewayIntents;
use songbird::Config as SongbirdConfig;
use songbird::SerenityInit;
use std::sync::Arc;
use songbird::Songbird;
use crate::utils::helpers::initialize;
use nanoid::nanoid;


pub async fn initialize_songbird(
    config: &Config,
    _ipc: &mut ProcessorIPC,
) -> Option<Arc<Songbird>> {
    let intents = GatewayIntents::non_privileged();

    let mut client = serenity::Client::builder(&config.config.discord_bot_token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Failed to register Songbird Instance");

    let client_data = client.data.clone();

    tokio::spawn(async move {
        let _ = client
            .start_autosharded()
            .await
            .map_err(|why| println!("Client ended: {:?}", why));
    });

    info!("Songbird Initialized Successfully");

    let manager = {
        let data = client_data.read().await;
        data.get::<songbird::SongbirdKey>().cloned()
    };

    if let Some(manager) = manager {
        let mut config = SongbirdConfig::default();
        config.use_softclip = false;
        manager.set_config(config);
        Some(manager)
    } else {
        None
    }
}

// pub async fn initialize_scheduler(config: Config, ipc: &mut ProcessorIPC) {
//     info!("Scheduler INIT");
//     // Init server
//     initialize_api(&config, ipc, &nanoid!()).await;
// }

pub fn start_rusty() {
    initialize();
    log::info!("Server Starting...");
}