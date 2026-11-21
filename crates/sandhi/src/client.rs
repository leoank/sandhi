//! The client side of the control socket: `sandhi connect` and `sandhi status`.

use std::path::Path;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::daemon::read_line;

async fn dial(socket: &Path) -> Result<UnixStream> {
    UnixStream::connect(socket)
        .await
        .with_context(|| format!("reaching sandhi at {}; is `sandhi up` running?", socket.display()))
}

/// A stream to `host:port` through the daemon, ready for bytes.
pub async fn connect(socket: &Path, host: &str, port: u16) -> Result<UnixStream> {
    if host.is_empty() || host.contains(char::is_whitespace) {
        bail!("{host:?} is not a host name");
    }
    let mut s = dial(socket).await?;
    s.write_all(format!("connect {host} {port}\n").as_bytes()).await?;
    let reply = read_line(&mut s).await?;
    match reply.as_str() {
        "ok" => Ok(s),
        r => bail!("{}", r.strip_prefix("error ").unwrap_or(r)),
    }
}

pub async fn status(socket: &Path) -> Result<String> {
    let mut s = dial(socket).await?;
    s.write_all(b"status\n").await?;
    let mut out = String::new();
    s.read_to_string(&mut out).await?;
    Ok(out)
}

/// Splices stdin and stdout to `s`, for ssh's `ProxyCommand`, until the peer closes.
pub async fn splice_stdio(s: UnixStream) -> Result<()> {
    let (mut from_peer, mut to_peer) = s.into_split();
    tokio::spawn(async move {
        let _ = tokio::io::copy(&mut tokio::io::stdin(), &mut to_peer).await;
        let _ = to_peer.shutdown().await;
    });
    let mut stdout = tokio::io::stdout();
    tokio::io::copy(&mut from_peer, &mut stdout).await?;
    stdout.flush().await?;
    Ok(())
}
