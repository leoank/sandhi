# ADR-0001: sandhi is a DNS server and a proxy, built as backends, a router and front ends, in both directions

Status: accepted
Date: 2026-11-21

## Context
The neusis fleet (rogue, ghost001, curiosity) runs one system tailscaled per machine, so a
machine is on one tailnet at a time (neusis ADR-0008). Moving the fleet from `shntnu.github`
to `runxi-shen.github` in November 2026 meant switching every machine by hand, in order,
with the deploy path (comin, saarthi's hub) breaking in between. Friends' tailnets (runxi,
shntnu) and the owner's own (leoank) all need to be reachable at once, and later other kinds
of network than tailnets.

The owner proposed a DNS server filled with the hostnames of every tailnet, answering with
local addresses a proxy listens on, the proxy forwarding each connection to the right
network. Considered against it:
- **Tailscale device sharing**: no code, but a shared device can be reached, not reach out,
  it must still be on some system tailnet, and each share is set up by hand.
- **One permanent fleet tailnet plus sharing**: no more migrations, but no full membership
  of the friends' tailnets.
- **Several tailscaled daemons**: rejected in neusis ADR-0008 (Linux only, breaks exit
  nodes and MSS clamping).
- **Proxy-aware programs only** (the tsmux spec in neusis `docs/tailmux-proxy-spec.md`:
  SOCKS5, HTTP CONNECT, a PAC file): no root, no DNS and no address allocation, but
  programs that ignore proxy settings are left out.
- **A gateway**: one machine on every tailnet advertising a fake address range as a subnet
  route to the fleet's tailnet, so devices with nothing installed (phones) reach them all;
  one more hop and one machine everything depends on.

## Decision
- sandhi keeps the owner's shape, DNS plus a proxy, split in three layers so the
  alternatives that remain useful are modes of one program rather than rivals:
  - **backends** list their peers, dial a peer and listen for callers. The machine's
    identity on one network is its **dutavas** (दूतावास, the envoy's residence: an
    embassy), which represents the machine there, dialling out for it and receiving
    callers on its behalf (the owner, 2026-11-21). A tailnet, whose dutavas is an
    embedded tsnet identity, is the first backend; the interface must not assume a
    tailnet (no MagicDNS, no 100.x addresses, no Tailscale identities in it).
  - **the router** maps a name or a local address to one (backend, peer). A name owned
    by one backend never falls back to another.
  - **front ends** take connections in: `sandhi connect <host> <port>` on stdio for ssh's
    `ProxyCommand`; inbound forwarding (a backend listens on chosen ports and forwards to
    local services); DNS with a local address per peer (ADR-0003, ADR-0004); a TUN later.
- Both directions (the owner, 2026-11-21): a machine reaches the peers of every backend,
  and is published into backends whose system client it does not run, so ghost001 can be on
  leoank and runxi at once.
- Gateway mode is out (the owner, 2026-11-21); phones are left to saarthi, which embeds
  its own identity per hub (saarthi ADR-0049).
- Build order: `connect` and inbound forwarding first, which end tailnet switching without
  root or DNS; then DNS with loopback addresses; then the TUN.

## Consequences
- Every machine joins every backend as a device of its own: machines × tailnets devices
  and auth keys. Key handling in neusis has to be automated, not hand-run per machine.
- sandhi down means every name in another backend stops resolving and every published
  service is unreachable there; it must start from saved state and retry the first dials
  (saarthi found the netmap arrives after `up` returns).
- An inbound connection reaches the local service from `127.0.0.1`: sshd and the like lose
  who called. sandhi logs the caller from the backend and may pass it on (PROXY protocol)
  to services that take it. The backend's own access rules decide who reaches the device.
- A loopback front end is open to every user on the machine; on shared machines sandhi
  runs per user or, with the TUN, checks the owner of a connection.
