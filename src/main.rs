mod utils;
mod worker;
mod handlers;
mod startup;
mod state;
use crate::startup::start_rusty_server;

#[tokio::main]
async fn main() {
    start_rusty_server().await;
}
