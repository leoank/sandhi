---
id: EXP-20261127-j1rwdg
type: experiment
schema_version: 1
title: Does a tsnet identity come back from its tailscaled.state alone, without the auth key?
project: PRJ-0001
status: concluded         # planned | active | paused | concluded | abandoned
outcome: positive         # null | positive | negative | inconclusive | infra
influenced_by: [EXP-20261127-hsprp3]
created: 2026-11-27
author: agent:claude
protocols: []
materials: []
tags: []
review: pending
---

## Question
curiosity's live disk loses its dutavas at every reboot, and a new device takes a new name
([[EXP-20261127-hsprp3]]). The plan in neusis is to save the dutavas's `tailscaled.state`
on SPARKDATA and copy it back at boot. Is that one file the whole identity: does a dutavas
started from a copy of it, in an otherwise empty state directory and with no auth key,
come back as the same device?

## Hypothesis and predictions
H1: tsnet keeps the node and machine keys and the login in `tailscaled.state`;
`profile-data/` and the logs are not needed to resume. If H1 is true we expect the second
start to be ready at the same tailnet address as the first, without reading a key. If
false we expect it to fail for want of a key (state not accepted) or to come up at a new
address (a new device).

## Plan
1. The run's script, on rogue, with the installed `sandhi`: one backend on leoank,
   `sandhi-test-restore`, ephemeral, joined with rogue's own sandhi key; wait until ready,
   record its address, stop it.
2. Copy only `leoank/tailscaled.state` into a fresh state directory; start again from it
   with `authkey_file` pointing at a file that does not exist; record its address.
3. Leaves one ephemeral device on leoank, gone soon after the run.

## Success criteria
Positive: the second start is ready at the first start's address. Negative: it fails or
comes up at another address. Either decides how curiosity's state is saved.

## Runs
One line per run after its link, what was tried and what it showed; `yajna validate`
rejects a bare link here. Metric runs also land in results.tsv.
- [[RUN-20261127-j1sbbh]] aborted: the script hung on its own daemon after the first join
- [[RUN-20261127-j811rw]] an ephemeral identity did not resume from tailscaled.state alone; confounded, as the tailnet may delete an ephemeral device on disconnect
- [[RUN-20261127-jg8x1f]] a non-ephemeral identity resumed from a copy of tailscaled.state alone, without a key, at its first address

## Conclusion
H1 holds: a dutavas started from a copy of its `tailscaled.state` alone, in an otherwise
empty state directory and with no auth key, comes back as the same device at the same
address ([[RUN-20261127-jg8x1f]]); `profile-data/` and the logs are not needed. That holds
for a non-ephemeral identity; an ephemeral one did not resume ([[RUN-20261127-j811rw]]),
most likely because the tailnet deleted it once it disconnected.

For the project: curiosity's dutavas can survive its live disk's reboots by saving that one
file on SPARKDATA and copying it back before the daemon starts, and it must not be
ephemeral. Also found: `sandhi up` may not exit on SIGTERM while a join is stuck.
