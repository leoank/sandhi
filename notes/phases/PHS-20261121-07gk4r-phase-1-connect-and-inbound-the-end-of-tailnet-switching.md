---
id: PHS-20261121-07gk4r
type: phase
schema_version: 1
title: "Phase 1: connect and inbound, the end of tailnet switching"
project: PRJ-0001
status: planned           # planned | active | done | abandoned
since: null               # the commit it started at: yajna phase start
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
- [ ] 1.1 The backend trait (peers, dial, listen) and the tailnet backend over
  `sandhi-tailnet`; one identity per configured tailnet; the first dial after a start
  from saved state retried
- [ ] 1.2 The config (backends, names they own, inbound forwards) and the router: a name
  owned by one backend, no fallback, a domain owned twice rejected
- [ ] 1.3 `sandhi up` (the daemon) and `sandhi connect <host> <port>` on stdio, talking
  over a unix socket
- [ ] 1.4 Inbound forwarding: a backend listens on chosen ports and forwards to local
  services; the caller logged
- [ ] 1.5 neusis: a home-manager module (launchd agent, systemd user service), auth keys
  as agenix secrets, ssh `ProxyCommand` for the backends' domains
- [ ] 1.6 Checked live: rogue reaches ghost001 and curiosity on leoank while its system
  tailnet is runxi, and ghost001 is reachable on two tailnets at once

## Notes
