//! `sandhi up`: every backend's dutavas, the router, the inbound forwards and the control
//! socket that `sandhi connect` and `sandhi status` talk to (ADR-0005).
//!
//! The control protocol is one line from the client, `connect <host> <port>` or `status`.
//! To `connect` the daemon answers `ok` and splices the socket to the peer from then on, or
//! answers `error <why>` and closes; to `status` it writes one line per backend and closes.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UnixListener, UnixStream};
use tokio::sync::{mpsc, watch};

use crate::backend::tailnet::TailnetBackend;
use crate::backend::{Backend, BoxIo, Inbound};
use crate::config::{Config, Forward, Kind};
use crate::router::Router;

/// How long a connect waits for a backend that is still joining.
const WAIT_FOR_JOIN: Duration = Duration::from_secs(60);

#[derive(Clone)]
pub enum State {
    Joining,
    Ready {
        backend: Arc<dyn Backend>,
        /// Forwards that could not be set up.
        problems: Vec<String>,
    },
    Failed(String),
}

struct Entry {
    kind: &'static str,
    forwards: Vec<Forward>,
    state: watch::Receiver<State>,
}

pub struct Daemon {
    router: Router,
    backends: BTreeMap<String, Entry>,
    wait_for_join: Duration,
}

/// Settles one backend's state once its dutavas has joined, or not.
pub struct Joiner {
    pub name: String,
    forwards: Vec<Forward>,
    tx: watch::Sender<State>,
}

impl Joiner {
    /// The dutavas is up: its forwards start listening and connects go through it.
    pub fn ready(self, backend: Arc<dyn Backend>) {
        let mut problems = Vec::new();
        for f in self.forwards {
            match backend.listen(f.port) {
                Ok(rx) => {
                    tracing::info!(backend = %self.name, port = f.port, to = %f.to, "forwarding");
                    tokio::spawn(forward(self.name.clone(), f, rx));
                }
                Err(e) => {
                    tracing::error!(backend = %self.name, port = f.port, error = %e, "cannot listen");
                    problems.push(format!("port {}: {e}", f.port));
                }
            }
        }
        tracing::info!(backend = %self.name, addresses = %backend.describe(), "ready");
        self.tx.send_replace(State::Ready { backend, problems });
    }

    pub fn failed(self, why: String) {
        tracing::error!(backend = %self.name, %why, "failed");
        self.tx.send_replace(State::Failed(why));
    }
}

impl Daemon {
    pub fn new(router: Router) -> Daemon {
        Daemon { router, backends: BTreeMap::new(), wait_for_join: WAIT_FOR_JOIN }
    }

    /// How long a connect waits for a backend still joining; for tests.
    pub fn wait_for_join(mut self, d: Duration) -> Daemon {
        self.wait_for_join = d;
        self
    }

    /// Registers a backend as joining.
    pub fn add(&mut self, name: &str, kind: &'static str, forwards: Vec<Forward>) -> Joiner {
        let (tx, rx) = watch::channel(State::Joining);
        self.backends.insert(name.to_string(), Entry { kind, forwards: forwards.clone(), state: rx });
        Joiner { name: name.to_string(), forwards, tx }
    }

    /// Answers the control socket until accepting fails.
    pub async fn serve(self: Arc<Self>, listener: UnixListener) -> Result<()> {
        loop {
            let (conn, _) = listener.accept().await.context("accepting on the control socket")?;
            let me = self.clone();
            tokio::spawn(async move {
                if let Err(e) = me.handle(conn).await {
                    tracing::debug!(error = %format!("{e:#}"), "control connection");
                }
            });
        }
    }

    async fn handle(&self, mut s: UnixStream) -> Result<()> {
        let line = read_line(&mut s).await?;
        let words: Vec<&str> = line.split_whitespace().collect();
        match words.as_slice() {
            ["connect", host, port] => {
                let opened = match port.parse::<u16>() {
                    Ok(port) => self.open(host, port).await,
                    Err(_) => Err(anyhow!("{port:?} is not a port")),
                };
                match opened {
                    Ok(mut peer) => {
                        s.write_all(b"ok\n").await?;
                        tokio::io::copy_bidirectional(&mut s, &mut peer).await?;
                    }
                    Err(e) => {
                        tracing::info!(%host, %port, error = %format!("{e:#}"), "connect refused");
                        s.write_all(format!("error {}\n", one_line(&e)).as_bytes()).await?;
                    }
                }
            }
            ["status"] => s.write_all(self.status().as_bytes()).await?,
            _ => s.write_all(format!("error unknown command {line:?}\n").as_bytes()).await?,
        }
        Ok(())
    }

    /// A connection to `host:port` through the backend that owns `host`.
    pub async fn open(&self, host: &str, port: u16) -> Result<BoxIo> {
        let route = self.router.route(host).ok_or_else(|| anyhow!("no backend owns {host}"))?;
        let backend = self.ready(&route.backend).await?;
        backend
            .dial(&route.host, port)
            .await
            .with_context(|| format!("dialling {}:{port} on {}", route.host, route.backend))
    }

    async fn ready(&self, name: &str) -> Result<Arc<dyn Backend>> {
        let entry = self.backends.get(name).ok_or_else(|| anyhow!("no backend {name}"))?;
        let mut rx = entry.state.clone();
        let deadline = tokio::time::Instant::now() + self.wait_for_join;
        loop {
            match &*rx.borrow_and_update() {
                State::Ready { backend, .. } => return Ok(backend.clone()),
                State::Failed(why) => bail!("{name} failed: {why}"),
                State::Joining => {}
            }
            if tokio::time::timeout_at(deadline, rx.changed()).await.is_err() {
                bail!("{name} is still joining");
            }
        }
    }

    /// One line per backend: name, kind, state, addresses and forwards.
    pub fn status(&self) -> String {
        let mut out = String::new();
        for (name, e) in &self.backends {
            let state = match &*e.state.borrow() {
                State::Joining => "joining".to_string(),
                State::Ready { backend, problems } => {
                    let mut s = format!("ready {}", backend.describe());
                    for p in problems {
                        s += &format!(" (cannot forward {p})");
                    }
                    s
                }
                State::Failed(why) => format!("failed: {}", why.replace('\n', " ")),
            };
            let forwards: Vec<String> = e.forwards.iter().map(|f| format!("{}->{}", f.port, f.to)).collect();
            out += &format!("{name} {} {state}", e.kind);
            if !forwards.is_empty() {
                out += &format!(" forwards {}", forwards.join(","));
            }
            out.push('\n');
        }
        out
    }
}

/// Splices each caller on a backend's port to the local service.
async fn forward(backend: String, f: Forward, mut rx: mpsc::Receiver<Inbound>) {
    while let Some(inbound) = rx.recv().await {
        let (backend, f) = (backend.clone(), f.clone());
        tokio::spawn(async move {
            let Inbound { mut io, caller } = inbound;
            match TcpStream::connect(&f.to).await {
                Ok(mut local) => {
                    tracing::info!(%backend, port = f.port, %caller, to = %f.to, "inbound");
                    let _ = tokio::io::copy_bidirectional(&mut io, &mut local).await;
                }
                Err(e) => tracing::warn!(%backend, port = f.port, %caller, to = %f.to, error = %e, "the local service refused"),
            }
        });
    }
}

/// One `\n`-terminated line, read a byte at a time so nothing after it is consumed.
pub async fn read_line(s: &mut (impl AsyncRead + Unpin)) -> Result<String> {
    let mut line = Vec::new();
    loop {
        let b = s.read_u8().await.context("the line ended early")?;
        if b == b'\n' {
            break;
        }
        if line.len() >= 4096 {
            bail!("the line is too long");
        }
        line.push(b);
    }
    Ok(String::from_utf8(line)?.trim_end_matches('\r').to_string())
}

fn one_line(e: &anyhow::Error) -> String {
    format!("{e:#}").replace('\n', " ")
}

/// Runs the daemon from a config until interrupted.
pub async fn run(config: Config) -> Result<()> {
    let state_dir = config.state_dir();
    private_dir(&state_dir)?;
    let socket = config.socket();
    let listener = bind(&socket)?;

    let mut daemon = Daemon::new(Router::new(&config));
    let mut joins = Vec::new();
    for (name, b) in &config.backends {
        match b.kind()? {
            Kind::Tailnet(t) => joins.push((daemon.add(name, "tailnet", b.forwards.clone()), t)),
        }
    }
    for (joiner, t) in joins {
        let state_dir = state_dir.clone();
        tokio::spawn(async move {
            match TailnetBackend::start(&joiner.name, &t, &state_dir).await {
                Ok(b) => joiner.ready(Arc::new(b)),
                Err(e) => joiner.failed(format!("{e:#}")),
            }
        });
    }
    tracing::info!(socket = %socket.display(), "listening");
    let daemon = Arc::new(daemon);
    let result = tokio::select! {
        r = daemon.serve(listener) => r,
        _ = shutdown() => Ok(()),
    };
    let _ = std::fs::remove_file(&socket);
    result
}

async fn shutdown() {
    use tokio::signal::unix::{SignalKind, signal};
    let mut term = signal(SignalKind::terminate()).expect("a SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
    }
}

fn private_dir(dir: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .with_context(|| format!("restricting {}", dir.display()))
}

/// The control socket, for this user only. A socket left by a daemon that died is
/// replaced; one a running daemon answers on is not.
pub fn bind(socket: &Path) -> Result<UnixListener> {
    use std::os::unix::fs::PermissionsExt;
    if let Some(dir) = socket.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    if socket.exists() {
        if std::os::unix::net::UnixStream::connect(socket).is_ok() {
            bail!("sandhi is already running on {}", socket.display());
        }
        std::fs::remove_file(socket).with_context(|| format!("removing the stale {}", socket.display()))?;
    }
    let listener = UnixListener::bind(socket).with_context(|| format!("binding {}", socket.display()))?;
    std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("restricting {}", socket.display()))?;
    Ok(listener)
}
