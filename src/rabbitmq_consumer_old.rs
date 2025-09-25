use amqprs::{
    BasicProperties, Deliver,
    channel::{self, Channel},
    consumer::{self, AsyncConsumer},
};
use async_trait::async_trait;

use crate::rabbitmq::{self, Message};

pub type ConsumerFunction = fn(&Channel, &Deliver, BasicProperties, &[u8], &Message) -> bool;

pub struct QueueConsumer {
    manual_ack: bool,
    consume_fn: ConsumerFunction,
    message: Message,
}

pub async fn register_consuming_channels(
    consume_fn: ConsumerFunction,
    queue_name: &str,
    channels: &mut [channel::Channel],
    queue_message: Message,
) {
    for channel in channels.iter() {
        // Consumer tag deve vir de rabbit variables.
        let args = channel::BasicConsumeArguments::new(queue_name, "parser-xml")
            .manual_ack(true)
            .finish();
        let consume = QueueConsumer {
            manual_ack: true,
            consume_fn,
            message: queue_message.clone(),
        };
        let tag = channel.basic_consume(consume, args).await;

        match tag {
            Ok(content) => log::info!("Consumer connected with tag {}", content),
            Err(e) => log::info!("Failed to connect consumer: {}", e),
        }
    }
}

#[async_trait]
impl AsyncConsumer for QueueConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let shall_continue = (self.consume_fn)(
            channel,
            &deliver,
            _basic_properties,
            &content,
            &self.message,
        );

        if shall_continue && self.manual_ack {
            //log::info!("ack to delivery {} on channel {}", deliver., channel);
            let args = channel::BasicAckArguments::new(deliver.delivery_tag(), false);

            match channel.basic_ack(args).await {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Could not send: {e}");
                    return;
                }
            };
        }
    }
}

pub fn queue_consume_function(
    _channel: &Channel,
    _deliver: &Deliver,
    _basic_properties: BasicProperties,
    content: &[u8],
    queue_message: &Message, // Pode ser o MinIO/A fila novamente
) -> bool {
    let returned_string = match std::str::from_utf8(content) {
        Ok(r) => r,
        Err(e) => {
            log::error!("The data received is not valid UTF-8: {e}");
            return false;
        }
    };

    log::info!("Consuming message: {}", returned_string);

    let returned = true; // Envio novamente para uma outra fila, por ex.

    return returned;
}
