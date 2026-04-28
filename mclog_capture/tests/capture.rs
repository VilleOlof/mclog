use mclog_capture::{WatchConfig, log};

#[test]
fn capture() {
    dotenvy::dotenv().unwrap();
    let latest_log = dotenvy::var("LATEST_LOG").unwrap();
    log(&latest_log, WatchConfig::default());
}
