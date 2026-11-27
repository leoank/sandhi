---
id: PHS-20261121-07gk4r
type: phase
schema_version: 1
title: "Phase 1: connect and inbound, the end of tailnet switching"
project: PRJ-0001
status: active
since: 72b0907b4b4d1a01cb537c1d2f575d72c46c3a54
until: null               # the commit it finished at: yajna phase done
created: 2026-11-21
author: agent:claude
tags: []
---

## Goal
A machine reaches peers on every configured tailnet by name through `sandhi connect`
(ssh's `ProxyCommand`), and is published into tailnets it is not on by inbound forwarding,
so no fleet machine ever switches its system tailnet again. No root, no DNS.

## Progress
One item per piece of work, ticked when it lands; add items as the plan grows and keep the
done ones. The project page's *Progress* lists this phase on one line.
- [x] 1.1 The backend trait (dial, listen, describe; a peer list waits for phase 2's DNS)
  and the tailnet backend over `sandhi-tailnet`, one dutavas per configured tailnet; dials
  retried for 30 s after the dutavas comes up (ADR-0005)
- [x] 1.2 The config (backends, the names they own, inbound forwards, one table per kind)
  and the router: longest suffix, a suffix owned twice refused, no fallback (ADR-0005)
- [x] 1.3 `sandhi up`, `sandhi connect <host> <port>` on stdio and `sandhi status`, over a
  user-only unix socket and a one-line protocol (ADR-0005); tested against a fake backend
- [x] 1.4 Inbound forwarding: a backend listens on chosen ports and forwards to local
  services; the caller logged as the backend names them
- [x] 1.4a Smoke test on rogue, 2026-11-21: two ephemeral dutavas on leoank from one
  daemon; one reached the other's forward to a local HTTP server (41 ms, caller
  `sandhi-test-b (tag:neusis)`), and saarthi-hub answered an HTTP/2 preface on 7700
  ([[AGS-20261121-068w2d]])
- [x] 1.5 neusis: a home-manager module (launchd agent, systemd user service), auth keys
  as agenix secrets, ssh `ProxyCommand` for the backends' domains (ADR-0006; neusis
  `users/ank/sandhi.nix`); a first join waits for its key (ADR-0007)
- [x] 1.6 Checked live: rogue reaches ghost001 and curiosity on leoank while its system
  tailnet is runxi, and ghost001 is reachable on two tailnets at once
  ([[EXP-20261127-hsprp3]]); curiosity's identity kept on SPARKDATA (neusis ADR-0023,
  [[EXP-20261127-j1rwdg]]); `sandhi up` exits on SIGTERM with a stuck join
  ([[EXP-20261127-jwk3y3]])
- [ ] 1.7 curiosity keeps `sandhi-curiosity` across a reboot on an ISO built with neusis
  ADR-0023

## Notes
