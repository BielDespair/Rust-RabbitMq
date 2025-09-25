use amqprs::{
    BasicProperties, Deliver,
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{
        BasicAckArguments, BasicConsumeArguments, Channel, QueueBindArguments,
        QueueDeclareArguments,
    },
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
};
use async_trait::async_trait;
use tokio::time::{self, sleep};

use crate::rabbitmq::{self, RabbitVariables};

pub struct XmlConsumer {
    manual_ack: bool,
}

pub struct RabbitMqConsumer {
    rabbit_variables: RabbitVariables,
    connection: Connection,
    channels: Vec<Channel>,
}

impl RabbitMqConsumer {
    pub async fn new(rabbit_variables: RabbitVariables) -> Self {
        let conn: Connection;
        let channels: Vec<Channel> = Vec::new();

        conn = rabbitmq::connect_rabbitmq(&rabbit_variables).await;
        Self {
            rabbit_variables: rabbit_variables,
            connection: conn,
            channels,
        }
    }

    pub async fn start_consuming(&mut self) {
        if self.connection.is_open() {
            self.initialize_channels().await;
            self.register_consuming_channels().await;
        }

        loop {
            if !self.connection.is_open() {
                log::error!("RabbitMQ Connection was closed");
                self.connection = rabbitmq::connect_rabbitmq(&self.rabbit_variables).await;
                self.initialize_channels().await;
                self.register_consuming_channels().await;
            }
            sleep(time::Duration::from_millis(1)).await;
        }
    }

    async fn initialize_channels(&mut self) {
        rabbitmq::initialize_channels(&self.rabbit_variables, &self.connection, &mut self.channels)
            .await;
    }

    async fn register_consuming_channels(&self) {
        for channel in self.channels.iter() {
            // Consumer tag deve vir de rabbit variables.
            let args: BasicConsumeArguments =
                BasicConsumeArguments::new(&self.rabbit_variables.queue_name, "parser-xml")
                    .manual_ack(true)
                    .finish();

            let consume: XmlConsumer = XmlConsumer { manual_ack: true };
            let tag = channel.basic_consume(consume, args).await;

            match tag {
                Ok(content) => log::info!("Consumer connected with tag {}", content),
                Err(e) => log::info!("Failed to connect consumer: {}", e),
            }
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

        log::info!("Consuming message: {}", returned_string);

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
