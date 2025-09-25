use dotenv::dotenv;

use crate::rabbitmq_consumer::RabbitMqConsumer;

mod logger;
mod rabbitmq;
mod rabbitmq_consumer;
mod rabbitmq_consumer_old;

#[tokio::main]
async fn main() {
    dotenv().ok();
    logger::register_logger();

    let rabbit_variables = rabbitmq::initialize_rabbit_variables();
    let mut consumer = RabbitMqConsumer::new(rabbit_variables).await;

    // Loop infinito.
    consumer.start_consuming().await;
}
