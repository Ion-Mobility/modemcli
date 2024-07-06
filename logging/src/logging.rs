use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    append::rolling_file::{
        policy::compound::{
            roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy,
        },
        RollingFileAppender,
    },
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
use std::env;
use std::fs;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MyLogging {
    log_file_name: String,
    log_file_maxsize: u64,
    log_default_level: LevelFilter
}

impl Default for MyLogging {
    fn default() -> Self {
        MyLogging {
            log_file_name: "my_logging.log".to_owned(),
            log_file_maxsize: 10 * 1024 * 1024, // 10MB,
            log_default_level: LevelFilter::Info, // Default log level
        }
    }
}

impl MyLogging {
    pub fn new(&self, name: String, size: u64, level: LevelFilter) -> Self {
        MyLogging {
            log_file_name: name,
            log_file_maxsize: size,
            log_default_level: level
        }
    }

    // Function to create the default logging configuration
    pub fn set_default_log_setting(&self, log_level: LevelFilter) -> Config {
        // Console logger
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "[{d(%H:%M:%S%.3f)}][{f}:{L}][{l}] {m}{n}",
            )))
            .build();

        // Configure log4rs
        Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
            .logger(
                Logger::builder()
                    .build("app::requests", LevelFilter::Info),
            )
            .build(
                Root::builder()
                    .appender("stdout")
                    .build(log_level),
            )
            .unwrap()
    }

    pub fn init_logger(&self) {
        // Parsing if having runtime configure
        let args: Vec<String> = env::args().collect();
        let mut log_level = if args.contains(&"--debug".to_string()) {
            LevelFilter::Debug
        } else if args.contains(&"--info".to_string()) {
            LevelFilter::Info
        } else if args.contains(&"--warning".to_string()) {
            LevelFilter::Warn
        } else {
            //NO user setting indication
            LevelFilter::Off
        };

        // Attempt to initialize logger from log4rs.yml if it exists
        if fs::metadata("log4rs.yml").is_ok() {
            match log4rs::init_file("log4rs.yml", Default::default()) {
                Ok(_) => {
                    // Apply runtime log level if specified (overriding the config file)
                    if log_level != LevelFilter::Off {
                        println!(
                            "Logger initialized with log4rs.yml configuration with LOG LEVEL {:?}",
                            log::max_level()
                        );
                        println!("Argument log level: {:?} Run time can't affect (Solution: remove log4rs.yml or Update log Level inside", log_level);
                    } else {
                        println!(
                            "Logger initialized with log4rs.yml && log level: {:?}",
                            log::max_level()
                        );
                    }
                }
                Err(e) => {
                    println!("Failed to initialize logger from log4rs.yml: {} ==> config default log setting", e);
                    if log_level == LevelFilter::Off {
                        log_level = self.log_default_level;
                    }
                    let config = self.set_default_log_setting(log_level);
                    log4rs::init_config(config).unwrap();
                    println!(
                        "Logger initialized with default configuration and set log level: {:?}",
                        log_level
                    );
                }
            }
        } else {
            // Fallback to default configuration
            if log_level == LevelFilter::Off {
                log_level = self.log_default_level;
            }
            let config = self.set_default_log_setting(log_level);
            log4rs::init_config(config).unwrap();
            println!(
                "Logger initialized with default configuration and log level: {:?}",
                log_level
            );
        }
    }

}