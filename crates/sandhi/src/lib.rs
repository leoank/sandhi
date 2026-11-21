//! sandhi: one machine on several networks at once (ADR-0001). Backends reach each network
//! through this machine's dutavas there, the router picks the backend that owns a name, and
//! the daemon serves `sandhi connect` and the inbound forwards.

pub mod backend;
pub mod client;
pub mod config;
pub mod daemon;
pub mod router;
