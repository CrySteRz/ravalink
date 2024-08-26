mod utils;
mod scheduler;
mod worker;
mod handlers;
mod initializers;
mod startup;
use crate::startup::start_rusty;


#[tokio::main]
async fn main() {

    start_rusty()
    
}