---
id: AGS-20261127-n8p5w3
type: agent_session
schema_version: 1
title: Study rayfish as a future iroh backend and as prior art for sandhi; keep the learnings as a reference
model: "claude-opus-5-5"
harness: "claude-code"
started: 2026-11-27T19:21+00:00
finished: 2026-11-27T19:45+00:00
created_entities: [rayfish]
edited_entities: []
outcome: done             # done | failed | aborted | pending
created: 2026-11-27
author: agent:claude
---

## Task
The owner: study github.com/rayfish/rayfish properly, as a future iroh backend and for the
work in it that sandhi is building; say what can be reused and learnt, and keep the learnings
as references, the folder being empty so far.

## What happened
Cloned at `a371852` into a scratch directory and read by three readers in parallel (its
iroh core and mesh; its DNS, TUN and rootless stack; its daemon, IPC, status and tests),
their load-bearing claims checked against the source. The result is the reading note
[@rayfish], `notes/references/rayfish.md`, with its entry in `references/library.bib`
(a `@software` entry by hand; the owner may move it to Zotero).

The verdict: a reference to learn from and copy small pieces of, not a dependency. Its lib
crate is not a public API and builds against forks of iroh; nothing in it dials a peer's
port as a stream, so an iroh backend's `dial` and forwards are ours to write. Most of value:
the endpoint's discovery layers, hiding tailnet addresses from iroh, the ALPN and
connection-closing rules, the DNS answers and the Tailscale coexistence pitfalls for
phase 2, tun-rs and smoltcp behind one packet trait for phase 3, and `FatalExit`, a
per-backend token and a status model for the daemon.
