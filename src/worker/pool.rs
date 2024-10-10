use songbird::tracks::TrackHandle;
use songbird::Songbird;
use tokio::sync::{broadcast, mpsc, Mutex};
use std::sync::Arc;
use anyhow::Result;
use ravalink_interconnect::protocol::{Command, Event, EventType, Message, Response, ResponseType};
use log::{info, error, debug};
use rdkafka::producer::FutureProducer;
use crate::utils::config::CONFIG;
use crate::utils::helpers::get_timestamp;
use crate::worker::connector::send_message;
use crate::worker::commands::{connect, stop, play};
use tokio::sync::broadcast::{Sender, Receiver};
use reqwest::Client as HttpClient;

use super::types::{ServerEventType, ServerIPC, ServerIPCData, ServerMessage};

pub struct WorkerPool {
    job_sender: mpsc::Sender<(Message, Arc<Mutex<FutureProducer>>, Option<Arc<Songbird>>)>,
    
}

impl WorkerPool {
    pub fn new(ipc: &mut ServerIPC) -> Self {
        let (tx, mut rx) = mpsc::channel::<(Message, Arc<Mutex<FutureProducer>>, Option<Arc<Songbird>>)>(100);

        let sender = ipc.sender.clone();
        let mut receiver = ipc.sender.subscribe();

        tokio::spawn(async move {
            loop {
 
                tokio::select! {
                   
                    event = receiver.recv() => {
                        match event {
                            Ok(event) => {
                                if let Some(producer) = event.producer.clone() {
                                    tokio::spawn(Self::process_ipc(event.clone(), producer));
                                } else {
                                    error!("Received event without a valid producer: {:?}", event);
                                }
                                debug!("Received event: {:?}", event);
                            },
                            Err(e) => {
                                error!("Failed to receive event: {:?}", e);
                            }
                        }
                    },
                    Some((job, producer, manager)) = rx.recv() => {
                        tokio::spawn(Self::process_job(job, producer, manager, sender.clone()));
                    },
                    else => {
                        error!("Job channel closed");
                        break;
                    }
                }
            }
        });

        WorkerPool { job_sender: tx }
    }


    pub async fn send_job(&self, job: Message, producer: Arc<Mutex<FutureProducer>>, manager: Option<Arc<Songbird>>) -> Result<()> {
        self.job_sender.send((job, producer, manager)).await.map_err(|e| {
            error!("Failed to send job to worker pool: {}", e);
            anyhow::anyhow!("Failed to send job")
        })
    }

    async fn process_job(job: Message, producer: Arc<Mutex<FutureProducer>>, manager: Option<Arc<Songbird>>, ipc: Arc<Sender<ServerIPCData>>) {

        let client = HttpClient::new();
        let mut track: Option<TrackHandle> = None;

        match job {
            Message::Request(request) => {
                match request.command {
                    Command::Connect => {
                        if let Some(manager) = manager {
                            if let Err(e) = connect::run(&request, &mut Some(manager), ipc, producer.clone()).await {
                                error!("Failed to connect to voice channel: {:?}", e);
                                Self::send_response(Message::Response(Response {
                                    job_id: request.job_id.clone(),
                                    guild_id: request.guild_id.clone(),
                                    response_type: ResponseType::Failure { reason: (e.to_string()) },
                                    timestamp: request.timestamp,
                                }), producer).await;
                            } else {
                                Self::send_response(Message::Response(Response {
                                    job_id: request.job_id.clone(),
                                    guild_id: request.guild_id.clone(),
                                    response_type: ResponseType::Success,
                                    timestamp: request.timestamp,
                                }), producer).await;
                            }
                        }
                    }
                    Command::Search { query } => {
                        info!("Searching for: {}", query);
                        let search_results = vec![];
                        Self::send_response(Message::Response(Response {
                            job_id: request.job_id.clone(),
                            guild_id: request.guild_id.clone(),
                            response_type: ResponseType::SearchResults { tracks: search_results },
                            timestamp: request.timestamp,
                        }), producer).await;
                    }

                    Command::Play { ref url } => {
                        if let Some(manager) = manager {
                            match play::run(&request, &mut Some(manager), client.clone(), url.clone()).await {
                                Ok(track_handle) => {
                                    track = Some(track_handle);
                    
                                    Self::send_response(Message::Response(Response {
                                        job_id: request.job_id.clone(),
                                        guild_id: request.guild_id.clone(),
                                        response_type: ResponseType::Success,
                                        timestamp: request.timestamp,
                                    }), producer).await;
                                }
                                Err(e) => {
                                    error!("Failed to play track: {:?}", e);
                                    Self::send_response(Message::Response(Response {
                                        job_id: request.job_id.clone(),
                                        guild_id: request.guild_id.clone(),
                                        response_type: ResponseType::Failure { reason: (e.to_string()) },
                                        timestamp: request.timestamp,
                                    }), producer).await;
                                }
                            }
                        }
                    }
                    Command::Stop => {
                        if let Some(manager) = manager {
                            if let Err(e) = stop::run(&request, &mut Some(manager)).await {
                                error!("Failed to stop playback: {:?}", e);
                                Self::send_response(Message::Response(Response {
                                    job_id: request.job_id.clone(),
                                    guild_id: request.guild_id.clone(),
                                    response_type: ResponseType::Failure { reason: (e.to_string()) },
                                    timestamp: request.timestamp,
                                }), producer).await;
                            } else {
                                Self::send_response(Message::Response(Response {
                                    job_id: request.job_id.clone(),
                                    guild_id: request.guild_id.clone(),
                                    response_type: ResponseType::Success,
                                    timestamp: request.timestamp,
                                }), producer).await;
                            }
                        }
                    },
                    Command::Pause => todo!(),
                    Command::Resume => todo!(),
                    Command::SeekToPosition { position } => todo!(),
                    Command::SetVolume { volume } => todo!(),
                    Command::GetPlaylists => todo!(),
                    Command::AddToPlaylist { playlist_id, track } => todo!(),
                    Command::RemoveFromPlaylist { playlist_id, track } => todo!(),
                    Command::LoadPlaylist { playlist_id } => todo!(),
                    Command::ClearPlaylist { playlist_id } => todo!(),
                    Command::ShuffleQueue => todo!(),
                    Command::Skip => todo!(),
                    Command::Loop => todo!(),
                }
            }
        Message::Ping { id } => {
            Self::send_response(Message::Pong { id }, producer).await;
        }
        Message::Pong { id: _ } => {
            return;
        }
        _ => {
            debug!("Unknown or unhandled message: {:?}", job);
        }
        }
    }

    async fn process_ipc(event: ServerIPCData, producer: Arc<Mutex<FutureProducer>>) {
        match event.message {
            ServerMessage::Event(ServerEventType::TrackError { error }) => {
    
                Self::send_event(Message::Event(Event{
                    event_type: EventType::TrackError { error: (error) },
                    job_id: event.job_id.clone(),
                    guild_id: event.guild_id,
                    timestamp: get_timestamp()
                }), producer).await;
            },
            ServerMessage::Event(ServerEventType::TrackEnded) => {
                Self::send_event(Message::Event(Event{
                    event_type: EventType::TrackEnd,
                    job_id: event.job_id.clone(),
                    guild_id: event.guild_id,
                    timestamp: get_timestamp()
                }), producer).await;
            },
    
        }
    }

    async fn send_response(response: Message, producer: Arc<Mutex<FutureProducer>>) {
        let mut producer_guard = producer.lock().await;
        send_message(&response, &CONFIG.kafka.kafka_topic, &mut *producer_guard).await;
    }

    async fn send_event(event: Message, producer: Arc<Mutex<FutureProducer>>) {
        let mut producer_guard = producer.lock().await;
        send_message(&event, &CONFIG.kafka.kafka_topic, &mut *producer_guard).await;
    }
}