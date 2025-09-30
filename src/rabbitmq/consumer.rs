use std::{sync::{atomic::AtomicUsize, Arc}, thread::{self, ThreadId}};

use amqprs::{
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, Channel
    }, connection::Connection, consumer::AsyncConsumer, BasicProperties, Deliver
};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::rabbitmq::{self, common::RabbitVariables};


pub struct XmlConsumer {
    manual_ack: bool,
    producer_channels: Arc<Mutex<Vec<Channel>>>,
    total_consumed: Arc<AtomicUsize>,
    publish_args: BasicPublishArguments
}

pub struct RabbitMqConsumer {
    rabbit_variables: RabbitVariables,
    minio_bucket_name: String,
    connection: Arc<Connection>,
    consumer_channels: Vec<Channel>,
    producer_channels: Arc<Mutex<Vec<Channel>>>,
    total_consumed: Arc<AtomicUsize>,
}

impl RabbitMqConsumer {
    pub async fn new(rabbit_variables: RabbitVariables, minio_bucket_name: String, connection: Arc<Connection>) -> Self {

        let counter = Arc::new(AtomicUsize::new(0));
        Self {
            rabbit_variables: rabbit_variables,
            minio_bucket_name: minio_bucket_name,
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
        rabbitmq::common::initialize_channels(&self.rabbit_variables, &self.connection, &mut self.consumer_channels)
            .await;

        let mut producer_channels = self.producer_channels.lock().await;
        rabbitmq::common::initialize_channels(&self.rabbit_variables, &self.connection, &mut producer_channels).await;

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
            let publish_args: BasicPublishArguments = BasicPublishArguments::new(&self.rabbit_variables.queue_name, &self.rabbit_variables.consumer);
            let consume: XmlConsumer = XmlConsumer { 
                manual_ack: true,
                total_consumed: counter,
                producer_channels: producer_channels,
                publish_args: publish_args.clone()
            };
            let tag = channel.basic_consume(consume, args).await;

            match tag {
                Ok(content) => log::info!("Consumer connected with tag {}", content),
                Err(e) => log::info!("Failed to connect consumer: {}", e),
            }
        }
    }

    pub async fn reset_connection(&mut self, conn:Arc<Connection>) {
        self.connection = conn;
        self.consumer_channels.clear();
        self.producer_channels.lock().await.clear();
    }

}

impl XmlConsumer {
    async fn publish (&self, message: Vec<u8> ) -> bool {
        
        let channel = match self.get_publisher_channel().await {
            Some(ch) => ch,
            None => return false,
        };

        let json: String = "{\"body\": \"Hello, World!\"}".to_string();
        let result = channel.basic_publish(
            BasicProperties::default(), message, self.publish_args.clone()).await;

        match result {
            Ok(_) => return true,
            Err(e) => {
                log::error!("Failed to publish message: {}", e);
                return false;
            }
        }

        log::info!("Successfully published message");
        return true;
    }

    async fn get_publisher_channel(&self) -> Option<Channel> {
        loop {
            let mut channels = self.producer_channels.lock().await;
            if let Some(ch) = channels.iter_mut().find(|ch| ch.is_open()).cloned() {
                return Some(ch);
            }
            drop(channels); // libera o lock
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
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

        let total: usize = self.total_consumed.load(std::sync::atomic::Ordering::SeqCst);
        let current_thread: ThreadId = thread::current().id();
        log::info!("Consuming message: {} on thread {:?}", total, current_thread);

        self.publish(total).await;
        


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
