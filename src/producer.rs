use amqprs::{self, channel::Channel, connection::Connection};
use dotenv::dotenv;
use std::{
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
    time::Duration,
};

use ::futures::future::join_all;
use tokio::{
    task::futures,
    time::{self, sleep},
};

use crate::rabbitmq::{Message, RabbitVariables};

mod logger;
mod rabbitmq;

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    // Inicializa variáveis e conexão
    let rabbit_variables: RabbitVariables = rabbitmq::initialize_variables("PRODUCER");
    let connection: Arc<Connection> = rabbitmq::connect_rabbitmq(&rabbit_variables).await;

    // Inicializa um único channel
    let mut channels: Vec<Channel> = Vec::new();
    rabbitmq::initialize_channels(&rabbit_variables, &connection, &mut channels).await;
    let mut channel: Channel = channels.into_iter().next().expect("No channel initialized");

    let mut counter: i32 = 0;

    loop {
        let message = Message {
            id: counter.to_string(),
            body: format!("Message number {}", counter),
        };
        let json = serde_json::to_string(&message).unwrap();

        rabbitmq::publish(json, channel.clone(), &rabbit_variables.exchange_name, &rabbit_variables.routing_key)
            .await;

        log::info!("Published message n° {}", counter);
        counter += 1;

        sleep(Duration::from_millis(1)).await;

        if counter >= 1000 {
            break;
        }
    }
}