//! The config file (ADR-0005): the backends this machine is a member of, the names each
//! owns, and the local services each publishes inward. TOML, written by hand or by neusis.
//!
//! ```toml
//! [backends.leoank]
//! domains = ["tail1234.ts.net"]
//! forwards = [{ port = 22, to = "127.0.0.1:22" }]
//!
//! [backends.leoank.tailnet]
//! hostname = "rogue"
//! authkey_file = "/run/agenix/leoankTsAuthKey"
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Where each dutavas keeps its state, one directory per backend.
    pub state_dir: Option<PathBuf>,
    /// The control socket `sandhi connect` and `sandhi status` talk to.
    pub socket: Option<PathBuf>,
    #[serde(default)]
    pub backends: BTreeMap<String, BackendConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendConfig {
    /// DNS domains this backend owns: `<peer>.<domain>` goes to it with the name unchanged.
    /// `<peer>.<backend name>` always does, with the backend's name taken off.
    #[serde(default)]
    pub domains: Vec<String>,
    /// Local services published into this backend.
    #[serde(default)]
    pub forwards: Vec<Forward>,
    /// The kind of network, one table per kind; exactly one is set.
    pub tailnet: Option<TailnetConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TailnetConfig {
    /// The machine's name on the tailnet; this machine's hostname by default.
    pub hostname: Option<String>,
    /// The auth key, needed for the first join and after the tailnet logs the dutavas out.
    pub authkey_file: Option<PathBuf>,
    /// The tailnet removes the device soon after it goes offline: for tests.
    #[serde(default)]
    pub ephemeral: bool,
}

/// A local service published into a backend: callers on the backend reaching this
/// machine's `port` are connected to `to`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forward {
    pub port: u16,
    /// `host:port`, usually on loopback.
    pub to: String,
}

/// The kind of a backend, with its settings.
#[derive(Debug, Clone)]
pub enum Kind {
    Tailnet(TailnetConfig),
}

impl BackendConfig {
    pub fn kind(&self) -> Result<Kind> {
        match &self.tailnet {
            Some(t) => Ok(Kind::Tailnet(t.clone())),
            None => bail!("no kind of network given: add a [tailnet] table"),
        }
    }
}

impl Config {
    pub fn parse(text: &str) -> Result<Config> {
        let config: Config = toml::from_str(text)?;
        config.validate()?;
        Ok(config)
    }

    /// Reads `path`; a missing file is an empty config, for `sandhi connect` on a machine
    /// whose daemon uses the default socket.
    pub fn load(path: &Path) -> Result<Config> {
        match std::fs::read_to_string(path) {
            Ok(text) => Config::parse(&text).with_context(|| format!("{}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
            Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
        }
    }

    fn validate(&self) -> Result<()> {
        let mut owner: HashMap<String, &str> = HashMap::new();
        for (name, b) in &self.backends {
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                bail!("backend {name:?}: a name is lowercase letters, digits and dashes");
            }
            b.kind().with_context(|| format!("backend {name}"))?;
            // The backend's name is a one-label suffix of its own.
            for suffix in std::iter::once(name.clone()).chain(b.domains.iter().map(|d| normalise(d))) {
                if suffix.is_empty() {
                    bail!("backend {name}: an empty domain");
                }
                if let Some(other) = owner.insert(suffix.clone(), name) {
                    bail!("{suffix} is owned by both {other} and {name}: a name belongs to one backend");
                }
            }
            let mut ports = HashSet::new();
            for f in &b.forwards {
                if !ports.insert(f.port) {
                    bail!("backend {name}: port {} is forwarded twice", f.port);
                }
                if f.to.rsplit_once(':').and_then(|(_, p)| p.parse::<u16>().ok()).is_none() {
                    bail!("backend {name}: forward to {:?} has no port", f.to);
                }
            }
        }
        Ok(())
    }

    pub fn state_dir(&self) -> PathBuf {
        self.state_dir.clone().unwrap_or_else(|| xdg("XDG_STATE_HOME", ".local/state").join("sandhi"))
    }

    pub fn socket(&self) -> PathBuf {
        if let Some(s) = &self.socket {
            return s.clone();
        }
        match std::env::var_os("XDG_RUNTIME_DIR") {
            Some(d) if !d.is_empty() => PathBuf::from(d).join("sandhi.sock"),
            _ => self.state_dir().join("sandhi.sock"),
        }
    }
}

/// The config file's default place.
pub fn default_path() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("sandhi").join("config.toml")
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    match std::env::var_os(var) {
        Some(d) if !d.is_empty() => PathBuf::from(d),
        _ => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(fallback),
    }
}

/// A DNS name as the router compares it: lowercase, without leading or trailing dots.
pub fn normalise(name: &str) -> String {
    name.trim_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
        [backends.leoank]
        domains = ["Tail1234.ts.net."]
        forwards = [{ port = 22, to = "127.0.0.1:22" }]
        [backends.leoank.tailnet]
        hostname = "rogue"
        authkey_file = "/run/agenix/leoankTsAuthKey"

        [backends.runxi.tailnet]
    "#;

    #[test]
    fn a_config_names_backends_their_domains_and_forwards() {
        let c = Config::parse(EXAMPLE).unwrap();
        let leoank = &c.backends["leoank"];
        assert_eq!(leoank.forwards, vec![Forward { port: 22, to: "127.0.0.1:22".into() }]);
        assert!(matches!(leoank.kind().unwrap(), Kind::Tailnet(t) if t.hostname.as_deref() == Some("rogue")));
        assert!(c.backends["runxi"].domains.is_empty());
    }

    fn err(text: &str) -> String {
        format!("{:#}", Config::parse(text).unwrap_err())
    }

    #[test]
    fn a_name_owned_twice_is_refused() {
        let e = err(r#"
            [backends.a]
            domains = ["x.ts.net"]
            [backends.a.tailnet]
            [backends.b]
            domains = ["X.ts.net."]
            [backends.b.tailnet]
        "#);
        assert!(e.contains("x.ts.net is owned by both a and b"), "{e}");
        // a backend's name is one of its suffixes too
        let e = err(r#"
            [backends.a]
            domains = ["b"]
            [backends.a.tailnet]
            [backends.b.tailnet]
        "#);
        assert!(e.contains("owned by both"), "{e}");
    }

    #[test]
    fn a_backend_needs_a_kind_and_sound_forwards() {
        assert!(err("[backends.a]\ndomains = []").contains("no kind of network"));
        assert!(err("[backends.a]\nforwards = [{ port = 22, to = \"localhost\" }]\n[backends.a.tailnet]").contains("has no port"));
        assert!(
            err("[backends.a]\nforwards = [{ port = 22, to = \"a:1\" }, { port = 22, to = \"a:2\" }]\n[backends.a.tailnet]")
                .contains("forwarded twice")
        );
        assert!(err("[backends.A.tailnet]").contains("lowercase"));
        assert!(err("[backends.a.tailnet]\nauthkey = \"x\"").contains("unknown field"));
    }
}
