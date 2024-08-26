use std::collections::VecDeque;
use songbird::tracks::TrackHandle;
use std::fmt;
use std::sync::Arc;
use tokio::sync::broadcast::{Sender, Receiver};
use songbird::Songbird;
use ravalink_interconnect::errors::BotError;
use ravalink_interconnect::protocol::{JobRequest, JobRequestType};

#[derive(Clone, Debug)]
pub enum Infrastructure {
    CheckTime,
    TrackEnded,
}

#[derive(Clone, Debug)]
pub enum ProcessorIncomingAction {
    Infrastructure(Infrastructure),
    Actions(JobRequestType),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JobID {
    Global(),
    Specific(String),
}

impl fmt::Display for JobID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let JobID::Specific(s) = self {
            return write!(f, "{}", s);
        }
        write!(f, "GLOBAL")
    }
}

#[derive(Clone, Debug)]
pub struct ProcessorIPCData {
    pub action_type: ProcessorIncomingAction,
    pub songbird: Option<Arc<Songbird>>,
    pub job_request: Option<JobRequest>,
    pub error: Option<BotError>,
    pub job_id: JobID,
}

pub struct ProcessorIPC {
    pub sender: Arc<Sender<ProcessorIPCData>>,
    pub receiver: Receiver<ProcessorIPCData>,
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