---
id: AGS-20261121-068w2d
type: agent_session
schema_version: 1
title: "Bootstrap sandhi: notebook, findings from saarthi, the first decisions and the tailnet crate"
model: claude-opus-5-5
harness: claude-code
started: 2026-11-21T00:10+00:00
finished: 2026-11-21T02:00+00:00
created_entities: [PHS-20261121-07gk9n, PHS-20261121-07gk4r, PHS-20261121-07gkgk, PHS-20261121-07gkvm]
edited_entities: [PHS-20261121-07gk4r]
outcome: done             # done | failed | aborted | pending
created: 2026-11-21
author: agent:claude
---

## Task
Start sandhi, the multi-network DNS and proxy the owner proposed to end tailnet switching
in the neusis fleet: critique the design, then set up the repo, its notebook, the first
decisions and saarthi's tailnet crate.

## What happened
- Read neusis's tsmux spec and ADR-0008, and saarthi's ADR-0015, ADR-0048, ADR-0049, the
  `saarthi-tailnet` crate and [[saarthi/EXP-20261112-rq8kgc]].
- Critiqued the owner's DNS + proxy design against device sharing, one fleet tailnet,
  several tailscaled daemons, proxy-aware programs only, and a gateway. The owner's choices
  of 2026-11-21: own repo and notebook; Rust unless Go were clearly faster or more mature
  (it is not, ADR-0002); a loopback address per peer then a TUN; both directions; no
  gateway; phones left to saarthi; the crate copied, shared later.
- Created the repo, the flake (libtailscale `ac7ed43`, dev shell, workspace and notebook
  checks), `crates/tailnet` copied from saarthi `954bc23` and trimmed, ADR-0001 to
  ADR-0004, `docs/architecture.md`, the project page and four phases.
- Renamed neumesh to sandhi before the first commit (the owner's choice, after names on
  the theme of yajna, saarthi, manthan, nitya, panini), and named a backend's per-network
  identity its dutavas (दूतावास, an embassy). Created `github:leoank/sandhi`, public.
- Phase 1, items 1.1 to 1.4 (the owner, 2026-11-21: "start phase 1"): the `sandhi` crate
  (config, router, backend trait, tailnet dutavas, daemon, client, CLI) with 14 unit and
  integration tests against a fake backend, `packages.sandhi` in the flake, ADR-0005; a
  smoke test on rogue with two ephemeral dutavas on leoank, inbound and outbound.
