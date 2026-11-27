---
id: EXP-20261127-jwk3y3
type: experiment
schema_version: 1
title: Why does sandhi up not exit on SIGTERM after a join failed, and does a bounded runtime shutdown fix it?
project: PRJ-0001
status: concluded         # planned | active | paused | concluded | abandoned
outcome: positive         # null | positive | negative | inconclusive | infra
influenced_by: [EXP-20261127-j1rwdg]
created: 2026-11-27
author: agent:claude
protocols: []
materials: []
tags: []
review: pending
---

## Question
In [[RUN-20261127-j811rw]] a `sandhi up` whose join had failed kept running after SIGTERM
until it was killed. A service manager would wait out its stop timeout and kill it the same
way. What holds it, and does ending the runtime with a time limit fix it?

## Hypothesis and predictions
H1: `Tailnet::start` gives up on a join while libtailscale's blocking `tailscale_up` is
still running on tokio's blocking pool, holding a clone of the tailnet (so it is never
closed); on SIGTERM `run` returns, and `#[tokio::main]`'s runtime waits for that blocking
thread forever. If H1 is true we expect: a daemon with a join refused for want of a person
stays alive after SIGTERM, its process sample shows the main thread waiting on the runtime's
shutdown and a thread inside `tailscale_up`; and with the runtime shut down by
`shutdown_timeout` it exits within that limit. If false we expect a different stack (a
deadlock in `tailscale_close`, or the signal not received) and the fix not to help.

## Plan
1. Reproduce on rogue with the installed `sandhi`: a non-ephemeral identity joined with
   rogue's key, its device then logged out, would need the admin console; instead start a
   dutavas from an auth key that is not valid (`tskey-auth-not-a-key`), which a person
   would have to replace, and see whether its join fails and SIGTERM is ignored; sample
   the process (macOS `sample`).
2. Fix: build the runtime by hand and end it with `shutdown_timeout` after `run` returns.
3. The same script with the fixed binary: the daemon exits within the limit.

## Success criteria
Positive: reproduced, the sample shows the blocking pool wait, and the fixed binary exits
within the limit after SIGTERM. If the invalid key does not reproduce it, the run says so
and another trigger is planned before any fix.

## Runs
One line per run after its link, what was tried and what it showed; `yajna validate`
rejects a bare link here. Metric runs also land in results.tsv.
- [[RUN-20261127-jwkqeb]] an invalid key fails inside tailscale_up, which returns, and the daemon exits on SIGTERM: no reproduction
- [[RUN-20261127-jwwze0]] reproduced: with tsnet waiting on a login the daemon outlived SIGTERM, its main thread waiting on tokio's blocking pool
- [[RUN-20261127-jxq9p4]] with the runtime ended by shutdown_timeout the same daemon exited 5 s after SIGTERM, a healthy one at once

## Conclusion
H1 holds. `sandhi up` outlives SIGTERM when a join failed while libtailscale's
`tailscale_up` is still waiting (a removed identity waiting for a login): the main thread
then waits on tokio's blocking pool for that call forever ([[RUN-20261127-jwwze0]], sampled).
A join refused inside `tailscale_up` returns and does not trigger it
([[RUN-20261127-jwkqeb]]). Building the runtime by hand and ending it with a 5 s
`shutdown_timeout` fixes it: the same daemon exits 5 s after SIGTERM, a healthy one at once
([[RUN-20261127-jxq9p4]]).

For the project: a service manager's stop no longer waits out its timeout. The stuck
`tailscale_up` is left to the process's exit, not closed; tsnet could still be asked to
close a join it was given up on (in `sandhi-tailnet`, shared with saarthi), not done here.
