---
id: AGS-20261127-hsneay
type: agent_session
schema_version: 1
title: Phase 1.6 live across the fleet, and a first join that waits for its auth key
model: claude-opus-5-5
harness: claude-code
started: 2026-11-27T16:12+00:00
finished: null
created_entities: [EXP-20261127-hsprp3, RUN-20261127-hsxqrz, RUN-20261127-hv43pm, RUN-20261127-hw411b, RUN-20261127-hw8vay, EXP-20261127-j1rwdg, RUN-20261127-j1sbbh, RUN-20261127-j811rw, RUN-20261127-jg8x1f, EXP-20261127-jwk3y3, RUN-20261127-jwkqeb, RUN-20261127-jwwze0, RUN-20261127-jxq9p4, RUN-20261127-k89h5w]
edited_entities: [AGS-20261121-2gzfyy, PHS-20261121-07gk4r]
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
- Committed and pushed the key-race fix (`c03bafd`), the yajna bump with its refreshed skill
  (`f1716aa`) and 1.6's notes (`eef36e5`).
- [[EXP-20261127-j1rwdg]] before building curiosity's persistence: a non-ephemeral dutavas
  resumes from `tailscaled.state` alone ([[RUN-20261127-jg8x1f]]); an ephemeral one did not
  ([[RUN-20261127-j811rw]]); the first attempt hung on a fault of the script
  ([[RUN-20261127-j1sbbh]], aborted). Twice a test state directory with a node key was left
  under /tmp and deleted by hand. Left `sandhi-test-restore` on leoank for the owner.
  Conclusion drafted for review.
- neusis (its AGS-20261127-jkrp31): curiosity's identity on SPARKDATA (neusis ADR-0023), the
  leoank domain `tail73031.ts.net`, sandhi bumped; committed and pushed.
- neusis committed and pushed (its four commits); sandhi's notes pushed (`7b33d34`).
- [[EXP-20261127-jwk3y3]]: the SIGTERM hang reproduced with a join tsnet kept waiting on,
  sampled (the main thread waits on tokio's blocking pool), and fixed in `main.rs` with a
  hand-built runtime and a 5 s `shutdown_timeout`; the fixed binary exits in 5 s. Two
  ephemeral `sandhi-test-sigterm` devices joined, removed by the tailnet on stop. Conclusion
  drafted for review. Not committed yet.
- Committed and pushed the SIGTERM fix (`832d6a7`) and its notes (`45f1290`); bumped sandhi
  in neusis (`0b2292f`, pushed by the owner). The owner moved curiosity's identity by hand
  and deployed: it came back at the same address from `/var/lib/sandhi-ank`, and the save
  unit put it on SPARKDATA.
- [[RUN-20261127-k89h5w]]: rogue reached ghost001 and curiosity by name through sandhi and
  over runxi; [[EXP-20261127-hsprp3]] concluded positive as a draft for review. Ticked 1.5
  and 1.6; added 1.7, curiosity's name across a reboot on the new ISO.

