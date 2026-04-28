use std::{
    fmt::Write,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
    thread::sleep,
    time::{Duration, SystemTime},
};

use vanity_nbt::snbt::from_snbt;

use crate::data::LogMessage;

const LOG_IDENTIFIER: &'static str = "Test log";
const LOG_PRE_IDENT: usize = 32;
const LOG_SUF_IDENT: usize = 32;

mod data;

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
    pub fn sleep(&self) {
        sleep(self.interval);
    }
}

pub fn log(latest_log: impl AsRef<Path>, config: WatchConfig) -> ! {
    let latest_log = latest_log.as_ref();
    if !latest_log.exists() {
        panic!("log file doesn't exist")
    }

    let mut byte_offset = 0;
    let mut latest_modified = SystemTime::UNIX_EPOCH;

    loop {
        let file = File::open(latest_log).unwrap();

        let modified = file.metadata().unwrap().modified().unwrap();
        if modified <= latest_modified {
            config.sleep();
            continue;
        }
        latest_modified = modified;

        let mut buf = BufReader::new(file);
        buf.seek(SeekFrom::Start(byte_offset)).unwrap();

        for line in buf.split(b'\n') {
            let line = line.unwrap();
            byte_offset += line.len() as u64;

            let line = String::from_utf8_lossy(&line);
            let mut s_line = String::with_capacity(line.len());
            for c in line.chars() {
                if !c.is_ascii() || ('\u{E000}'..='\u{F8FF}').contains(&c) {
                    write!(s_line, "\\u{:04X}", c as u32).unwrap();
                } else {
                    s_line.push(c);
                }
            }

            let ident_end = LOG_PRE_IDENT + config.identifier.len();
            if s_line.len() <= ident_end
                || s_line.get(LOG_PRE_IDENT + 1..=ident_end).unwrap() != config.identifier
            {
                continue;
            }

            let snbt_start = ident_end + LOG_SUF_IDENT;
            let nbt = match from_snbt(s_line.get(snbt_start..).unwrap()) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("{e:?}");
                    continue;
                }
            };

            let log = LogMessage::from_snbt(nbt).unwrap();
            println!("{log:?}");
        }
    }
}
