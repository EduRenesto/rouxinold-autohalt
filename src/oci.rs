use tokio::process::Command;

pub async fn stop_instance(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cmd_path = std::env::var("ROUXINOLD_OCI_CLI_PATH")?;
    let conf_path = std::env::var("OCI_CLI_CONFIG_FILE")?;

    let out = Command::new(cmd_path)
        .arg("--config-file")
        .arg(conf_path)
        .arg("compute")
        .arg("instance")
        .arg("action")
        .arg("--action")
        .arg("STOP")
        .arg("--instance-id")
        .arg(id)
        .output();

    let out = out.await?;

    if out.status.success() {
        return Ok(());
    }

    let out_str = String::from_utf8(out.stderr)?;

    return Err(format!("failed to stop instance: ```{}```", out_str).into());
}
