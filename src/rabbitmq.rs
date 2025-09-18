use std::env;
use amqprs::{callbacks, connection};
use time::Duration;
use tokio::time::sleep;

pub struct RabbitVariables {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub queue_name: String,
    pub num_channels: u8,
    pub routing_key: String,
    pub exchange_name: String
}

#[inline]
fn env_not_present(var_name: &str) -> String {
    return format!("Enviroment variable '{}' not set", var_name);
}

pub fn initialize_rabbit_variables() -> RabbitVariables {
    let host = env::var("RABBITMQ_SERVICE_HOST")
        .expect(&env_not_present("RABBITMQ_SERVICE_HOST"));

    let port: u16 = env::var("RABBITMQ_SERVICE_PORT")
        .expect(&env_not_present("RABBITMQ_SERVICE_PORT"))
        .parse()
        .expect(&env_not_present("RABBITMQ_SERVICE_PORT"));

    let username = env::var("RABBITMQ_SERVICE_USERNAME")
        .expect(&env_not_present("RABBITMQ_SERVICE_USERNAME"));

    let password = env::var("RABBITMQ_SERVICE_PASSWORD")
        .expect(&env_not_present("RABBITMQ_SERVICE_PASSWORD"));

    let queue_name = env::var("RABBITMQ_SERVICE_QUEUE_NAME")
        .expect(&env_not_present("RABBITMQ_SERVICE_QUEUE_NAME"));

    let num_channels: u8 = env::var("RABBITMQ_SERVICE_NUM_CHANNELS")
        .expect(&env_not_present("RABBITMQ_SERVICE_NUM_CHANNELS"))
        .parse()
        .expect(&env_not_present("RABBITMQ_SERVICE_NUM_CHANNELS"));

    let routing_key = env::var("RABBITMQ_SERVICE_ROUTING_KEY")
        .expect(&env_not_present("RABBITMQ_SERVICE_ROUTING_KEY"));

    let exchange_name = env::var("RABBITMQ_SERVICE_EXCHANGE")
        .expect(&env_not_present("RABBITMQ_SERVICE_EXCHANGE"));

    RabbitVariables {
        host,
        port,
        username,
        password,
        queue_name,
        num_channels,
        routing_key,
        exchange_name,
    }
}


async fn connect_rabbitmq(
    rabbit_variables: &RabbitVariables,
) -> connection::Connection {

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
                sleep(std::time::Duration::from_secs(5));
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