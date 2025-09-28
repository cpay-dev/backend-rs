use tokio::signal;

pub async fn shutdown_signal() {
  #[cfg(unix)]
  {
    let mut term_signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install SIGTERM handler");
    tokio::select! {
      _ = signal::ctrl_c() => {  }
      _ = term_signal.recv() => {  }
    }
    return;
  }

  #[cfg(not(unix))]
  {
    let _ = signal::ctrl_c().await;
  }
}
