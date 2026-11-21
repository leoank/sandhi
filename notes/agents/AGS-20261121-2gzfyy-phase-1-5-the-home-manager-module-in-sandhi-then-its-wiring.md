---
id: AGS-20261121-2gzfyy
type: agent_session
schema_version: 1
title: "Phase 1.5: the home-manager module in sandhi, then its wiring in neusis"
model: claude-opus-5-5
harness: claude-code
started: 2026-11-21T02:18+00:00
finished: null
created_entities: []
edited_entities: [AGS-20261121-068w2d, PHS-20261121-07gk4r]
outcome: pending          # done | failed | aborted | pending
created: 2026-11-21
author: agent:claude
---

## Task
Phase 1.5 ([[PHS-20261121-07gk4r]]): the home-manager module that runs `sandhi up` on
every machine, here, then its wiring in neusis (the input, the leoank backend with its
agenix key, ssh published inward, ssh's `ProxyCommand`), and pinning sandhi's checkout in
the yajna machine config there. Under review [[REV-20261121-2gzgx6]].

## What happened
- Closed the previous session's page ([[AGS-20261121-068w2d]]) as done.
- `homeModules.sandhi` (`nix/module.nix`) and `checks.module` (`nix/module-tests.nix`),
  after saarthi's node module; ADR-0006. Decided without asking: the config is passed to
  the daemon by store path (a change restarts the service) and also installed at the
  clients' default path; `stateDir` and `socket` are options too; the ssh
  `ProxyCommand` stays in neusis, not the module; a backend without a kind is an
  evaluation assertion.
- The check runs the real `sandhi status` on the rendered TOML; a doubly owned domain
  made it fail as it should (tried, then reverted).
