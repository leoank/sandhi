# ADR-0005: A TOML config with a table per kind of network, and a one-line protocol on a user-only socket

Status: accepted
Date: 2026-11-21

## Context
Phase 1 needs a config neusis can render and a person can read, and a way for
`sandhi connect`, run by ssh once per connection, to reach the daemon that holds the
dutavas. The backend layer must take kinds of network other than tailnets without the
config or the router changing (ADR-0001).

## Decision
- The config is TOML at `$XDG_CONFIG_HOME/sandhi/config.toml`. Each `[backends.<name>]`
  has `domains` and `forwards` (`{ port, to }`), common to every kind, and exactly one
  table for its kind, `[backends.<name>.tailnet]` (`hostname`, `authkey_file`,
  `ephemeral`). A new kind is a new table. Unknown fields are errors.
- A backend's name is one of its suffixes: `<peer>.<name>` goes to it with the name taken
  off, `<peer>.<domain>` with the name unchanged. The longest suffix wins, a suffix owned
  twice is refused when the config loads, and nothing falls back (ADR-0001).
- The daemon listens on a unix socket (`$XDG_RUNTIME_DIR/sandhi.sock`, else in the state
  directory), mode 0600 in a 0700 directory: only its user can use its dutavas. The client
  sends one line, `connect <host> <port>` or `status`. To a connect the daemon answers
  `ok` and splices from then on, or `error <why>`; a status is one line per backend.
- Backends join in the background while the socket already answers; a connect to a
  backend still joining waits up to a minute. A join refused for want of a person (a used
  or revoked key, an admin's approval) fails within seconds and is reported by `status`
  and to every connect, as is a join unanswered for ten minutes.
- For 30 s after a dutavas comes up its dials are retried every 500 ms, since the netmap
  arrives after the join ([[saarthi/EXP-20261112-rq8kgc]]).
- An inbound forward connects each caller to `to` over TCP and logs who called, as the
  backend names them.

## Consequences
- A per-connection `sandhi connect` costs one process and one local socket hop, not a
  join: the dutavas stays up in the daemon.
- A peer name that does not exist fails only after the 30 s settling window while the
  dutavas is new (seen live: 22 s); later it fails at once.
- macOS caps a socket path at 104 bytes; a long `state_dir` needs `socket` set to a
  shorter path.
- A failed backend stays failed until the daemon restarts; the service manager's restart
  is the retry.
