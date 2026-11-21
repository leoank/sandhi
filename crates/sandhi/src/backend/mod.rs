//! A backend is one network this machine is a member of, through its dutavas there
//! (ADR-0001): it dials the network's peers for this machine and receives the network's
//! callers on its behalf. Nothing above this trait knows what kind of network it is.

pub mod tailnet;

use std::future::Future;
use std::io;
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;

/// A byte stream to or from a peer.
pub trait Io: AsyncRead + AsyncWrite + Send + Unpin {}
impl<T: AsyncRead + AsyncWrite + Send + Unpin> Io for T {}
pub type BoxIo = Box<dyn Io>;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A connection from a peer to this machine.
pub struct Inbound {
    pub io: BoxIo,
    /// Who is calling, as the network names them, for the log.
    pub caller: String,
}

pub trait Backend: Send + Sync {
    /// A connection to `host:port`, `host` as the network names it.
    fn dial<'a>(&'a self, host: &'a str, port: u16) -> BoxFuture<'a, io::Result<BoxIo>>;

    /// Connections from the network to this machine's `port`, until the receiver is
    /// dropped or listening fails.
    fn listen(&self, port: u16) -> io::Result<mpsc::Receiver<Inbound>>;

    /// One line about the dutavas for `sandhi status`: its addresses on the network.
    fn describe(&self) -> String;
}
