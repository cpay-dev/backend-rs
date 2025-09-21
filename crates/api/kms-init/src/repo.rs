use crate::config::DatabaseConfig;
use crate::error::AppError;
use postgres_types::{FromSql, ToSql};
use rustls::pki_types::{CertificateDer, pem::PemObject};
use rustls_tokio_postgres::MakeRustlsConnect;
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
  pub key_version: i32,
  pub root_key_version: i32,
  pub status: KeyStatus,
  pub encrypted_key: Vec<u8>,
}

impl Repository {
  pub async fn init(config: &DatabaseConfig) -> Result<Self, AppError> {
    trace!("initializing postgres repository");

    let mut roots = rustls::RootCertStore::empty();
    let root_cert = CertificateDer::from_pem_file(&config.tls_cert_path)?;
    roots.add(root_cert).map_err(|e| AppError::RootCertAdd(e.to_string()))?;

    let tls_config = rustls::ClientConfig::builder()
      .with_root_certificates(roots)
      .with_no_client_auth();
    let tls = MakeRustlsConnect::new(tls_config);

    let (client, connection) = tokio_postgres::connect(&config.conn_string(), tls).await?;

    let connection_task = tokio::spawn(async move {
      trace!("spawning database connection");
      let res = connection.await;
      if let Err(e) = &res {
        error!(error = ?e, "postgres connection error");
      }
      res
    });

    _ = client.query("SELECT 1", &[]).await?;
    info!("db ping ok");

    Ok(Self {
      client,
      connection_task,
    })
  }

  pub async fn insert_key(&self, key: &KekRecord) -> Result<(), AppError> {
    self
      .client
      .execute(
        "
        INSERT INTO kms.keys (id, key_version, root_key_version, status, encrypted_key, created_at, updated_at)
        VALUES (gen_ulid(), $1, $2, $3, $4, NOW(), NOW());",
        &[&key.key_version, &key.root_key_version, &key.status, &key.encrypted_key],
      )
      .await?;
    Ok(())
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
