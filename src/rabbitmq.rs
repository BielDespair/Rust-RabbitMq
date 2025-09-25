use amqprs::{
    BasicProperties, Deliver, callbacks,
    channel::{self, Channel},
    connection::{self, Connection},
};
use std::env;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RabbitVariables {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub queue_name: String,
    pub num_channels: u8,
    pub routing_key: String,
    pub exchange_name: String,
    pub consumer: String,
}

#[derive(serde::Serialize, Clone)]
pub struct Message {
    pub id: String,
    pub body: String,
}

#[inline]
fn env_not_present(var_name: &str) -> String {
    return format!("Enviroment variable '{}' not set", var_name);
}

pub fn initialize_rabbit_variables() -> RabbitVariables {
    let host = env::var("RABBIT_HOST").expect(&env_not_present("RABBIT_HOST"));
    let port: u16 = env::var("RABBIT_PORT").unwrap().parse().unwrap();
    let username = env::var("RABBIT_USER").expect(&env_not_present("RABBIT_USER"));
    let password = env::var("RABBIT_PASS").expect(&env_not_present("RABBIT_PASS"));
    let queue_name = env::var("RABBIT_QUEUE").expect(&env_not_present("RABBIT_QUEUE"));
    let num_channels: u8 = env::var("RABBIT_NUM_CHANNELS").unwrap().parse().unwrap();
    let routing_key = env::var("RABBIT_ROUTING_KEY").expect(&env_not_present("RABBIT_ROUTING_KEY"));
    let exchange_name = env::var("RABBIT_EXCHANGE").expect(&env_not_present("RABBIT_EXCHANGE"));
    let consumer = env::var("RABBIT_CONSUMER").expect(&env_not_present("RABBIT_CONSUMER"));

    RabbitVariables {
        host,
        port,
        username,
        password,
        queue_name,
        num_channels,
        routing_key,
        exchange_name,
        consumer,
    }
}

pub async fn connect_rabbitmq(rabbit_variables: &RabbitVariables) -> connection::Connection {
    let connection;
    loop {
        let result_connection =
            connection::Connection::open(&connection::OpenConnectionArguments::new(
                &rabbit_variables.host,
                rabbit_variables.port,
                &rabbit_variables.username,
                &rabbit_variables.password,
            ))
            .await;
        match result_connection {
            Ok(c) => {
                connection = c;
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

    connection
}

pub async fn initialize_channels(
    rabbit_variables: &RabbitVariables,
    connection: &Connection,
    channels: &mut Vec<Channel>,
) -> () {
    channels.clear();
    for _ in 0..rabbit_variables.num_channels {
        let channel = match connection.open_channel(None).await {
            Ok(r) => r,
            Err(e) => {
                log::error!("Could not open channel to RabbitMQ: {e}");
                return;
            }
        };

        match channel
            .register_callback(callbacks::DefaultChannelCallback)
            .await
        {
            Ok(_) => (),
            Err(e) => log::error!("Could not register channel callback: {e}"),
        }

        match channel
            .queue_declare(channel::QueueDeclareArguments::durable_client_named(
                &rabbit_variables.queue_name,
            ))
            .await
        {
            Ok(_) => (),
            Err(e) => {
                log::error!("Could not declare queue: {e}");
                return;
            }
        }

        if !rabbit_variables.exchange_name.is_empty() {
            match channel
                .queue_bind(channel::QueueBindArguments::new(
                    &rabbit_variables.queue_name,
                    &rabbit_variables.exchange_name,
                    &rabbit_variables.routing_key,
                ))
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Could not bind queue: {e}");
                    return;
                }
            }
        }
        channels.push(channel);
    }
}

pub async fn publish(
    content: &String,
    channel: &mut Channel,
    exchange_name: &String,
    routing_key: &String,
) -> bool {
    let args = channel::BasicPublishArguments::new(exchange_name, routing_key);

    let result = channel
        .basic_publish(
            BasicProperties::default(),
            content.as_bytes().to_vec(),
            args,
        )
        .await;

    match result {
        Ok(_) => {
            return true;
        }
        Err(e) => {
            log::error!("Could not publish message: {e}");
            return false;
        }
    }
}
