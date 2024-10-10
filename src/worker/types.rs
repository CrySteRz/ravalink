use std::{collections::VecDeque, fmt};
use rdkafka::producer::FutureProducer;
use songbird::tracks::TrackHandle;
use std::sync::Arc;
use tokio::sync::{broadcast::{self}, Mutex};
use std::num::NonZero;

#[derive(Clone, Debug)]
pub enum ServerEventType {
    TrackError { error: String },
    TrackEnded,
}

#[derive(Clone, Debug)]
pub enum ServerMessage{
    Event(ServerEventType),
}

#[derive(Clone)]
pub struct ServerIPCData{
    pub message: ServerMessage,
    pub guild_id: NonZero<u64>,
    pub job_id: String,
    pub producer: Option<Arc<Mutex<FutureProducer>>>,
}

// Manual Debug implementation
impl fmt::Debug for ServerIPCData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerIPCData")
            .field("message", &self.message)
            .field("guild_id", &self.guild_id)
            .field("job_id", &self.job_id)
            // You can either omit the producer or print a custom message instead
            .field("producer", &"FutureProducer omitted")
            .finish()
    }
}

pub struct ServerIPC {
    pub sender: Arc<broadcast::Sender<ServerIPCData>>,
    pub receiver: broadcast::Receiver<ServerIPCData>,
}

struct GuildQueue {
    track_queue: VecDeque<TrackHandle>,
    is_playing: bool,
}

impl GuildQueue {
    fn new() -> Self {
        GuildQueue {
            track_queue: VecDeque::new(),
            is_playing: false,
        }
    }

    fn add_track(&mut self, track: TrackHandle) {
        self.track_queue.push_back(track);
    }

    fn next_track(&mut self) -> Option<TrackHandle> {
        self.track_queue.pop_front()
    }
}