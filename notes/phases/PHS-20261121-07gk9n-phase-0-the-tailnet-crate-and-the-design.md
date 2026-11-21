---
id: PHS-20261121-07gk9n
type: phase
schema_version: 1
title: "Phase 0: the tailnet crate and the design"
project: PRJ-0001
status: planned           # planned | active | done | abandoned
since: null               # the commit it started at: yajna phase start
until: null               # the commit it finished at: yajna phase done
created: 2026-11-21
author: agent:claude
tags: []
---

## Goal
The design decided and recorded (ADR-0001 to ADR-0004) and saarthi's tailnet crate in
this workspace, built and tested by the flake.

## Progress
One item per piece of work, ticked when it lands; add items as the plan grows and keep the
done ones. The project page's *Progress* lists this phase on one line.
- [x] 0.1 The repo, its notebook and the flake; the owner's choices of 2026-11-21
  ([[AGS-20261121-068w2d]])
- [x] 0.2 `sandhi-tailnet`, copied from saarthi `954bc23` and trimmed (ADR-0002); its unit
  tests pass under `nix flake check`
- [x] 0.3 ADR-0001 to ADR-0004 and `docs/architecture.md`
- [x] 0.4 neusis: ADR-0021 supersedes the tsmux part of its ADR-0008

## Notes
