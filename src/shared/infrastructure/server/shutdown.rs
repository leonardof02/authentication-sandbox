pub async fn signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Ctrl+C doesn't work on this platform");
    tracing::info!("Shutdown signal received, stopping the server");
}
