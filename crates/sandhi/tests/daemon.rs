//! The daemon over its control socket, with a fake backend whose peers are local echo
//! servers: what `sandhi connect` and the inbound forwards do, with no network.

use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sandhi::backend::{Backend, BoxFuture, BoxIo, Inbound};
use sandhi::config::{Config, Forward};
use sandhi::daemon::{Daemon, bind};
use sandhi::router::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// A server that writes back what it reads, prefixed with its tag.
async fn echo(tag: &'static str) -> String {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = l.local_addr().unwrap().to_string();
    tokio::spawn(async move {
        loop {
            let (mut s, _) = l.accept().await.unwrap();
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                while let Ok(n) = s.read(&mut buf).await {
                    if n == 0 {
                        return;
                    }
                    let mut out = tag.as_bytes().to_vec();
                    out.extend_from_slice(&buf[..n]);
                    s.write_all(&out).await.unwrap();
                }
            });
        }
    });
    addr
}

/// Dials every peer at one echo server and records the names it was asked for; its
/// callers come in through senders the test holds.
#[derive(Default)]
struct Fake {
    peer: String,
    dialled: Mutex<Vec<String>>,
    listeners: Mutex<HashMap<u16, mpsc::Sender<Inbound>>>,
}

impl Backend for Fake {
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxIo>> {
        Box::pin(async move {
            self.dialled.lock().unwrap().push(format!("{host}:{port}"));
            Ok(Box::new(TcpStream::connect(&self.peer).await?) as BoxIo)
        })
    }
    fn listen(&self, port: u16) -> io::Result<mpsc::Receiver<Inbound>> {
        if port == 9 {
            return Err(io::Error::other("port 9 is taken"));
        }
        let (tx, rx) = mpsc::channel(4);
        self.listeners.lock().unwrap().insert(port, tx);
        Ok(rx)
    }
    fn describe(&self) -> String {
        "100.64.0.1".into()
    }
}

const CONFIG: &str = r#"
    [backends.home]
    domains = ["tail1234.ts.net"]
    [backends.home.tailnet]
    [backends.slow.tailnet]
    [backends.broken.tailnet]
"#;

struct Rig {
    _dir: tempfile::TempDir,
    socket: std::path::PathBuf,
    home: Arc<Fake>,
    slow: Option<sandhi::daemon::Joiner>,
}

async fn rig(forwards: Vec<Forward>) -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("sandhi.sock");
    let config = Config::parse(CONFIG).unwrap();
    let mut daemon = Daemon::new(Router::new(&config)).wait_for_join(Duration::from_secs(5));
    let home = Arc::new(Fake { peer: echo("home:").await, ..Default::default() });
    daemon.add("home", "fake", forwards).ready(home.clone());
    daemon.add("broken", "fake", vec![]).failed("the auth key was refused".into());
    let slow = daemon.add("slow", "fake", vec![]);
    let listener = bind(&socket).unwrap();
    tokio::spawn(Arc::new(daemon).serve(listener));
    Rig { _dir: dir, socket, home, slow: Some(slow) }
}

async fn round_trip(s: &mut (impl AsyncReadExt + AsyncWriteExt + Unpin), msg: &str) -> String {
    s.write_all(msg.as_bytes()).await.unwrap();
    let mut buf = [0u8; 1024];
    let n = s.read(&mut buf).await.unwrap();
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

#[tokio::test]
async fn connect_reaches_the_peer_through_the_backend_that_owns_the_name() {
    let r = rig(vec![]).await;
    let mut s = sandhi::client::connect(&r.socket, "ghost001.home", 22).await.unwrap();
    assert_eq!(round_trip(&mut s, "hello").await, "home:hello");
    let mut s = sandhi::client::connect(&r.socket, "Ghost001.Tail1234.ts.net.", 443).await.unwrap();
    assert_eq!(round_trip(&mut s, "again").await, "home:again");
    assert_eq!(*r.home.dialled.lock().unwrap(), ["ghost001:22", "ghost001.tail1234.ts.net:443"]);
}

#[tokio::test]
async fn a_name_no_backend_owns_and_a_failed_backend_are_refused_with_why() {
    let r = rig(vec![]).await;
    let e = sandhi::client::connect(&r.socket, "example.com", 22).await.unwrap_err();
    assert_eq!(e.to_string(), "no backend owns example.com");
    let e = sandhi::client::connect(&r.socket, "x.broken", 22).await.unwrap_err();
    assert_eq!(e.to_string(), "broken failed: the auth key was refused");
}

#[tokio::test]
async fn a_connect_waits_for_a_backend_still_joining() {
    let mut r = rig(vec![]).await;
    let socket = r.socket.clone();
    let pending = tokio::spawn(async move { sandhi::client::connect(&socket, "x.slow", 22).await });
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!pending.is_finished());
    let slow = Arc::new(Fake { peer: echo("slow:").await, ..Default::default() });
    r.slow.take().unwrap().ready(slow);
    let mut s = pending.await.unwrap().unwrap();
    assert_eq!(round_trip(&mut s, "hi").await, "slow:hi");
}

#[tokio::test]
async fn a_caller_on_a_forwarded_port_reaches_the_local_service() {
    let local = echo("local:").await;
    let r = rig(vec![
        Forward { port: 22, to: local },
        Forward { port: 9, to: "127.0.0.1:9".into() },
    ])
    .await;
    let tx = r.home.listeners.lock().unwrap()[&22].clone();
    let (mut caller, ours) = tokio::io::duplex(1024);
    tx.send(Inbound { io: Box::new(ours), caller: "rogue (me@github)".into() }).await.unwrap();
    assert_eq!(round_trip(&mut caller, "ssh").await, "local:ssh");

    let status = sandhi::client::status(&r.socket).await.unwrap();
    let lines: Vec<&str> = status.lines().collect();
    assert_eq!(lines.len(), 3, "{status}");
    assert_eq!(lines[0], "broken fake failed: the auth key was refused");
    assert!(
        lines[1].starts_with("home fake ready 100.64.0.1 (cannot forward port 9: port 9 is taken) forwards 22->127.0.0.1:"),
        "{status}"
    );
    assert!(lines[1].ends_with(",9->127.0.0.1:9"), "{status}");
    assert_eq!(lines[2], "slow fake joining");
}

#[tokio::test]
async fn a_second_daemon_on_the_same_socket_is_refused() {
    let r = rig(vec![]).await;
    let e = bind(&r.socket).unwrap_err();
    assert!(e.to_string().contains("already running"), "{e}");
}
