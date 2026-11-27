# ADR-0007: A first join waits for an auth key file that is not there yet, up to the join limit

Status: accepted
Date: 2026-11-27

## Context
On rogue the launchd agent from ADR-0006 started before home-manager's agenix agent had
decrypted the leoank key. The dutavas had no state yet, so `sandhi-tailnet` failed to read
the key, the backend was reported failed, and by ADR-0005 it stayed failed until the owner
restarted the service by hand. curiosity joins anew at every boot (its live disk keeps no
state) and meets the same race whenever the agenix user unit is slow, even ordered after it.

Alternatives: order the service after agenix in each service manager (systemd can, launchd
has no ordering between agents); retry every failed join (a revoked key or an admin
approval would retry forever, hiding what needs a person); exit and let the service
manager restart the daemon (every backend would rejoin for one missing file).

## Decision
- Before a tailnet dutavas joins without saved state, a configured `authkey_file` that does
  not exist yet is waited for, looked for every second; the backend reads as joining
  meanwhile.
- The wait ends with the same limit as an unanswered join (ten minutes, ADR-0005); then the
  backend fails with the file named.
- An identity that has joined before (its `tailscaled.state` exists) never waits: it needs no
  key, as `sandhi-tailnet` already assumes.
- The wait lives in sandhi's tailnet backend; the crate copied from saarthi stays as copied.

## Consequences
- A daemon started before its secrets settles by itself; other refusals still fail at once.
- A mistyped key path is reported after ten minutes rather than at once.
- The backend knows tsnet's state file name, beside the crate that also does.
