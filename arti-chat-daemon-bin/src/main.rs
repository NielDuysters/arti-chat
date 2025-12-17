use daemonize::Daemonize;
use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    let daemon = Daemonize::new()
        .pid_file("/tmp/arti-chat-daemon.pid")
        .working_directory("/")
        .umask(0o027);
    daemon.start().expect("Failed to daemonize");

    let rt = Runtime::new()?;
    rt.block_on(async {
        if let Err(e) = arti_chat_daemon::run().await {
            tracing::error!("Daemon runtime error: {:?}", e);
        }
    });

    Ok(())
}
