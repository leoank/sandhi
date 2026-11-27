---
id: AGS-20261127-hsneay
type: agent_session
schema_version: 1
title: Phase 1.6 live across the fleet, and a first join that waits for its auth key
model: claude-opus-5-5
harness: claude-code
started: 2026-11-27T16:12+00:00
finished: null
created_entities: [EXP-20261127-hsprp3, RUN-20261127-hsxqrz, RUN-20261127-hv43pm, RUN-20261127-hw411b, RUN-20261127-hw8vay]
edited_entities: [AGS-20261121-2gzfyy]
outcome: pending          # done | failed | aborted | pending
created: 2026-11-27
author: agent:claude
---

## Task
Phase 1.6 ([[PHS-20261121-07gk4r]]): check live that rogue reaches ghost001 and curiosity
over leoank through sandhi while their system tailnet is runxi, as an experiment
([[EXP-20261127-hsprp3]]). And the race the owner hit on rogue after switching: the
launchd agent came up before agenix had decrypted the key, the backend failed and needed
a kickstart.

## What happened
- Closed [[AGS-20261121-2gzfyy]] as done; neusis had the two sandhi commits on main
  already (rebased as `cdb7eb4`, `6a355d7`), so the push was done.
- The key race: a first join now waits up to the join limit for a key file not there yet
  (ADR-0007), in sandhi's tailnet backend with three unit tests; the copied crate is
  untouched. Not committed yet.
- [[EXP-20261127-hsprp3]], four runs from rogue: ghost001 reached through sandhi over
  leoank and over runxi with the same host key, rogue and ghost001 staying on runxi;
  curiosity reached through sandhi only by address, because its dutavas is
  `sandhi-curiosity-2` while older devices from earlier boots keep `sandhi-curiosity`.
  The first and third scripts had faults, kept as run and fixed in the next.
- Proposed conclusion, for the owner: positive for ghost001; for curiosity the mechanism
  works and its naming does not; an identity that survives curiosity's reboots (state on
  SPARKDATA) would fix the name, not ephemeral.

