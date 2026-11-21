# ADR-0004: Connections reach the proxy at a loopback address per peer, then through a TUN

Status: accepted
Date: 2026-11-21

## Context
Plain TCP carries no hostname, so the address a connection arrives at must say which peer
it is for. Linux routes all of `127.0.0.0/8` to loopback; macOS has only `127.0.0.1` until
an alias is added to `lo0`, which takes root. A listener per address must list its ports.
A TUN with a fake address range and a userspace stack catches every port and UDP, at the
cost of a root daemon and a bigger first build. The owner chose the loopback path first,
the TUN after (2026-11-21).

## Decision
- Each peer gets a loopback address from a range sandhi owns; sandhi listens there on a
  declared port set (22, 80, 443 and those the config adds). On darwin a root helper adds
  and removes the `lo0` aliases.
- The TUN, with a fake address range and every port, follows as its own front end; the
  router and backends do not change for it.

## Consequences
- Until the TUN: TCP only, listed ports only, no mosh or HTTP/3 to peers.
- Some programs treat any `127.x` address as local (browsers' secure contexts, cookie
  scopes); noted, not handled.
