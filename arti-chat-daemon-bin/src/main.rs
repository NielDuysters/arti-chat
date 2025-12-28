use daemonize::Daemonize;
use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let manual_daemonize = args.get(1).map(String::as_str) == Some("run");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    if manual_daemonize {
        let daemon = Daemonize::new()
            .pid_file("/tmp/arti-chat-daemon.pid")
            .working_directory("/")
            .umask(0o027);

        daemon.start()
            .map_err(|e| anyhow::anyhow!("Failed to daemonize: {}", e))?;
    }

    let rt = Runtime::new()?;
    rt.block_on(async {
        arti_chat_daemon::run().await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

