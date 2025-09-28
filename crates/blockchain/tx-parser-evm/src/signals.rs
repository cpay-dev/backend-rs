use tokio::signal;
use tracing::info;

pub async fn shutdown_signal() {
  #[cfg(unix)]
  {
    let mut term_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install SIGTERM handler");
    tokio::select! {
      _ = signal::ctrl_c() => { info!("received SIGINT; shutting down"); }
      _ = term_signal.recv() => { info!("received SIGTERM; shutting down"); }
    }
    return;
  }

  #[cfg(not(unix))]
  {
    let _ = signal::ctrl_c().await;
    info!("received interrupt; shutting down");
  }
}
