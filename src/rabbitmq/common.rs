use amqprs::{
     callbacks,
    channel::{Channel, QueueBindArguments, QueueDeclareArguments},
    connection::{self, Connection}, FieldTable,
};
use serde::{Deserialize, Serialize};
use core::panic;
use std::{env, error::Error, sync::Arc};
use tokio::time::sleep;


#[derive(Clone)]
pub struct RabbitVariables {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pwd: String,
    pub consume_queue: String,
    pub publish_queue: String,

    pub routing_key: String,
    pub exchange: String,
    pub num_channels: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub org_id: i64,
    pub company_id: i64,
    pub file: String,
}


#[inline]
fn env_not_present(var_name: &str) -> String {
    return format!("Enviroment variable '{}' not set", var_name);
}

#[inline]
fn get_var(var: &str) -> String {
    env::var(var).expect(&env_not_present(&var))
}

pub fn initialize_variables() -> RabbitVariables {
    let n_channels: u8 = match get_var("NUM_CHANNELS").parse() {
        Ok(n) => {
            if n > 20 {
                panic!("Number of channels cannot exceed 20!")
            } else {n}
        }

        Err(e) => panic!("Invalid number of channels: {}", e)
    };

    let port: u16 = match get_var("RABBIT_PORT").parse() {
        Ok(p) => p,
        Err(e) => panic!("Invalid port: {}", e)
    };

    RabbitVariables {
        host: get_var("RABBIT_HOST"),
        port: port,
        user: get_var("RABBIT_USER"),
        pwd: get_var("RABBIT_PWD"),
        
        consume_queue: get_var("CONSUME_QUEUE"),        
        publish_queue: get_var("PUBLISH_QUEUE"),

        exchange: get_var("EXCHANGE"),
        routing_key: get_var("ROUTING_KEY"),

        num_channels: n_channels,
    }
}

pub async fn connect_rabbitmq(rabbit_variables: &RabbitVariables) -> Arc<connection::Connection> {
    let connection;
    loop {
        let result_connection =
            connection::Connection::open(&connection::OpenConnectionArguments::new(
                &rabbit_variables.host,
                rabbit_variables.port,
                &rabbit_variables.user,
                &rabbit_variables.pwd
            ))
            .await;
        match result_connection {
            Ok(c) => {
                connection = c;
                log::info!("Successfully connected to RabbitMQ!");
                break;
            }
            Err(e) => {
                log::error!(
                    "Could not connect to rabbitmq, {}, trying again in 5 second",
                    e
                );
                sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
    match connection
        .register_callback(callbacks::DefaultConnectionCallback)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            log::error!("Could not register connection callback: {e}");
        }
    };

    return Arc::new(connection);
}

pub async fn initialize_channels(queue: &String, routing_key: &String, exchange: &String, num_channels: u8, connection: &Connection) -> Result<Vec<Channel>, Box<dyn Error>>{
    let mut channels: Vec<Channel> = Vec::new();
    for _ in 0..num_channels {
        let channel: Channel = match initialize_consumer_channel(&queue, &routing_key, &exchange, &connection).await {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        channels.push(channel);
    }

    log::info!("Sucessfully initialized channels on queue {}", queue);
    Ok(channels)
}

pub async fn initialize_consumer_channel(
    queue: &String,
    routing_key: &String,
    exchange: &String,
    connection: &Connection,
) -> Result<Channel, Box<dyn Error>> {
    let channel: Channel = connection.open_channel(None).await?;
    channel.register_callback(callbacks::DefaultChannelCallback).await?;

    declare_dlx_exchange(&channel).await?;


    let mut table: FieldTable = FieldTable::new();
    table.insert("x-dead-letter-exchange".try_into()?,"dead_letter_exchange".try_into()?);
    table.insert("x-dead-letter-routing-key".try_into()?, "dead_letter_queue".try_into()?);

    let declare_args: QueueDeclareArguments = QueueDeclareArguments::durable_client_named(queue).arguments(table).finish();

    
    channel.queue_declare(declare_args).await?;

    if !exchange.is_empty() {
        channel.queue_bind(QueueBindArguments::new(queue, exchange, routing_key)).await?;
    }
    
    Ok(channel)
}

pub async fn initialize_publish_channel(
    queue: &String,
    routing_key: &String,
    exchange: &String,
    connection: &Connection,
) -> Result<Channel, Box<dyn Error>> {

    let channel: Channel = connection.open_channel(None).await?;
    channel.register_callback(callbacks::DefaultChannelCallback).await?;
    
    channel.queue_declare(QueueDeclareArguments::durable_client_named(queue)).await?;

    if !exchange.is_empty() {
        channel.queue_bind(QueueBindArguments::new(queue, exchange, routing_key)).await?;
    }
    
    Ok(channel)
}

async fn declare_dlx_exchange(channel: &Channel) -> Result<(), Box<dyn Error>> {
    let dlq_args: QueueDeclareArguments = QueueDeclareArguments::durable_client_named("dead_letter_queue").finish();

    channel.exchange_declare(amqprs::channel::ExchangeDeclareArguments::new("dead_letter_exchange", "direct")).await?;
    channel.queue_declare(dlq_args).await?;
    channel.queue_bind(QueueBindArguments::new("dead_letter_queue", "dead_letter_exchange", "dead_letter_queue")).await?;

    Ok(())
}