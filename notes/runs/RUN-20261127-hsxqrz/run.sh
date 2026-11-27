#!/usr/bin/env bash
# EXP-20261127-hsprp3: ssh from rogue to ghost001 and curiosity through sandhi over leoank,
# and to ghost001 over the system tailnet (runxi), comparing host keys. Run on rogue, with
# the neusis sandhi bundles switched in on all three machines. Writes run.log and
# known_hosts beside itself (or into $YAJNA_RUN_DIR). Reads nothing secret; logs in with
# whatever ssh agent ~/.ssh/config names.
set -uo pipefail
out=${YAJNA_RUN_DIR:-$(cd "$(dirname "$0")" && pwd)}
kh=$out/known_hosts
: >"$kh"
exec > >(tee "$out/run.log") 2>&1

remote='echo "hostname: $(hostname)"; echo "sandhi status:"; sandhi status; echo "system tailnet:"; tailscale switch --list'
ssh_to() { # <name> [ssh options…]
    local name=$1; shift
    local t0=$SECONDS
    echo "=== ssh $name"
    if ssh -o BatchMode=yes -o ConnectTimeout=60 -o UserKnownHostsFile="$kh" \
        -o StrictHostKeyChecking=accept-new "$@" "$name" "$remote"; then
        echo "ok: $name in $((SECONDS - t0)) s"
    else
        echo "FAILED: $name (exit $?) after $((SECONDS - t0)) s"
    fi
}
via_sandhi=(-o "ProxyCommand=sandhi connect %h %p")

echo "=== rogue at $(date -u +%FT%TZ)"
echo "sandhi status:"; sandhi status
echo "system tailnet:"; tailscale switch --list

ssh_to sandhi-ghost001.leoank "${via_sandhi[@]}"
ssh_to ghost001
ssh_to sandhi-curiosity.leoank "${via_sandhi[@]}"

echo "=== host keys (ed25519)"
ssh-keygen -lf "$kh" | grep -i ed25519
