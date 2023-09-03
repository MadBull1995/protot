use log::LevelFilter;
use log4rs::append::{console::ConsoleAppender, file::FileAppender};
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::SchedulerError;

pub fn init() -> Result<(), SchedulerError> {
    let encoder = PatternEncoder::new("[{d(%Y%m%d %H:%M:%S%.3f)}][{h({l})}][{T}]{t}:{L} - {m}\n");
    let console_encoder = encoder.clone();

    let logfile = FileAppender::builder()
        .encoder(Box::new(encoder))
        .build("log/output.log")
        .map_err(|e| {
            SchedulerError::LoggerSetupError(format!("Error building file for log4rs: {:?}", e))
        })?;

    let console = ConsoleAppender::builder()
        .encoder(Box::new(console_encoder))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .appender("console")
                .build(LevelFilter::Debug),
        )
        .map_err(|e| {
            SchedulerError::LoggerSetupError(format!(
                "Error while building logger configs for log4rs: {:?}",
                e
            ))
        })?;

    log4rs::init_config(config).map_err(|e| {
        SchedulerError::LoggerSetupError(format!("Error init config for log4rs: {:?}", e))
    })?;

    Ok(())
}
