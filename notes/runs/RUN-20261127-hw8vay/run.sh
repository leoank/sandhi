#!/usr/bin/env bash
# EXP-20261127-hsprp3: which leoank name curiosity's dutavas holds, from its own tsnet log
# (over runxi), and which sandhi-curiosity names rogue's dutavas has seen in its netmaps.
# Run on rogue; writes run.log beside itself (or into $YAJNA_RUN_DIR).
set -uo pipefail
out=${YAJNA_RUN_DIR:-$(cd "$(dirname "$0")" && pwd)}
exec > >(tee "$out/run.log") 2>&1
names='sandhi-curiosity[-0-9]*(\.[a-z0-9-]+\.ts\.net)?'

echo "=== curiosity's own tsnet log, at $(date -u +%FT%TZ)"
ssh -o BatchMode=yes -o ConnectTimeout=30 curiosity \
  "grep -aEo '$names' ~/.local/state/sandhi/leoank/tsnet.log | sort | uniq -c; ls -la --time-style=full-iso ~/.local/state/sandhi/leoank/"
echo "=== rogue's tsnet log"
grep -aEo "$names" ~/.local/state/sandhi/leoank/tsnet.log | sort | uniq -c
