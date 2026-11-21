---
id: PHS-20261121-07gkgk
type: phase
schema_version: 1
title: "Phase 2: DNS and a loopback address per peer"
project: PRJ-0001
status: planned           # planned | active | done | abandoned
since: null               # the commit it started at: yajna phase start
until: null               # the commit it finished at: yajna phase done
created: 2026-11-21
author: agent:claude
tags: []
---

## Goal
Programs that know nothing of proxies reach peers by their real tailnet names: a DNS
server answers with a stable loopback address per peer, where sandhi listens on a listed
port set (ADR-0003, ADR-0004).

## Progress
One item per piece of work, ticked when it lands; add items as the plan grows and keep the
done ones. The project page's *Progress* lists this phase on one line.
- [ ] 2.1 Peer lists kept current: a peer-change notification exported from libtailscale,
  or polling `status_json`
- [ ] 2.2 Stable address allocation per (backend, peer), saved across restarts
- [ ] 2.3 The DNS server: A answers for each backend's domains, empty AAAA, short TTL
- [ ] 2.4 Loopback listeners per peer; the darwin root helper for `lo0` aliases
- [ ] 2.5 neusis: `/etc/resolver/<domain>` on darwin, resolved routing domains on Linux,
  never the system tailnet's domain

## Notes
