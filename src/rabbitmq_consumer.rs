use std::{sync::{atomic::AtomicUsize, Arc}, thread::{self, ThreadId}};

use amqprs::{
    BasicProperties, Deliver,
    channel::{
        BasicAckArguments, BasicConsumeArguments, Channel,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
};
use async_trait::async_trait;
use tokio::{sync::Mutex, time::{self, sleep}};

use crate::{rabbitmq::{self, RabbitVariables}, rabbitmq_producer::RabbitMqProducer};

pub struct XmlConsumer {
    manual_ack: bool,
    producer_channels: Arc<Mutex<Vec<Channel>>>,
    exchange_name: String,
    routing_key: String,
    total_consumed: Arc<AtomicUsize>,
}

pub struct RabbitMqConsumer {
    rabbit_variables: RabbitVariables,
    connection: Arc<Connection>,
    consumer_channels: Vec<Channel>,
    producer_channels: Arc<Mutex<Vec<Channel>>>,
    total_consumed: Arc<AtomicUsize>,
}

impl RabbitMqConsumer {
    pub async fn new(rabbit_variables: RabbitVariables, connection: Arc<Connection>) -> Self {

        let counter = Arc::new(AtomicUsize::new(0));
        Self {
            rabbit_variables: rabbit_variables,
            connection: connection,
            consumer_channels: Vec::new(),
            producer_channels: Arc::new(Mutex::new(Vec::new())),
            total_consumed: counter
        }
    }

    pub async fn start_consuming(&mut self) {
        if self.connection.is_open() {
            self.initialize_channels().await;
            self.register_consuming_channels().await;
        }
    }

    async fn initialize_channels(&mut self) {
        rabbitmq::initialize_channels(&self.rabbit_variables, &self.connection, &mut self.consumer_channels)
            .await;

        let mut producer_channels = self.producer_channels.lock().await;
        rabbitmq::initialize_channels(&self.rabbit_variables, &self.connection, &mut producer_channels).await;

    }

    async fn register_consuming_channels(&self) {
        for channel in self.consumer_channels.iter() {
            // Consumer tag deve vir de rabbit variables.
            let args: BasicConsumeArguments =
                BasicConsumeArguments::new(&self.rabbit_variables.queue_name, "parser-xml")
                    .manual_ack(true)
                    .finish();

            let counter = self.total_consumed.clone();
            let producer_channels: Arc<Mutex<Vec<Channel>>> = self.producer_channels.clone();
            let consume: XmlConsumer = XmlConsumer { 
                manual_ack: true,
                total_consumed: counter,
                producer_channels,
                exchange_name: self.rabbit_variables.exchange_name.clone(),
                routing_key: self.rabbit_variables.routing_key.clone()
            };
            let tag = channel.basic_consume(consume, args).await;

            match tag {
                Ok(content) => log::info!("Consumer connected with tag {}", content),
                Err(e) => log::info!("Failed to connect consumer: {}", e),
            }
        }
    }
    pub async fn get_publisher_channel(&self)-> Option<Channel> {
        let mut channels = self.producer_channels.lock().await;
        channels.iter_mut().find(|ch| ch.is_open()).cloned()
    }

    pub async fn reset_connection(&mut self, conn:Arc<Connection>) {
        self.connection = conn;
        self.consumer_channels.clear();
        self.producer_channels.lock().await.clear();
    }

}

impl XmlConsumer {
    async fn publish (&self) -> bool {
        
        let channel = self.get_publisher_channel().await.expect("Failed to get publishing channel");
        let json: String = "{\"body\": \"Hello, World!\"}".to_string();
        
        rabbitmq::publish(json, channel, &self.exchange_name, &self.routing_key).await;
        log::info!("Successfully published!");
        return true;
    }

    async fn get_publisher_channel(&self)-> Option<Channel> {
        let mut channels = self.producer_channels.lock().await;
        channels.iter_mut().find(|ch| ch.is_open()).cloned()
    }
}
#[async_trait]
impl AsyncConsumer for XmlConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let returned_string = match std::str::from_utf8(&content) {
            Ok(r) => r,
            Err(e) => {
                log::error!("The data received is not valid UTF-8: {e}");
                return;
            }
        };

        self.total_consumed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);


        let current_thread: ThreadId = thread::current().id();
        log::info!("Consuming message: {} on thread {:?}", self.total_consumed.load(std::sync::atomic::Ordering::SeqCst), current_thread);

        self.publish();
        


        let args = BasicAckArguments::new(deliver.delivery_tag(), false);
        match channel.basic_ack(args).await {
            Ok(_) => (),
            Err(e) => {
                log::error!("Could not send: {e}");
                return;
            }
        };
    }
}
