use std::{sync::Arc, time::Duration};

use amqprs::{
    self,
    channel::{self, BasicPublishArguments, Channel},
    connection::Connection, BasicProperties,
};
use dotenv::dotenv;
use serde_json::to_vec;
use tokio::time::sleep;

use crate::rabbitmq::common::{Message, RabbitVariables};

mod logger;
mod minio_client;
mod nfe;
mod nfe_parser;
mod nfes;
mod rabbitmq;

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let variables = rabbitmq::common::initialize_variables();
    let connection: Arc<Connection> = rabbitmq::common::connect_rabbitmq(&variables).await;

    // Inicializa um único channel
    let mut channel: Channel = rabbitmq::common::initialize_publish_channel(
        &"xml_queue".to_string(),
        &"xml_queue".to_string(),
        &String::new(),
        &connection,
    ).await.unwrap();

    let mut counter: i64 = 0;
    let mut message = Message {
        company_id: 0,
        org_id: 0,
        file: "1".to_string(),
    };

    let args = BasicPublishArguments::new(&String::new(), &String::from("xml_queue"));
    let props: BasicProperties = BasicProperties::default();

    let interval_ms: u64 = 1;
    loop {
        counter += 1;

        message.company_id = counter;
        message.org_id = counter * -1;
        message.file = counter.to_string();

        let mut content: Vec<u8> = serde_json::to_vec(&message).unwrap();

        channel.basic_publish(props.clone(), content, args.clone()).await.unwrap();

        log::info!("Message {counter} sent!");
        sleep(Duration::from_millis(interval_ms)).await;

        if counter >= 1000 {
            break;
        }
    }
}
