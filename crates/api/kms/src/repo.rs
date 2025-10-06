use crate::config::{DatabaseConfig};
use crate::error::AppError;
use postgres_types::{FromSql, ToSql};
use rustls_tokio_postgres::{config_webpki_roots, MakeRustlsConnect};
use tracing::{error, info, trace};

pub struct Repository {
  client: tokio_postgres::Client,
  connection_task: tokio::task::JoinHandle<Result<(), tokio_postgres::Error>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "key_status")]
pub enum KeyStatus {
  #[postgres(name = "ACTIVE")]
  Active,
  #[postgres(name = "DECRYPT_ONLY")]
  DecryptOnly,
  #[postgres(name = "DISABLED")]
  Disabled,
}

#[derive(Debug, Clone)]
pub struct KekRecord {
  pub key_version: u32,
  pub root_key_version: u32,
  pub status: KeyStatus,
  pub encrypted_key: Vec<u8>,
}

impl Repository {
  pub async fn init(config: &DatabaseConfig) -> Result<Self, AppError> {
    trace!("initializing postgres repository");

    let tls = MakeRustlsConnect::new(config_webpki_roots());
    let (client, connection) = tokio_postgres::connect(&config.conn_string(), tls).await?;

    let connection_task = tokio::spawn(async move {
      info!("spawning database connection");
      let res = connection.await;
      if let Err(e) = &res {
        error!(error = ?e, "postgres connection error");
      }
      res
    });

    _ = client.query("SELECT 1", &[]).await?;
    info!("postgres ping succeeded");

    Ok(Self {
      client,
      connection_task,
    })
  }

  pub async fn load_keys(&self) -> Result<Vec<KekRecord>, AppError> {
    trace!("loading KMS keys from database");
    let rows = self
      .client
      .query(
        "
        SELECT key_version, root_key_version, status, encrypted_key 
        FROM kms.keys 
        WHERE status != $1 AND disabled_at IS NULL;",
        &[&KeyStatus::Disabled],
      )
      .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
      let key_version: i32 = row.get(0);
      let root_key_version: i32 = row.get(1);
      let status: KeyStatus = row.get(2);
      let encrypted_key: Vec<u8> = row.get(3);
      out.push(KekRecord {
        key_version: u32::try_from(key_version).expect("key version too large"),
        root_key_version: u32::try_from(root_key_version).expect("root key version too large"),
        status,
        encrypted_key,
      });
    }
    info!(count = out.len(), "loaded KMS keys");
    Ok(out)
  }

  pub async fn shutdown(self) -> Result<(), AppError> {
    trace!("shutting down postgres repository");
    let Repository {
      client,
      connection_task,
    } = self;
    drop(client);
    let join_res = connection_task.await?;
    join_res?;
    info!("postgres connection closed");
    Ok(())
  }
}
