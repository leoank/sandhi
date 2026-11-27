---
id: EXP-20261127-hsprp3
type: experiment
schema_version: 1
title: Does sandhi carry ssh between the fleet's machines over leoank while their system tailnet is runxi?
project: PRJ-0001
status: active            # planned | active | paused | concluded | abandoned
outcome: null             # null | positive | negative | inconclusive | infra
influenced_by: []
created: 2026-11-27
author: agent:claude
protocols: []
materials: []
tags: []
review: pending
---

## Question
Phase 1.6 ([[PHS-20261121-07gk4r]]): with neusis's `sandhi-<host>` bundles switched in,
does rogue reach ghost001 and curiosity by ssh over leoank through `sandhi connect`, while
the system tailnet of every machine stays runxi, and is ghost001 reachable on both
tailnets at once?

## Hypothesis and predictions
H1: each machine's dutavas is on leoank as `sandhi-<host>` with sshd forwarded, so
`ssh -o ProxyCommand='sandhi connect %h %p' sandhi-<host>.leoank` lands on that machine.
If H1 is true we expect, from rogue: `sandhi status` ready on leoank; rogue's and ghost001's
system tailnet still runxi; a login on `sandhi-ghost001.leoank` and on `ghost001` (runxi)
presenting the same ed25519 host key; a login on `sandhi-curiosity.leoank`; each in a few
seconds. If false we expect `sandhi connect` to fail naming the peer (the dutavas is not
deployed or not named so), or the forward to refuse (sshd not on 127.0.0.1:22).

## Plan
1. The run's script, on rogue: record rogue's `sandhi status` and system tailnet.
2. ssh through sandhi to `sandhi-ghost001.leoank` and over runxi to `ghost001`, each with
   a known-hosts file of the run's own; on the far side print the hostname, `sandhi status`
   and the system tailnet; compare the two host keys.
3. ssh through sandhi to `sandhi-curiosity.leoank`, the same.
4. The ProxyCommand is passed with `-o`, so the run does not rest on rogue's
   hand-written `~/.ssh/config` including `config.d/sandhi`.

## Success criteria
All of: rogue reaches ghost001 and curiosity through sandhi over leoank; ghost001 answers
on runxi too with the same host key; rogue's system tailnet is runxi throughout. A machine
that is offline or not yet deployed makes the run `infra`, to be run again, not negative.

## Runs
One line per run after its link, what was tried and what it showed; `yajna validate`
rejects a bare link here. Metric runs also land in results.tsv.
- [[RUN-20261127-hsxqrz]] rogue reached ghost001 over leoank through sandhi (~1 s) and over runxi; curiosity gave no banner in 60 s; the script misread a root-only remote command as failure
- [[RUN-20261127-hv43pm]] ghost001 answered through sandhi and over runxi with the same host key; curiosity answered over runxi with its dutavas ready, yet not through sandhi
- [[RUN-20261127-hw411b]] curiosity reached through sandhi by its leoank address at once: forward and sshd work, the name does not
- [[RUN-20261127-hw8vay]] curiosity's dutavas is sandhi-curiosity-2: older ephemeral devices from earlier boots keep the name; leoank is tail73031.ts.net

## Conclusion
Written once, by a human, when status becomes `concluded`.
