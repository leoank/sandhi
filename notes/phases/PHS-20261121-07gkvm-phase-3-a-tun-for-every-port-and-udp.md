---
id: PHS-20261121-07gkvm
type: phase
schema_version: 1
title: "Phase 3: a TUN for every port and UDP"
project: PRJ-0001
status: planned           # planned | active | done | abandoned
since: null               # the commit it started at: yajna phase start
until: null               # the commit it finished at: yajna phase done
created: 2026-11-21
author: agent:claude
tags: []
---

## Goal
Every TCP port and UDP to every peer, through a TUN with a fake address range and a
userspace stack, as one more front end; the router and backends unchanged (ADR-0004).

## Progress
One item per piece of work, ticked when it lands; add items as the plan grows and keep the
done ones. The project page's *Progress* lists this phase on one line.
- [ ] 3.1 An experiment: which Rust userspace stack, and a TUN on darwin and Linux
- [ ] 3.2 The TUN front end

## Notes
