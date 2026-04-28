use std::{
    env::{args, current_dir},
    fs::File,
    path::PathBuf,
    process::exit,
    sync::Arc,
    thread::sleep,
    time::Duration,
};

use mclog_capture::{WatchConfig, log_with_tracing};
use tokio::signal;
use tracing_subscriber::{
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[inline]
fn wait_exit() -> ! {
    sleep(Duration::from_secs(5));
    exit(0)
}

#[tokio::main]
async fn main() {
    let latest_log = match get_log_path() {
        Some(ll) => ll,
        None => {
            eprintln!("No log file was found");
            wait_exit();
        }
    };

    println!("Starting to capture logs from: '{}'", latest_log.display());

    match std::fs::exists(&latest_log) {
        Ok(true) => (),
        _ => {
            println!("Warning: Log file doesn't exist, watching for when a file appears...");
        }
    }

    print!("\n");

    let plain = File::create("mclog.log").unwrap();
    let json = File::create("mclog.json").unwrap();

    let l_stdout = fmt::layer().with_writer(std::io::stdout).with_target(true);
    let l_plain = fmt::layer().with_ansi(false).with_writer(Arc::new(plain));
    let l_json = fmt::layer()
        .json()
        .with_ansi(false)
        .with_writer(Arc::new(json));

    tracing_subscriber::registry()
        .with(l_stdout)
        .with(l_plain)
        .with(l_json)
        .try_init()
        .unwrap();

    let _handle = log_with_tracing(&latest_log, WatchConfig::default());

    match signal::ctrl_c().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("unable to listen for shutdown: {e}");
        }
    }

    exit(0)
}

const ENV_KEY: &'static str = "LATEST_LOG";
const LOG_FILENAME: &'static str = "latest.log";
const MINECRAFT_DIR: &'static str = ".minecraft";
fn get_log_path() -> Option<PathBuf> {
    // 1. CLI arg
    let args = args().collect::<Vec<String>>();
    if let Some(arg_path) = args.get(1) {
        return Some(PathBuf::from(arg_path));
    }

    // 2. Env var
    match dotenvy::dotenv() {
        Ok(_) => {
            if let Ok(var_path) = dotenvy::var(ENV_KEY) {
                return Some(PathBuf::from(var_path));
            }
        }
        Err(_) => {
            if let Ok(var_path) = std::env::var(ENV_KEY) {
                return Some(PathBuf::from(var_path));
            }
        }
    }

    // 3. Current directory
    if let Ok(true) = std::fs::exists(LOG_FILENAME) {
        return Some(PathBuf::from(LOG_FILENAME));
    }

    // 4. Traverse up a few (if we for example exist within a datapack folder)
    if let Some(mut minecraft) = traverse_to_minecraft() {
        // go into the logs directory if we got .minecraft
        minecraft.push("logs");
        minecraft.push(LOG_FILENAME);
        return Some(minecraft);
    }

    None
}

#[inline]
fn traverse_to_minecraft() -> Option<PathBuf> {
    let mut curr = current_dir().unwrap();
    while curr.components().count() > 0 {
        if curr.file_name().unwrap_or_default() == MINECRAFT_DIR {
            return Some(curr);
        }

        match curr.parent() {
            Some(p) => curr = p.to_path_buf(),
            None => return None,
        };
    }

    None
}
