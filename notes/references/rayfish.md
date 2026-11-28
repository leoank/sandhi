---
type: reference
schema_version: 1
citekey: rayfish
title: "Rayfish: a peer-to-peer mesh VPN over iroh"
authors: Rayfish contributors
year: 2026
doi: null
status: read              # to-read | skimmed | read
role: method              # method | background | key-result | tool
created: 2026-11-27
author: agent:claude
review: pending
tags: [iroh, dns, tun, prior-art]
---

Studied 2026-11-27 at `a371852` (0.5.19, 2026-11-26) from its module docs, its `AGENTS.md`
and the key code paths ([[AGS-20261127-n8p5w3]]); about 141,000 lines of Rust, not read
line by line, nothing built or run. Paths below are in that tree. MPL-2.0: a file copied
from it stays MPL-2.0 with its source open; files of our own, and ideas re-implemented,
carry no obligation.

## What it says
A mesh VPN with no control server. Each device is an iroh endpoint; its overlay address is
IPv6 in `200::/7`, derived from the endpoint id (`0x02 ‖ blake3(hex id)[..15]`,
`src/membership.rs:431`), never allocated or sent. A *network* is an Ed25519 keypair: whoever
holds the secret is its coordinator; the roster is one canonical msgpack `GroupBlob`
fetched with iroh-blobs, and the network key signs a pkarr record (`_rayfish`: the blob's
hash, seed peers, the minimum wire version) published to n0's pkarr relay and the Mainline
DHT (`src/dht.rs`). Peers only trust the signed record; gossip says "look again", never
"here is the roster". Joining is by one-time invite, live approval, or a reusable key whose
hash sits in the blob; devices of one person are tied by a user-signed `DeviceCert`.

It is a root daemon with a kernel TUN (tun-rs), a packet firewall, a magic DNS inside the
TUN at `200::53`, exit nodes, keyless mesh SSH, file transfer and Android/macOS/Windows
apps. A **rootless** Linux mode replaces the TUN with smoltcp and offers SOCKS5/HTTP
proxies, `--forward` and `--expose` (`src/rootless/`). Its lib crate is "not a stable
public API" (`src/lib.rs:1`) and it builds against forks of iroh, noq-udp and netdev
through `[patch.crates-io]` (`Cargo.toml:247`), which a dependent crate would not inherit.
No Nix.

## What it means for this work
[[PRJ-0001]]. A reference to learn from and copy small pieces of, not a dependency.

**An iroh backend.** Nothing in rayfish dials a peer's TCP port as a stream: services are
reached through the TUN (or its smoltcp stand-in), and `ray connect` is a friend request,
not a splice. Our `dial(peer, port)` and inbound forwards over iroh are ours to write: an
ALPN such as `sandhi/tcp/1`, `open_bi`, a port header, checked against who may call. What
to take from rayfish for it:
- The endpoint: one per machine for every network (`src/transport.rs:420`), n0 relays
  augmented or replaced by our own (`build_relay_mode`, `:806`), discovery layered as
  pkarr/DNS, their own Mainline lookup (`src/dht/mainline.rs`), a warm cache of paths that
  worked (`src/endpoint_hints.rs`) and mDNS on the LAN (`src/daemon/mdns.rs`).
- The secret key as 32 raw bytes in the state directory (`src/identity.rs:15`): the iroh
  dutavas's state, like `tailscaled.state` today.
- Names: no global naming; a name resolves in a signed roster (`resolve_peer_selector`,
  `src/membership.rs:252`). Our router's `<peer>.<backend>` fits; the open question is who
  signs our roster (a neusis-rendered list of endpoint ids is the simplest start).
- Hide tailnet addresses from iroh: their `UnderlayMonitor` drops CGNAT and Tailscale's ULA
  from advertised addresses (`src/transport.rs:521`), so a public pkarr record never names
  a tailnet and iroh never tunnels through one by accident. sandhi runs beside tailscaled
  on every machine, so this applies.
- Wire rules (`AGENTS.md`, *Wire protocol*): the ALPN is the only version gate, dial the
  newest and step down on mismatch (`src/transport.rs:865`); a handler that sends a final
  reply finishes the send stream and then waits, bounded, on `connection.closed()`, or the
  peer may never read it.
- Reconnect: one dial per peer in flight (`src/daemon/mesh_dialer.rs`), a 10 s dial
  timeout, backoff; on "invalid peer certificate" forget the peer's direct hints, since a
  stale NAT mapping now reaches another device and iroh fails the whole dial on it
  (`src/transport.rs:905`). A connection id guards against an old connection's late close
  evicting its replacement (`src/forward.rs:500`).

**Phase 2, DNS and loopback addresses.** Their design is IPv6-only with no A records, the
opposite of our 127.x plan, so the lessons matter more than the code:
- Tailscale drops packets sourced from `100.64/10` on any other interface; their first
  magic DNS at 100.100.100.53 died for it (`src/dns/mod.rs:44`). Never answer from, or
  route through, a CGNAT address outside tailscale0.
- Never claim a bare name as a resolver domain on macOS: `leoank` as a supplemental domain
  would also capture a public `leoank` TLD lookup. A `.ts.net` subdomain is safe;
  `<peer>.leoank` needs care (`src/dns/config/macos.rs:154`).
- Answer NODATA, not NXDOMAIN, for the record type we do not serve (AAAA for us), or stubs
  fail the A query too; a miss is NXDOMAIN with our own SOA and a short negative TTL so a
  name is not cached dead for a day (`src/dns/mod.rs:262-283`). Positive TTL 60.
- Linux: systemd-resolved over D-Bus, `SetLinkDNS`/`SetLinkDomains` with `~domain`,
  DefaultRoute, DNSSEC, DoT, LLMNR and mDNS off (`src/dns/config/linux_backend.rs:172`,
  `:362`), used only when `/etc/resolv.conf` really points at resolved's stub (`:279`),
  else the routing domains silently do nothing. In neusis this is declarative, but the
  check is worth a test.
- macOS: they use SCDynamicStore session keys (gone when the process exits) rather than
  `/etc/resolver`; configd asks a supplemental resolver for AAAA only with a confirmed
  service id and a sibling IPv6 key (`macos.rs:176`). We serve A from loopback, so
  `/etc/resolver/<domain>` should do; an experiment should confirm it.
- Never forward to 100.100.100.100 without a loop breaker (`src/dns/resolver.rs:34`).

**Phase 3, a TUN.** Answers our open question in part: tun-rs for the device, smoltcp
for a userspace stack (theirs is TCP and IPv6 only; we would add IPv4 and UDP, which they
never tested). Their `TunRead`/`TunWrite` traits let one forwarder drive the kernel TUN and
smoltcp alike (`src/rootless/stack.rs:61`); one task owns every smoltcp socket and hands
out `DuplexStream`s (`:187`). Kernel lessons: on Linux set the address after the per-device
sysctls and the route after link-up, with the route's MTU, since a TUN sends no Packet Too
Big (`src/tun.rs:480`, `:634`); macOS utun drops its route on down/up (`:815`); a read error
on one packet must not end the loop (`src/forward.rs:638`); never put a TUN or socket
behind a mutex, split the halves. Lazy dial with bounded buffers is 60 lines
(`src/forward/lazy_dial.rs`). Compare ipstack and netstack-smoltcp before choosing.

**The daemon.** Ideas for `sandhi up`, `status` and their tests:
- Two tiers of cancellation: the process's, and a child per network minted fresh on each
  up (`src/daemon/mod.rs:5`), the shape of per-backend lifecycles.
- A `FatalExit`: a dead subsystem records why and ends the daemon non-zero, so the service
  manager restarts it, instead of a daemon alive and doing nothing (`src/daemon/mod.rs:670`).
  A tsnet that dies should do this.
- One instance: `flock` before touching anything, probe a live socket before unlinking it,
  remove the socket on exit only if its inode is still ours (`bootstrap.rs:1016`).
- The client checks the server's `peer_cred` uid before trusting a socket (`src/main.rs:174`);
  readers that create nothing (`load_for_read`).
- Every child process waited on with a bound (`src/child.rs`); they would still hang on a
  stuck `spawn_blocking` as we did, our `shutdown_timeout` ([[EXP-20261127-jwk3y3]]) is
  the stricter fix.
- Linux suspend: `CLOCK_MONOTONIC` stops in suspend, so compare with `CLOCK_BOOTTIME` and
  tell the network stack it changed (`bootstrap.rs:2010`). Worth knowing for tsnet on a
  laptop.
- Status per peer: Active, Idle or Offline (Offline only when a reach actually failed), the
  path Direct or Relay with RTT, the daemon's version, a reason string for a backend that
  is down; `status --json` exits non-zero without a daemon (`ray-proto/src/ipc.rs:1384`,
  `src/cli/status.rs:449`). Their CHANGELOG has three fixes for "connected" peers carrying
  no traffic: probe with data.
- Tests: the real binary as a normal user in a temporary state directory (`tests/rootless.rs`);
  e2e over SSH on real hosts with real relays, scenarios `churn`, `restore-offline`,
  `mixed-version`, `reliability`, each ending with a sweep of the logs for panics
  (`tests/e2e/README.md`). Docker gives no NAT, so no hole punching is tested there.

Not for us: mesh SSH (root, trusts the kernel's source address; we forward to sshd, whose
keys are the auth), exit nodes, file transfer, the apps, `ray apply` (a fleet tool; neusis
is ours), the tailscaled cgroup firewall (needs root; only relevant with a TUN overlay).

## Doubts and follow-ups
- Which iroh to build on: upstream 1.3 lacks the fixes in their fork (a socket hook, a home
  relay release, GSO); check what an iroh backend needs before choosing.
- Who signs an iroh backend's roster, and whether sandhi needs coordinators, invites and
  pkarr publication at all when neusis can render the member list.
- `/etc/resolver/<domain>` with A records from 127.x on macOS, and resolved routing domains
  beside tailscaled's own on Linux: experiments for phase 2.
- smoltcp with IPv4 and UDP against ipstack and netstack-smoltcp: phase 3's 3.1.
