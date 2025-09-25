use std::sync::{Arc};

use amqprs::connection::Connection;
use tokio::sync::Mutex;
use dotenv::dotenv;
use crate::{rabbitmq::RabbitVariables, rabbitmq_consumer::RabbitMqConsumer, rabbitmq_producer::RabbitMqProducer};

mod logger;
mod rabbitmq;
mod rabbitmq_producer;
mod rabbitmq_consumer;



#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let producer_variables: RabbitVariables = rabbitmq::initialize_variables("PRODUCER");
    let consumer_variables: RabbitVariables = rabbitmq::initialize_variables("CONSUMER");
    let connection_variables: RabbitVariables = producer_variables.clone();

    // Initial connection
    let mut connection: Arc<Connection> = rabbitmq::connect_rabbitmq(&producer_variables).await;
    
    
    let mut consumer: RabbitMqConsumer = RabbitMqConsumer::new(consumer_variables, connection.clone()).await;


    consumer.start_consuming().await;

    loop {
        if !connection.is_open() {
            connection = rabbitmq::connect_rabbitmq(&connection_variables).await;
            consumer.reset_connection(connection.clone());
        }
    }
}