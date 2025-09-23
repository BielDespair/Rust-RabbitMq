use std::{process::exit, sync::{atomic::{AtomicI32, Ordering}, Arc}, time::Duration};
use amqprs::{self, channel::Channel, connection::Connection};
use dotenv::dotenv;

use ::futures::future::join_all;
use tokio::{task::futures, time::{self, sleep}};

use crate::rabbitmq::{Message, RabbitVariables};

mod logger;
mod rabbitmq;

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let rabbit_variables = rabbitmq::initialize_rabbit_variables();
    let mut connection: Connection = rabbitmq::connect_rabbitmq(&rabbit_variables).await;
    let mut channels: Vec<Channel> = Vec::new();
    rabbitmq::initialize_channels(&rabbit_variables, &connection, &mut channels).await;

    // spawn uma task por channel
    let mut tasks = Vec::new();

    let counter = Arc::new(AtomicI32::new(0));

    for channel in channels.into_iter() {
        let rv = rabbit_variables.clone();
        let counter = counter.clone();

        tasks.push(tokio::spawn(async move {
            run_publisher(channel, rv, counter).await;
        }));
    }

    // espera todas as tasks (no caso, rodarão indefinidamente)
    join_all(tasks).await;

    
}


async fn run_publisher(mut channel: Channel, rabbit_variables: RabbitVariables, counter: Arc<AtomicI32>) {
    loop {
        let id = counter.fetch_add(1, Ordering::SeqCst); // incrementa e pega valor anterior
        let message = Message {
            id: id.to_string(),
            body: format!("Message number {}", id),
        };
        let json = serde_json::to_string(&message).unwrap();

        let _ = rabbitmq::publish(&json, &mut channel, &rabbit_variables.exchange_name, &rabbit_variables.routing_key).await;
        log::info!("Published message n° {}", id);
        sleep(Duration::from_millis(1)).await;

        if id >= 1000 {
            break;
        }
    }
}