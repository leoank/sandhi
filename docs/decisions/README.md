# Decisions

One architecture decision record per file, `ADR-0001-short-title.md`, numbered in order
and never renumbered. A decision that changes an earlier one supersedes it; both stay.

| ADR | Decision | Status |
|-----|----------|--------|
| [ADR-0001](ADR-0001-dns-and-a-proxy-in-layers-in-both-directions.md) | DNS plus a proxy, as backends, a router and front ends, in both directions; no gateway | accepted |
| [ADR-0002](ADR-0002-rust-with-saarthi-s-tailnet-crate-copied.md) | Rust, saarthi's tailnet crate copied until sandhi is stable | accepted |
| [ADR-0003](ADR-0003-real-tailnet-names-and-stable-addresses.md) | The real tailnet names, never the system tailnet's, with stable addresses | accepted |
| [ADR-0004](ADR-0004-a-loopback-address-per-peer-then-a-tun.md) | A loopback address per peer with listed ports, then a TUN | accepted |
| [ADR-0005](ADR-0005-a-toml-config-and-a-line-protocol-on-a-user-socket.md) | A TOML config with a table per kind of network; `connect` and `status` as one line on a user-only socket | accepted |

Each record:

```markdown
# ADR-0001: <the decision, as a sentence>

Status: proposed | accepted | superseded by ADR-xxxx
Date: YYYY-MM-DD

## Context
What forced the decision, and the alternatives considered.

## Decision
What was chosen.

## Consequences
What follows, good and bad.
```
