use std::time::{Duration, Instant};

use tracing::level_filters::LevelFilter;
use tracing_loki::url::Url;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod oci;

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

            let filter = EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy();

            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();

            tokio::spawn(task);
        }
        _ => {
            tracing_subscriber::fmt::init();
        }
    }

    let instance_ocid = std::env::var("ROUXINOLD_INSTANCE_ID")?;

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

            let ret = oci::stop_instance(&instance_ocid).await;

            match ret {
                Ok(()) => {
                    tracing::info!("Shutdown issued! Quitting rouxinold-autohalt...");
                    return Ok(())
                }
                Err(e) => {
                    tracing::error!("Error shutting down instance: {}", e.to_string());
                    return Err(e)
                }
            }
        }

        let stats = mc_query::status(&server_ip, server_port).await;

        match stats {
            Ok(stats) => {
                tracing::debug!("Players online: {}", stats.players.online);

                if stats.players.online > 0 {
                    last_active = Instant::now();
                }
            }
            Err(e) => {
                tracing::error!("Failed to query server {}:{}. Error: {}", server_ip, server_port, e.to_string());
            }
        }
    }
}
