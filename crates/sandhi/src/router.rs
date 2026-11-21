//! Which backend a name belongs to (ADR-0001): the longest suffix that matches, owned by one
//! backend only (the config refuses a suffix owned twice), and no fallback to another.

use crate::config::{Config, normalise};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    /// The backend that owns the name.
    pub backend: String,
    /// The name to dial on that backend.
    pub host: String,
}

#[derive(Debug, Clone)]
struct Suffix {
    suffix: String,
    backend: String,
    /// `<peer>.<backend name>`: the backend's name is taken off before dialling.
    alias: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Router {
    /// Longest first.
    suffixes: Vec<Suffix>,
}

impl Router {
    pub fn new(config: &Config) -> Router {
        let mut suffixes = Vec::new();
        for (name, b) in &config.backends {
            suffixes.push(Suffix { suffix: name.clone(), backend: name.clone(), alias: true });
            for d in &b.domains {
                suffixes.push(Suffix { suffix: normalise(d), backend: name.clone(), alias: false });
            }
        }
        suffixes.sort_by_key(|s| std::cmp::Reverse(s.suffix.len()));
        Router { suffixes }
    }

    pub fn route(&self, host: &str) -> Option<Route> {
        let host = normalise(host);
        self.suffixes.iter().find_map(|s| {
            let peer = host.strip_suffix(&s.suffix)?.strip_suffix('.')?;
            if peer.is_empty() {
                return None;
            }
            Some(Route {
                backend: s.backend.clone(),
                host: if s.alias { peer.to_string() } else { host.clone() },
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> Router {
        Router::new(
            &Config::parse(
                r#"
                [backends.leoank]
                domains = ["tail1234.ts.net"]
                [backends.leoank.tailnet]
                [backends.runxi]
                domains = ["tail9999.ts.net"]
                [backends.runxi.tailnet]
                [backends.wide]
                domains = ["ts.net"]
                [backends.wide.tailnet]
            "#,
            )
            .unwrap(),
        )
    }

    fn route(host: &str) -> Option<(String, String)> {
        router().route(host).map(|r| (r.backend, r.host))
    }

    fn some(backend: &str, host: &str) -> Option<(String, String)> {
        Some((backend.into(), host.into()))
    }

    #[test]
    fn a_domain_routes_the_whole_name_to_its_owner() {
        assert_eq!(route("ghost001.tail1234.ts.net"), some("leoank", "ghost001.tail1234.ts.net"));
        assert_eq!(route("GHOST001.Tail1234.ts.net."), some("leoank", "ghost001.tail1234.ts.net"));
        assert_eq!(route("ghost001.tail9999.ts.net"), some("runxi", "ghost001.tail9999.ts.net"));
    }

    #[test]
    fn a_backend_name_routes_the_peer_alone() {
        assert_eq!(route("ghost001.leoank"), some("leoank", "ghost001"));
        assert_eq!(route("ghost001.runxi"), some("runxi", "ghost001"));
    }

    #[test]
    fn the_longest_suffix_wins_and_nothing_falls_back() {
        assert_eq!(route("x.tail1234.ts.net"), some("leoank", "x.tail1234.ts.net"));
        assert_eq!(route("x.other.ts.net"), some("wide", "x.other.ts.net"));
        assert_eq!(route("example.com"), None);
        assert_eq!(route("leoank"), None);
        // a domain's own apex is not a peer of it, but is a name under a shorter domain
        assert_eq!(route("tail1234.ts.net"), some("wide", "tail1234.ts.net"));
        assert_eq!(route("ts.net"), None);
        assert_eq!(route("xleoank"), None);
    }
}
