# ADR-0003: DNS answers for the real tailnet names, never the system tailnet's, with stable addresses

Status: accepted
Date: 2026-11-21

## Context
A zone of sandhi's own (`ghost001.leoank.<zone>`) cannot collide with anything, but TLS
breaks: a peer's certificate is for its MagicDNS name, `ghost001.tail27b080.ts.net`. The
system tailscaled already claims `ts.net` (on darwin through `/etc/resolver/ts.net`, which
neusis manages), and resolvers on both platforms prefer the longest matching domain:
macOS's `/etc/resolver/<domain>` files (which take a `port`), and systemd-resolved's
routing domains. A program caches what DNS told it, so an address that means another peer
after a restart sends traffic to the wrong machine.

## Decision
- sandhi answers for each tailnet's own domain (`<tailnet>.ts.net`), registered with the
  OS per domain: an `/etc/resolver/<tailnet>.ts.net` file on darwin, a routing domain
  `~<tailnet>.ts.net` on Linux. Short aliases (`<peer>.<backend name>`) may sit on top.
- It never answers for the domain of the tailnet the system tailscaled is on; that tailnet
  stays the system's.
- A peer's local address is derived from (backend, peer) and saved, so it is the same
  after a restart; answers carry a short TTL.
- Only IPv4 (A) answers while the front end is loopback; AAAA gets an empty answer, so
  programs do not try IPv6 first and stall.

## Consequences
- HTTPS to a peer works with its real certificate.
- Another backend kind brings its own domains; the router rejects a domain owned twice.
