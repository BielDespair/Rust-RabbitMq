mod logger;
mod rabbitmq;
mod minio_client;

mod nfe_parser;
mod nfes;
mod nfe;

use dotenv::dotenv;

use crate::{minio_client::MinioVariables, rabbitmq::{common::{initialize_variables, RabbitVariables}, consumer::RabbitMqConsumer}};


#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let minio_variables: MinioVariables = minio_client::initialize_variables();
    minio_client::init_client(&minio_variables);
    
    
    let consumer_variables: RabbitVariables = initialize_variables();

    
    let mut consumer: RabbitMqConsumer = RabbitMqConsumer::new(
        consumer_variables,
        minio_variables.bucket_name).await;


    consumer.start().await;
    
}