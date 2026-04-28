use std::{
    fmt::Write,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
    thread::sleep,
    time::{Duration, SystemTime},
};

use tokio::task::JoinHandle;
use tracing::Level;
use vanity_nbt::snbt::from_snbt;

use crate::error::McLogError;

const LOG_IDENTIFIER: &'static str = "Test log";
const LOG_PRE_IDENT: usize = 32;
const LOG_SUF_IDENT: usize = 32;

mod data;
mod error;

pub use data::*;

#[derive(Debug, Clone)]
pub struct WatchConfig {
    pub identifier: String,
    pub interval: Duration,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            identifier: LOG_IDENTIFIER.to_string(),
            interval: Duration::from_millis(50),
        }
    }
}

impl WatchConfig {
    #[inline]
    pub fn sleep(&self, mult: Option<u32>) {
        sleep(self.interval * mult.unwrap_or(1));
    }
}

macro_rules! log_event {
    ($level:expr, $log:expr) => {
        tracing::event!(
            $level,
            function = $log.function,
            entity_type = $log.entity.r#type,
            tick = $log.tick,
            position = ?$log.pos,
            rotation = ?$log.rotation,
            dimension = $log.dimension,
            entity_data = ?$log.entity.data,
            entity_uuid = ?$log.entity.uuid,
            "{}",
            $log.message
        )
    };
}

#[cfg(feature = "tracing")]
pub fn log_with_tracing(
    latest_log: impl AsRef<Path>,
    config: WatchConfig,
) -> JoinHandle<Result<(), McLogError>> {
    log(latest_log, config, |log| {
        match log.level {
            LogLevel::Trace => log_event!(Level::TRACE, log),
            LogLevel::Debug => log_event!(Level::DEBUG, log),
            LogLevel::Info => log_event!(Level::INFO, log),
            LogLevel::Warn => log_event!(Level::WARN, log),
            LogLevel::Error => log_event!(Level::ERROR, log),
        };
        Ok(())
    })
}

pub fn log<F>(
    latest_log: impl AsRef<Path>,
    config: WatchConfig,
    on_log: F,
) -> JoinHandle<Result<(), McLogError>>
where
    F: Fn(LogMessage) -> Result<(), McLogError> + 'static + Send + Sync,
{
    let latest_log = latest_log.as_ref().to_path_buf();

    let handle = tokio::spawn(async move {
        let mut byte_offset = 0;
        let mut latest_modified = SystemTime::UNIX_EPOCH;

        loop {
            // if no log file we just wait a bit extra and wait
            let file = match File::open(&latest_log) {
                Ok(f) => f,
                Err(_) => {
                    config.sleep(Some(5));
                    continue;
                }
            };

            let modified = file
                .metadata()
                .map_err(|e| McLogError::FailedToGetMetadata(e))?
                .modified()
                .map_err(|e| McLogError::FailedToGetMetadata(e))?;
            if modified <= latest_modified {
                config.sleep(None);
                continue;
            }
            latest_modified = modified;

            let mut buf = BufReader::new(file);
            buf.seek(SeekFrom::Start(byte_offset))
                .map_err(|e| McLogError::FailedToSeekTo(byte_offset, e))?;

            for line in buf.split(b'\n') {
                let line = line.map_err(|e| McLogError::InvalidLine(e))?;
                byte_offset += line.len() as u64;

                let line = String::from_utf8_lossy(&line);
                let log = match parse_log_line(&line, &config.identifier)? {
                    Some(l) => l,
                    None => {
                        continue;
                    }
                };

                on_log(log)?;
            }

            config.sleep(None);
        }
    });

    handle
}

pub fn parse_log_line(line: &str, identifier: &str) -> Result<Option<LogMessage>, McLogError> {
    let mut s_line = String::with_capacity(line.len());
    for c in line.chars() {
        if !c.is_ascii() || ('\u{E000}'..='\u{F8FF}').contains(&c) {
            write!(s_line, "\\u{:04X}", c as u32)
                .map_err(|e| McLogError::FailedToWriteEscapedChars(e))?
        } else {
            s_line.push(c);
        }
    }

    let ident_end = LOG_PRE_IDENT + identifier.len();
    if s_line.len() <= ident_end
        || s_line
            .get(LOG_PRE_IDENT + 1..=ident_end)
            .ok_or(McLogError::NotEnoughForIdentifier(s_line.clone()))?
            != identifier
    {
        return Ok(None);
    }

    let snbt_start = ident_end + LOG_SUF_IDENT;
    let nbt = from_snbt(
        s_line
            .get(snbt_start..)
            .ok_or(McLogError::NotEnoughForSnbtStart(s_line.clone()))?,
    )?;

    Ok(Some(LogMessage::from_snbt(nbt)?))
}
