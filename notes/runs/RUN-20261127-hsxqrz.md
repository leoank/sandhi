---
id: RUN-20261127-hsxqrz
type: run
schema_version: 1
kind: compute               # bench | compute | assembly | calibration
project: PRJ-0001
experiment: EXP-20261127-hsprp3
protocols: []
status: done              # planned | in_progress | done | aborted
created: 2026-11-27
author: agent:claude
script: notes/runs/RUN-20261127-hsxqrz/run.sh         # executable that re-creates this run, relative to the repo root
commit: d4e4d7117e944def987dd54055af01a103f95fe0
host: rogue
outputs:
  - path: notes/runs/RUN-20261127-hsxqrz/run.log
    sha256: 275771fdb7bd76313d1660a131e712433365a173a4decc767e08b115bcdaa0bc
  - path: notes/runs/RUN-20261127-hsxqrz/known_hosts
    sha256: 62b20c23a9f278ce336433175396ff1fe4befd075d465860c019708b6538dafc
---

## Notes
Ran on rogue at 16:13Z, 2026-11-27, with the neusis sandhi bundles switched in on rogue
and ghost001. The script was not yet committed: `commit` is the HEAD it ran beside, and
the script as run is the one in this directory.

- rogue: `sandhi-rogue` ready on leoank; the system tailnet is runxi (`cs-lab-ts`).
- `ssh sandhi-ghost001.leoank` through `sandhi connect`: logged in on ghost001 in about
  1 s; there `sandhi-ghost001` is ready on leoank with 22 forwarded. **Reached.**
- `ssh ghost001` over runxi: logged in too, so ghost001 answers on both tailnets at once.
- Both are marked FAILED only because the remote `tailscale switch --list` needs root on
  Linux (exit 1 after the login): a fault of the script, fixed in the next run.
- ghost001's runxi host key did not land in the run's known-hosts file: ssh verified it
  against a system-wide file, so the two keys were not compared here.
- `ssh sandhi-curiosity.leoank`: the ProxyCommand started but no ssh banner came in 60 s.
  Not told apart here: curiosity offline or not deployed, or sandhi not up on it.

## Reproduce
Run `script` from the repo root in the project's dev shell at `commit`; it must produce the
same outputs. Note here what it cannot control.

## Checklist
- [x] ghost001 reached over leoank through sandhi and over runxi
- [ ] host keys compared (next run)
- [ ] curiosity (next run)
