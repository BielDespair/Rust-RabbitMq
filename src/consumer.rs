use amqprs::{channel::{self, BasicConsumeArguments, Channel}, connection::Connection, consumer::DefaultConsumer};
use tokio::time::{self, sleep};
use dotenv::dotenv;

use crate::rabbitmq::Message;

mod logger;
mod rabbitmq;
mod rabbitmq_consumer;

#[tokio::main]
async fn main () {

    dotenv().ok();
    logger::register_logger();


    let rabbit_variables = rabbitmq::initialize_rabbit_variables();
    let mut connection: Connection = rabbitmq::connect_rabbitmq(&rabbit_variables).await;
    let mut channels: Vec<Channel> = Vec::new();
    rabbitmq::initialize_channels(&rabbit_variables, &connection, &mut channels).await;

    let queue_message: Message = Message {id: "-1".to_string(), body: "".to_string()};
    rabbitmq_consumer::register_consuming_channels(
        rabbitmq_consumer::queue_consume_function,
        &rabbit_variables.queue_name,
        &mut channels,
        queue_message).await;

    loop {
        if !connection.is_open() {
            log::error!("Connection was not opened.");
            connection = rabbitmq::connect_rabbitmq(&rabbit_variables).await;
            rabbitmq::initialize_channels((&rabbit_variables), &connection, &mut channels).await;

        }
        sleep(time::Duration::from_millis(1)).await;
    }
    //connect_rabbitmq(&rabbit_variables);
    
    //let args = BasicConsumeArguments::new(&rabbit_variables.queue_name, &rabbit_variables.consumer);

    //channel.basic_consume(DefaultConsumer::new(args.no_ack), args).await.unwrap();


}