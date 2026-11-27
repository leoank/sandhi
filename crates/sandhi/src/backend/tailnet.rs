//! A tailnet backend: its dutavas is an embedded tsnet identity (`sandhi-tailnet`, ADR-0002).

use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use sandhi_tailnet::{AuthKey, Options, Tailnet};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use super::{Backend, BoxFuture, BoxIo, Inbound};
use crate::config::TailnetConfig;

/// A start from saved state returns before the netmap names the peers, so a dial by name
/// may fail at first ([[saarthi/EXP-20261112-rq8kgc]]): dials are retried this long after
/// the dutavas came up.
const SETTLING: Duration = Duration::from_secs(30);
const RETRY_EVERY: Duration = Duration::from_millis(500);
/// How long a join may go unanswered before the backend is reported failed. A join the
/// tailnet will not let through without a person fails within seconds.
const JOIN_LIMIT: Duration = Duration::from_secs(600);
/// How often a missing auth key file is looked for.
const KEY_POLL: Duration = Duration::from_secs(1);

pub struct TailnetBackend {
    tailnet: Arc<Tailnet>,
    up_at: Instant,
}

impl TailnetBackend {
    /// Joins `name`'s tailnet with its state under `state_dir/<name>`.
    pub async fn start(name: &str, config: &TailnetConfig, state_dir: &Path) -> Result<TailnetBackend> {
        let hostname = match &config.hostname {
            Some(h) => h.clone(),
            None => this_hostname()?,
        };
        let dir = state_dir.join(name);
        if let Some(file) = &config.authkey_file {
            wait_for_key(file, &dir, JOIN_LIMIT, KEY_POLL).await?;
        }
        let tailnet = Tailnet::start(Options {
            name: name.to_string(),
            dir,
            hostname,
            authkey: config.authkey_file.clone().map(AuthKey::File),
            ephemeral: config.ephemeral,
            give_up_after: Some(JOIN_LIMIT),
        })
        .await?;
        Ok(TailnetBackend { tailnet, up_at: Instant::now() })
    }
}

impl Backend for TailnetBackend {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxIo>> {
        Box::pin(async move {
            let addr = format!("{host}:{port}");
            loop {
                match self.tailnet.dial(&addr).await {
                    Ok(s) => return Ok(Box::new(s) as BoxIo),
                    Err(e) if self.up_at.elapsed() < SETTLING => {
                        tracing::debug!(tailnet = %self.tailnet.name(), %addr, error = %e, "dial failed while the tailnet settles; retrying");
                        tokio::time::sleep(RETRY_EVERY).await;
                    }
                    Err(e) => return Err(e),
                }
            }
        })
    }

    fn listen(&self, port: u16) -> io::Result<mpsc::Receiver<Inbound>> {
        let mut incoming = self.tailnet.incoming(port).map_err(|e| io::Error::other(format!("{e:#}")))?;
        let (tx, rx) = mpsc::channel(16);
        let name = self.tailnet.name().to_string();
        tokio::spawn(async move {
            while let Some(conn) = incoming.next().await {
                let conn = match conn {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!(tailnet = %name, port, error = %e, "accepting a connection failed");
                        continue;
                    }
                };
                let caller = match conn.remote().peer().await {
                    Some(p) => format!("{p} at {}", conn.remote().addr),
                    None => conn.remote().addr.clone(),
                };
                if tx.send(Inbound { io: Box::new(conn), caller }).await.is_err() {
                    return;
                }
            }
        });
        Ok(rx)
    }

    fn describe(&self) -> String {
        self.tailnet.ips()
    }
}

/// Waits for an auth key file that is not there yet, when a join will need it: the
/// service can start before agenix has decrypted the key (ADR-0007). An identity that
/// has joined before needs no key, so its start is never held up.
async fn wait_for_key(file: &Path, dir: &Path, limit: Duration, poll: Duration) -> Result<()> {
    // tsnet keeps the node key in this file; `sandhi-tailnet` reads the auth key only
    // while it is missing.
    let joined = || dir.join("tailscaled.state").exists();
    if file.exists() || joined() {
        return Ok(());
    }
    tracing::info!(file = %file.display(), "waiting for the auth key file");
    let deadline = Instant::now() + limit;
    while !file.exists() {
        if Instant::now() >= deadline {
            anyhow::bail!("the auth key file {} did not appear in {} s", file.display(), limit.as_secs());
        }
        tokio::time::sleep(poll).await;
    }
    Ok(())
}

/// This machine's hostname, without a `.local` or other domain.
fn this_hostname() -> Result<String> {
    let mut buf = [0u8; 256];
    if unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) } != 0 {
        anyhow::bail!("gethostname: {}", io::Error::last_os_error());
    }
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    let name = String::from_utf8_lossy(&buf[..end]);
    Ok(name.split('.').next().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(10);

    #[tokio::test]
    async fn a_missing_key_is_waited_for_until_it_appears() {
        let t = tempfile::tempdir().unwrap();
        let key = t.path().join("key");
        let writer = {
            let key = key.clone();
            tokio::spawn(async move {
                tokio::time::sleep(5 * MS).await;
                std::fs::write(key, "k").unwrap();
            })
        };
        wait_for_key(&key, &t.path().join("state"), Duration::from_secs(5), MS).await.unwrap();
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn a_key_that_never_appears_fails_after_the_limit() {
        let t = tempfile::tempdir().unwrap();
        let e = wait_for_key(&t.path().join("key"), &t.path().join("state"), 3 * MS, MS).await.unwrap_err();
        assert!(e.to_string().contains("did not appear"), "{e}");
    }

    #[tokio::test]
    async fn a_joined_identity_needs_no_key() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(t.path().join("tailscaled.state"), "{}").unwrap();
        wait_for_key(&t.path().join("key"), t.path(), Duration::ZERO, MS).await.unwrap();
    }
}
