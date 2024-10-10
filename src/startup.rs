use crate::handlers::default::Handler;
use crate::utils::config::CONFIG;
use log::info;
use serenity::prelude::GatewayIntents;
use songbird::Config as SongbirdConfig;
use songbird::SerenityInit;
use std::sync::Arc;
use songbird::Songbird;
use crate::utils::helpers::initialize;
use nanoid::nanoid;
use crate::state::{initializer::StateClient, manager::State};
use crate::worker::connector::initialize_api;
use crate::worker::types::{ServerIPCData, ServerIPC};
use tokio::sync::broadcast::{Sender, Receiver};
use tokio::sync::broadcast;

// pub async fn initialize_state() -> State {
//     let redis_url = &CONFIG.redis_url;
//     let state_client = StateClient::new(redis_url)
//         .expect("Failed to initialize Redis client");
//     let state = State::new(state_client);
//     info!("STATE initialized successfully");
    
//     state
// }

pub async fn initialize_songbird(
    _ipc: &mut ServerIPC,
) -> Option<Arc<Songbird>> {
    
    let intents = GatewayIntents::non_privileged();
    let mut client = serenity::Client::builder(&CONFIG.config.discord_bot_token, intents)
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

pub async fn initialize_worker_pool (ipc: &mut ServerIPC) {
    info!("Worker Pool Initialized");
    let songbird = initialize_songbird(ipc).await;
    initialize_api(ipc, songbird, &nanoid!()).await;
}

pub async fn initialize_ipc() -> ServerIPC {
    let (tx_processor, _) = broadcast::channel(16);
    let worker_rx = tx_processor.subscribe();
    let tx_main = Arc::new(tx_processor);
    let worker_ipc = ServerIPC {
        sender: tx_main.clone(),
        receiver: worker_rx,
    };

    return worker_ipc;
}


pub async fn start_rusty_server() {
    initialize().await;
    // initialize_state().await;
    let mut ipc = initialize_ipc().await;
    initialize_worker_pool(&mut ipc).await;

    log::info!("Server Starting...");
}

