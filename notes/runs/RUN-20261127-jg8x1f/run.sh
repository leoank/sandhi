#!/usr/bin/env bash
# EXP-20261127-j1rwdg: does a dutavas come back from a copy of its tailscaled.state alone?
# Run on rogue with the installed `sandhi` and rogue's sandhi key (~/.config/sandhi/
# leoank-authkey, read by sandhi, never printed). The state directories hold node keys, so
# they live in a temporary directory outside the repo and are deleted at the end. Writes
# run.log beside itself (or into $YAJNA_RUN_DIR). The identity is not ephemeral, since the
# tailnet may delete an ephemeral device on disconnect (RUN-20261127-j811rw): it leaves a
# device `sandhi-test-restore` on leoank, to be deleted in the admin console.
set -euo pipefail
out=${YAJNA_RUN_DIR:-$(cd "$(dirname "$0")" && pwd)}
exec > >(tee "$out/run.log") 2>&1
tmp=$(mktemp -d)
pid=
trap '[ -n "$pid" ] && kill $pid 2>/dev/null; rm -rf "$tmp"' EXIT
key=$HOME/.config/sandhi/leoank-authkey

config() { # <state dir> <key file>
  cat <<TOML
state_dir = "$1"
socket = "$1/sandhi.sock"
[backends.leoank.tailnet]
hostname = "sandhi-test-restore"
authkey_file = "$2"
ephemeral = false
TOML
}
# The daemon starts in this shell (never inside `$(…)`, which would wait on its stdout),
# and only the polling runs in a substitution.
up() { sandhi --config "$1" up >>"$tmp/daemon.log" 2>&1 & pid=$!; }
ready() { # <config>: the status line once ready or failed
  for _ in $(seq 240); do
    s=$(sandhi --config "$1" status 2>/dev/null || true)
    case "$s" in *ready*|*failed*) echo "$s"; return;; esac
    sleep 0.5
  done
  echo "not ready in 120 s: $s"
}
# SIGTERM, then SIGKILL after 10 s: a daemon whose join is stuck may not exit (j811rw).
down() {
  kill "$pid" 2>/dev/null || true
  for _ in $(seq 20); do kill -0 "$pid" 2>/dev/null || break; sleep 0.5; done
  if kill -0 "$pid" 2>/dev/null; then echo "(daemon ignored SIGTERM for 10 s; killed)"; kill -9 "$pid"; fi
  wait "$pid" 2>/dev/null || true
}

echo "=== first start, joining with the key, at $(date -u +%FT%TZ)"
mkdir -p "$tmp/a" "$tmp/b"
config "$tmp/a" "$key" >"$tmp/a.toml"
up "$tmp/a.toml"; first=$(ready "$tmp/a.toml"); echo "$first"
down
echo "files the first start left: $(cd "$tmp/a/leoank" && ls | tr '\n' ' ')"

echo "=== second start from tailscaled.state alone, no key"
mkdir -p "$tmp/b/leoank"
cp "$tmp/a/leoank/tailscaled.state" "$tmp/b/leoank/"
config "$tmp/b" "$tmp/no-such-key" >"$tmp/b.toml"
up "$tmp/b.toml"; second=$(ready "$tmp/b.toml"); echo "$second"
down

addr() { awk '{print $4}' <<<"$1"; }
echo "=== result"
if [ -n "$(addr "$first")" ] && [ "$(addr "$first")" = "$(addr "$second")" ]; then
  echo "same address: $(addr "$first")"
else
  echo "differ: first '$(addr "$first")', second '$(addr "$second")'"
fi
