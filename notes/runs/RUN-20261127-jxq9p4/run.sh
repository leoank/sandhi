#!/usr/bin/env bash
# EXP-20261127-jwk3y3: does `sandhi up` exit on SIGTERM after a join that tailscale_up keeps
# waiting on? An ephemeral dutavas joins leoank with rogue's sandhi key (read by sandhi,
# never printed) and is stopped, so the tailnet removes it; started again from its state
# with no key, tsnet waits for a login and sandhi reports the backend failed. Then SIGTERM,
# 15 s of watching, a `sample` of a daemon still alive, and SIGKILL. The binary is $SANDHI
# (default: `sandhi` on PATH). Writes run.log (and sample.txt) beside itself or into
# $YAJNA_RUN_DIR; the temporary state, with the node key, is removed at the end.
set -uo pipefail
out=${YAJNA_RUN_DIR:-$(cd "$(dirname "$0")" && pwd)}
exec > >(tee "$out/run.log") 2>&1
bin=${SANDHI:-sandhi}
tmp=$(mktemp -d)
pid=
cleanup() { if [ -n "$pid" ]; then kill -9 "$pid" 2>/dev/null; fi; rm -rf "$tmp"; }
trap cleanup EXIT

config() { # <key file>
  cat <<TOML
state_dir = "$tmp/state"
socket = "$tmp/s.sock"
[backends.leoank.tailnet]
hostname = "sandhi-test-sigterm"
authkey_file = "$1"
ephemeral = true
TOML
}
start() { "$bin" --config "$1" up >>"$tmp/daemon.log" 2>&1 & pid=$!; }
settle() { # <config>: the status once ready or failed
  local s=
  for _ in $(seq 120); do
    s=$("$bin" --config "$1" status 2>/dev/null || true)
    case "$s" in *ready*|*failed*) break;; esac
    sleep 0.5
  done
  echo "$s"
}
term() { # SIGTERM; 15 s; sample and SIGKILL if still alive
  kill -TERM "$pid"
  local t0=$SECONDS
  while kill -0 "$pid" 2>/dev/null && [ $((SECONDS - t0)) -lt 15 ]; do sleep 0.2; done
  if kill -0 "$pid" 2>/dev/null; then
    echo "still alive $((SECONDS - t0)) s after SIGTERM; sampling"
    sample "$pid" 2 -file "$out/sample.txt" >/dev/null 2>&1 && echo "sample written"
    kill -9 "$pid"; wait "$pid" 2>/dev/null
  else
    wait "$pid"; echo "exited $((SECONDS - t0)) s after SIGTERM, status $?"
  fi
  pid=
}

echo "=== $("$bin" --version) at $(date -u +%FT%TZ)"
echo "=== first start: ephemeral, with the key"
config "$HOME/.config/sandhi/leoank-authkey" >"$tmp/a.toml"
start "$tmp/a.toml"; echo "status: $(settle "$tmp/a.toml")"; term

echo "=== second start: its state, no key"
config "$tmp/no-such-key" >"$tmp/b.toml"
start "$tmp/b.toml"; echo "status: $(settle "$tmp/b.toml")"; term
