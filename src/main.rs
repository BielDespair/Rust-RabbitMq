mod logger;
mod rabbitmq;
mod minio_client;

mod nfes;
mod impostos;
mod det;


use amqprs::connection::Connection;
use dotenv::dotenv;
use std::sync::{Arc};

use crate::{minio_client::MinioVariables, rabbitmq::{common::{connect_rabbitmq, initialize_variables, RabbitVariables}, consumer::RabbitMqConsumer}};


#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let minio_variables: MinioVariables = minio_client::initialize_variables();
    minio_client::init_client(&minio_variables);
    
    
    
    let producer_variables: RabbitVariables = initialize_variables("PRODUCER");
    let consumer_variables: RabbitVariables = initialize_variables("CONSUMER");
    let connection_variables: RabbitVariables = producer_variables.clone();

    // Initial connection
    let mut connection: Arc<Connection> = connect_rabbitmq(&producer_variables).await;
    
    
    let mut consumer: RabbitMqConsumer = RabbitMqConsumer::new(
        consumer_variables,
        minio_variables.bucket_name,
        connection.clone()).await;


    consumer.start_consuming().await;

    loop {
        if !connection.is_open() {
            connection = connect_rabbitmq(&connection_variables).await;
            consumer.reset_connection(connection.clone());
        }
    }
}