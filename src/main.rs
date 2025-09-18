use std::{env, process::exit};
use dotenv::dotenv;
use std::path::PathBuf;
use log::LevelFilter;
use simplelog::{format_description, ColorChoice, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};

use crate::rabbitmq::{RabbitVariables};

mod rabbitmq;


#[tokio::main]
async fn main() {

    dotenv().ok();

    let path_res = env::current_exe();
    let mut path_full = match path_res {
        Ok(r) => r,
        Err(e) => {
            log::error!("Could not get the current executable path: {e}");
            exit(101);
        }
    };

    path_full.pop();
    let path_error: PathBuf = path_full.clone().join("Errors.log");

    let log_res: Result<(), log::SetLoggerError> = simplelog::CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            ConfigBuilder::new().set_time_format_custom(format_description!("[year]-[month]-[day] [hour]:[minute]:[second] +[offset_hour]")).build(),
            TerminalMode::Mixed,
            ColorChoice::Auto
        ),
        WriteLogger::new(
        LevelFilter::Warn,
        ConfigBuilder::new().set_time_format_custom(format_description!("[year]-[month]-[day] [hour]:[minute]:[second] +[offset_hour]")).build(),
        std::fs::OpenOptions::new().create(true).append(true).open(path_error).unwrap(),
        ),
    ]);
    match log_res {
        Ok(_) => (),
        Err(e) => {
            panic!("Could not start logger service: {e}");
        }
    }
    println!("The host is: {}", env::var("HOST").expect(&env_not_present("HOST")));
    

}


