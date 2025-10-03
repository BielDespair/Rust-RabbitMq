use std::{error::Error, fs, result, sync::Arc, time::Duration};

use amqprs::{
    channel::{
        BasicAckArguments, BasicConsumeArguments, BasicPublishArguments, BasicRejectArguments, Channel
    }, connection::Connection, consumer::AsyncConsumer, BasicProperties, Deliver
};
use async_trait::async_trait;
use bytes::Bytes;
use tokio::{time::sleep};

use crate::{minio_client, nfe_parser::parse_nfe, rabbitmq::{self, common::{Message, RabbitVariables}}};


// Implementa async consumer
pub struct XmlConsumer {
    publish_args: BasicPublishArguments,
    publish_channel: Channel,
    bucket_name: String,
}

impl XmlConsumer {
    pub async fn new(variables: &RabbitVariables, bucket_name: &String, connection: &Connection) -> Result<XmlConsumer, Box<dyn Error>> {

        let args: BasicPublishArguments = BasicPublishArguments {
            exchange: variables.exchange.clone(),
            routing_key: variables.routing_key.clone(),
            mandatory: false,
            immediate: false
        };

        let channel: Channel = rabbitmq::common::initialize_publish_channel(&variables.publish_queue, &variables.routing_key, &variables.exchange, &connection).await?;

        Ok(Self {
            publish_args: args,
            publish_channel: channel,
            bucket_name: bucket_name.clone(),
        })
    }
}

pub struct RabbitMqConsumer {
    variables: RabbitVariables,
    minio_bucket_name: String,
    connection: Arc<Connection>,
    consumer_channels: Vec<Channel>,
}

impl RabbitMqConsumer {
    pub async fn new(variables: RabbitVariables, minio_bucket_name: String) -> Self {

        let connection: Arc<Connection> = rabbitmq::common::connect_rabbitmq(&variables).await;
        Self {
            variables: variables,
            minio_bucket_name: minio_bucket_name,
            connection: connection,
            consumer_channels: Vec::new(),
        }


    }

    pub async fn start(&mut self) {
        if self.connection.is_open() {
            self.initialize_channels().await;
            self.register_consuming_channels().await.ok();
        }

        loop {
            if !self.connection.is_open() {
                log::warn!("Connection closed, restarting...");
                match self.restart().await {
                    Ok(_) => log::info!("Restart successful."),
                    Err(e) => log::error!("Failed to restart: {}", e),
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn restart(&mut self) -> Result<(), Box<dyn Error>> {
        self.consumer_channels.clear();
        self.connection = rabbitmq::common::connect_rabbitmq(&self.variables).await;
        self.initialize_channels().await;
        self.register_consuming_channels().await?;
        Ok(())
    
    }

    async fn initialize_channels(&mut self) {
        match rabbitmq::common::initialize_channels(
            &self.variables.consume_queue,
            &self.variables.routing_key,
            &mut self.variables.exchange,
            self.variables.num_channels,
             &self.connection).await {
            Ok(v) => self.consumer_channels = v,
            Err(e) => {
                log::error!("Failed to initialize channels: {}", e);
            }
        }
    }

    async fn register_consuming_channels(&self) -> Result<(), Box<dyn Error>> {
        for channel in self.consumer_channels.iter() {
            // Consumer tag deve vir de rabbit variables.
            let args: BasicConsumeArguments =
                BasicConsumeArguments::new(
                    &self.variables.consume_queue,
                    "parser-xml")
                    .manual_ack(true).finish();
            

            let consume: XmlConsumer = XmlConsumer::new(&self.variables, &self.minio_bucket_name, &self.connection).await?;
            channel.basic_consume(consume, args).await?;
        }
        log::debug!("Successfully registered consuming channels");
        Ok(())
    }
}

impl XmlConsumer {
    async fn publish (&self, message: Vec<u8> ) -> bool {
        
        let result = self.publish_channel.basic_publish(
            BasicProperties::default(), message, self.publish_args.clone()).await;

        match result {
            Ok(_) =>  return true,
            
            Err(e) => {
                log::error!("Failed to publish message: {}", e);
                return false;
            }
        }
    }

    async fn reject_message(&self, channel: &Channel, deliver: Deliver) {
        let args: BasicRejectArguments = BasicRejectArguments::new(deliver.delivery_tag(), false);

        let result = channel.basic_reject(args).await;

        match result {
            Ok(_) => (),
            Err(e) => log::error!("Failed to reject message: {}", e)
        }
    }
}

impl Drop for XmlConsumer {
    fn drop(&mut self) {
        println!("XmlConsumer is being dropped!");
    }
}
#[async_trait]
impl AsyncConsumer for XmlConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, _basic_properties: BasicProperties, content: Vec<u8>) {
        
        //let current_thread: ThreadId = thread::current().id();
        log::debug!("Consuming on channel: {}", channel.channel_id());
        

        let content_json = match std::str::from_utf8(&content) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{}", e);
                return self.reject_message(&channel, deliver).await;
            }
        };

        let message: Message = match serde_json::from_str::<Message>(&content_json) {
            Ok(m) => m,
            Err(e) => {
                let content_str = String::from_utf8_lossy(&content);
                log::error!("Failed to decode message: {} | Content: {}", e, content_str);
                return self.reject_message(&channel, deliver).await;
            }
        };

        let result =  minio_client::download_object(&message.file, &self.bucket_name).await
            .map_err(|e| {log::error!("Failed to download MinIO object: {}", e);});
        let file: Bytes = match result {
            Ok(f) => f,
            Err(_) => return self.reject_message(channel, deliver).await
        };
      
        let result = parse_nfe(file, message.company_id, message.org_id)
            .map_err(|e| {log::error!("Failed: {}", e);});

        let json_bytes = match result {
            Ok(v) => v,
            Err(_) => return self.reject_message(&channel, deliver).await
        };

        let result: bool = self.publish(json_bytes).await;
        if !result {
            return self.reject_message(&channel, deliver).await;
        }
        

        let args: BasicAckArguments = BasicAckArguments::new(deliver.delivery_tag(), false);

        match channel.basic_ack(args).await {
            Ok(_) => {
                log::info!("Message processed successfully | file: {} | company_id: {} | org_id: {}", 
                message.file, message.company_id, message.org_id);
            }
            Err(e) => {
                log::error!("Could not ack message: {e}");
                return;
            }
        };
    }
}
