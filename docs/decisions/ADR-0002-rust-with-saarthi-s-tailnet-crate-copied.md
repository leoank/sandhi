# ADR-0002: Rust, with saarthi's tailnet crate copied rather than shared

Status: accepted
Date: 2026-11-21

## Context
tsnet is Go; saarthi embeds it from Rust through libtailscale's C API, a hand-written sys
layer and a static archive built by Nix (saarthi ADR-0015, ADR-0048), and has proved it:
several identities in one process, dials by MagicDNS name, the caller named from
`status_json`, close-on-exec descriptors at `ac7ed43`, keyless restarts from saved state
([[saarthi/EXP-20261112-rq8kgc]], [[saarthi/EXP-20261016-rq1406]]).

Go would have the tsnet API whole (a peer-change watch, `LocalClient`, UDP) and gVisor's
netstack for the TUN, the most mature userspace stack. Rust has hickory for DNS, as good as
Go's, and younger userspace stacks. Performance does not decide it: the proxy waits on the
network and tsnet's netstack is the ceiling either way; libtailscale adds one socketpair
hop per connection. The owner asked for Go only if it were clearly faster or its ecosystem
clearly more mature, and for the findings to be shared with saarthi; other backends need
not be Go.

## Decision
- Rust, edition 2024, one cargo workspace.
- `crates/tailnet` (`sandhi-tailnet`) is a copy of `saarthi-tailnet` at saarthi
  `954bc23`, trimmed of what only saarthi uses: tonic's connector and `Connected` impl, and
  the Android log directory. libtailscale is built by the flake at `ac7ed43` as saarthi
  does and linked from `LIBTAILSCALE_LIB_DIR`.
- Sharing one crate between the two projects waits until sandhi is stable (the owner,
  2026-11-21).
- What the C API lacks (a peer-change notification for DNS, UDP if needed) is added as
  Go exports of our own beside libtailscale's, as saarthi adds `nix/libtailscale-android.go`.

## Consequences
- About 25 MB of libtailscale per binary; builds need Go through Nix and fail outside the
  dev shell with a message naming the variable.
- Fixes to the tailnet code have to be carried to saarthi by hand until the crates merge.
- The TUN front end will stand on a Rust userspace stack; that choice needs its own
  experiment.
