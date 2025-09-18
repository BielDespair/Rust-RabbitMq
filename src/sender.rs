use amqprs::{
    callbacks,
    security::SecurityCredentials,
    connection::{OpenConnectionArguments, Connection},
};


pub struct AmqpModule {
    client: Client,
    channel: Channel,
}


impl AmqpModule {
    
}