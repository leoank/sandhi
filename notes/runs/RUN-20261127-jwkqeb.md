---
id: RUN-20261127-jwkqeb
type: run
schema_version: 1
kind: compute               # bench | compute | assembly | calibration
project: PRJ-0001
experiment: EXP-20261127-jwk3y3
protocols: []
status: done              # planned | in_progress | done | aborted
created: 2026-11-27
author: agent:claude
script: notes/runs/RUN-20261127-jwkqeb/run.sh         # executable that re-creates this run, relative to the repo root
commit: 7b33d345926ff0efdc246c91664257647486008d
host: rogue
outputs:
  - path: notes/runs/RUN-20261127-jwkqeb/run.log
    sha256: 2f7f5ece0264bc159f9e0e7d9079165b6648a87e712694b38ae6d652e127b185
---

## Notes
On rogue at 17:12Z with the installed `sandhi` (567ab05's), the script not yet committed.
- An auth key that is not valid: the join failed at once, "tsnet.Up: backend: invalid key",
  returned by `tailscale_up` itself; after SIGTERM the daemon **exited at once**.
- Not reproduced, and consistent with H1: here `tailscale_up` returned, so nothing was left
  on the blocking pool. The hang needs a join refused while `tailscale_up` keeps waiting
  (the login-needed state of [[RUN-20261127-j811rw]]); repeated with that trigger.

## Reproduce
Run `script` from the repo root in the project's dev shell at `commit`; it must produce the
same outputs. Note here what it cannot control.

## Checklist
- [x] invalid key: exits on SIGTERM (no reproduction)
