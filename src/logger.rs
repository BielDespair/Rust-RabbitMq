use std::{env, path::PathBuf, process::exit};

use log::LevelFilter;
use simplelog::{
    format_description, ColorChoice, ConfigBuilder, TermLogger, TerminalMode, WriteLogger
};

pub fn register_logger() {
    let path_res = env::current_exe();
    let mut path_full = match path_res {
        Ok(r) => r,
        Err(e) => {
            log::error!("Could not get the current executable path: {e}");
            exit(101);
        }
    };

    let level: LevelFilter = get_log_level_from_env();

    path_full.pop();
    let path_error: PathBuf = path_full.clone().join("Errors.log");

    let log_res: Result<(), log::SetLoggerError> = simplelog::CombinedLogger::init(vec![
        TermLogger::new(
            level,
            ConfigBuilder::new()
                .set_location_level(LevelFilter::Debug)
                .set_time_offset_to_local().unwrap_or_else(|e| e)
                .set_time_format_custom(format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second] +[offset_hour]"
                ))
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Warn,
            ConfigBuilder::new()
                .set_location_level(LevelFilter::Info)
                .set_time_offset_to_local().unwrap_or_else(|e| e)
                .set_time_format_custom(format_description!(
                    "[year]-[month]-[day] [hour]:[minute]:[second] +[offset_hour]"
                ))
                .build(),
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path_error)
                .unwrap(),
        ),
    ]);
    match log_res {
        Ok(_) => (),
        Err(e) => {
            panic!("Could not start logger service: {e}");
        }
    }
}


fn get_log_level_from_env() -> LevelFilter {
    match env::var("RUST_LOG") {
        Ok(level) => match level.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => LevelFilter::Info,
        },
        Err(_) => LevelFilter::Info,
    }
}