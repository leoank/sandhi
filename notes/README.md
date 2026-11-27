---
id: PRJ-0001
type: project
schema_version: 1
title: sandhi - reach every network from every machine
status: active            # active | paused | concluded | moved | archived
code: .               # code/<name> here, or a repository URL once it moves out
created: 2026-11-21
author: ank
tags: []
---

## Introduction
sandhi (सन्धि: a joining, a treaty) lets one machine be on several networks at once
without switching anything at the OS level: it reaches the peers of every network it is
configured for, and publishes the machine's own services into each of them. On each network the machine is represented
by its **dutavas** (दूतावास, an embassy). Tailnets come first, with an embedded tsnet
identity as the dutavas; other kinds of network come later as backends of their own.

It exists because a Tailscale client runs one tailnet at a time (neusis ADR-0008), and
moving the neusis fleet between tailnets in November 2026 meant switching every machine by
hand, in order, with deploys broken in between. tailmux does the reaching-out part for
proxy-aware programs on a desktop; sandhi also answers DNS so ordinary programs work,
publishes services inward, and is configured by neusis on every machine. The first sketch
was neusis's tsmux spec (`docs/tailmux-proxy-spec.md` there); the embedding comes from
saarthi, which runs tsnet in its hub, node and phone.

## Progress
The project's phases, one line each, ticked when the phase is done; each phase's page holds
its goal and its own checklist. `yajna new phase` adds one; `yajna tasks` lists the open
items with the rest.
- [x] [[PHS-20261121-07gk9n]] Phase 0: the tailnet crate and the design
- [ ] [[PHS-20261121-07gk4r]] Phase 1: connect and inbound, the end of tailnet switching
- [ ] [[PHS-20261121-07gkgk]] Phase 2: DNS and a loopback address per peer
- [ ] [[PHS-20261121-07gkvm]] Phase 3: a TUN for every port and UDP

## Open questions
- How does neusis hand out auth keys for machines × tailnets without the hand-run rotation
  of November 2026? A key per tailnet shared by the fleet's machines, tagged, or keys
  minted by an OAuth client at activation.
- Does the peer list need a change notification from libtailscale, or is polling
  `status_json` enough for DNS (phase 2)?
- Which Rust userspace stack for the TUN (phase 3)?

## Design
The architecture is `docs/architecture.md` and its decision records are
`docs/decisions/`; here, only the few choices
a reader must know before touching the code.
- Three layers: backends (peers, dial, listen; the machine's identity on each is its
  dutavas), the router (name or address to one
  backend's peer, no fallback), front ends (`connect`, inbound forwards, DNS with loopback
  addresses, later a TUN) (ADR-0001).
- Rust; `crates/tailnet` is saarthi's crate copied at `954bc23`, to be shared once
  sandhi is stable (ADR-0002).
- DNS answers for each tailnet's real domain, never the system tailnet's, so TLS works
  (ADR-0003).

## Lessons
What was tried and dropped, and why, so nobody tries it again.
- Several tailscaled daemons on one machine: Linux only, breaks exit nodes and MSS
  clamping (neusis ADR-0008).
- A zone of our own for peer names: TLS fails, certificates name the MagicDNS name
  (ADR-0003).
- From saarthi ([[saarthi/EXP-20261112-rq8kgc]]): `up` from saved state returns before the
  netmap names the peers, so the first dial by name can fail; retry it, do not treat it as
  an outage.

## Experiments
One line per experiment after its link, what it asks and what came of it; `yajna validate`
rejects a bare link here. Newest last.
- [[EXP-20261127-hsprp3]] Does sandhi carry ssh between the fleet's machines over leoank while their system tailnet is runxi?
- [[EXP-20261127-j1rwdg]] a dutavas resumes from its tailscaled.state alone, without a key, if not ephemeral: the basis for saving curiosity's on SPARKDATA
