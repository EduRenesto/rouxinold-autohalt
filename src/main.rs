use std::time::{Duration, Instant};

use tokio::process::Command;
use tracing_loki::url::Url;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(file) = std::env::var("ROUXINOLD_ENV_FILE") {
        dotenvy::from_filename(file)?;
    } else {
        let _ = dotenvy::dotenv();
    }

    let env = std::env::var("ROUXINOLD_ENV").unwrap_or("DEV".to_string());

    match env.as_str() {
        "PROD" => {
            let loki_addr = std::env::var("ROUXINOLD_LOKI_ADDR")?;

            let (layer, task) = tracing_loki::builder()
                .label("host", std::env::var("HOSTNAME")?)?
                .build_url(Url::parse(&loki_addr)?)?;

            tracing_subscriber::registry()
                .with(layer)
                .init();

            tokio::spawn(task);
        }
        _ => {
            tracing_subscriber::fmt::init();
        }
    }

    let server_ip = std::env::var("ROUXINOLD_SERVER_IP")?;
    let server_port = std::env::var("ROUXINOLD_SERVER_PORT")?.parse()?;

    let timeout_secs = std::env::var("ROUXINOLD_TIMEOUT_SECS")?.parse()?;
    let timeout = Duration::from_secs(timeout_secs);

    let poll_interval_secs = std::env::var("ROUXINOLD_POLL_INTERVAL")?.parse()?;
    let poll_interval = Duration::from_secs(poll_interval_secs);

    let mut interval = tokio::time::interval(poll_interval);

    let mut last_active = Instant::now();

    tracing::info!("rouxinold-autohalt starting");

    loop {
        interval.tick().await;

        let delta = Instant::now() - last_active;

        if delta > timeout {
            tracing::info!("Last active date is more than {timeout_secs}s in the past. Shutting down machine.");

            let ret = Command::new("sudo")
                .arg("shutdown")
                .arg("now")
                .output()
                .await?;

            if !ret.status.success() {
                let stdout = String::from_utf8(ret.stdout)?;
                let stderr = String::from_utf8(ret.stderr)?;

                tracing::event!(
                    tracing::Level::ERROR,
                    stdout = stdout,
                    stderr = stderr,
                    "Could not turn machine off!"
                );
            }
        }

        let stats = mc_query::status(&server_ip, server_port).await?;

        tracing::debug!("Players online: {}", stats.players.online);

        if stats.players.online > 0 {
            last_active = Instant::now();
        }
    }
}
