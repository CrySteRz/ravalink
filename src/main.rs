mod utils;
mod scheduler;
mod worker;
mod handlers;
mod startup;
mod state;
use crate::startup::start_rusty;


#[tokio::main]
async fn main() {

    start_rusty().await;

}