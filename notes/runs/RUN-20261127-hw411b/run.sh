#!/usr/bin/env bash
# EXP-20261127-hsprp3: why rogue gets no ssh banner from sandhi-curiosity.leoank while
# curiosity's dutavas is up. Asks curiosity (over runxi) which leoank name its dutavas
# took and whether its sshd answers on 127.0.0.1:22, then dials curiosity's leoank
# address through sandhi from rogue, bypassing the name. Run on rogue; writes run.log
# beside itself (or into $YAJNA_RUN_DIR).
set -uo pipefail
out=${YAJNA_RUN_DIR:-$(cd "$(dirname "$0")" && pwd)}
exec > >(tee "$out/run.log") 2>&1
opts=(-o BatchMode=yes -o ConnectTimeout=30)

echo "=== curiosity over runxi at $(date -u +%FT%TZ)"
ssh "${opts[@]}" curiosity '
  echo "sandhi status: $(sandhi status)"
  echo "names in tsnet.log:"; grep -o "sandhi-curiosity[-0-9]*" ~/.local/state/sandhi/leoank/tsnet.log | sort | uniq -c
  echo "uptime: $(uptime)"
  if (exec 3<>/dev/tcp/127.0.0.1/22) 2>/dev/null; then echo "sshd on 127.0.0.1:22: open"; else echo "sshd on 127.0.0.1:22: closed"; fi
  true'
ip=$(ssh "${opts[@]}" curiosity 'sandhi status' | awk '/^leoank/ {split($4, a, ","); print a[1]}')
echo "curiosity's leoank address: $ip"

echo "=== rogue -> $ip.leoank through sandhi"
t0=$SECONDS
if ssh "${opts[@]}" -o UserKnownHostsFile=/dev/null -o GlobalKnownHostsFile=/dev/null \
    -o StrictHostKeyChecking=no -o "ProxyCommand=sandhi connect %h %p" "$ip.leoank" hostname; then
  echo "ok in $((SECONDS - t0)) s"
else
  echo "FAILED (exit $?) after $((SECONDS - t0)) s"
fi
