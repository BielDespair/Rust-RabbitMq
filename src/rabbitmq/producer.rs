use std::sync::Arc;
use amqprs::{channel::Channel, connection::Connection};
use tokio::sync::Mutex;
use crate::rabbitmq::{self, RabbitVariables};



pub struct RabbitMqProducer {
    exchange_name: String,
    routing_key: String,
    connection: Arc<Connection>,
    channels: Arc<Mutex<Vec<Channel>>>,
}


impl RabbitMqProducer {
    pub async fn new(rabbit_variables: RabbitVariables, connection: Arc<Connection>) -> Self {

        let channels: Arc<Mutex<Vec<Channel>>> = Arc::new(Mutex::new(Vec::new()));

        Self {
            exchange_name: rabbit_variables.exchange_name,
            routing_key: rabbit_variables.routing_key,
            connection: connection,
            channels,
        }
    }

    pub async fn publish(&self) -> bool {
        let channel: Channel = self.get_publisher_channel().await.expect("Could not get a channel to publish message");

        let json: String = "{\"body\": \"Hello, World!\"}".to_string();
        
        rabbitmq::publish(json, channel, &self.exchange_name, &self.routing_key).await;
        log::info!("Successfully published!");
        return true;
    }

    pub async fn reset_connection(&mut self, conn: Arc<Connection>) {
        self.connection = conn;
        self.channels.lock().await.clear();

    }


    // Pega um channel disponível
    pub async fn get_publisher_channel(&self)-> Option<Channel> {
        let mut channels = self.channels.lock().await;
        channels.iter_mut().find(|ch| ch.is_open()).cloned()
    }
}
