#!/usr/bin/env bash
# EXP-20261127-jwk3y3: does `sandhi up` exit on SIGTERM after a join failed? Starts a
# dutavas on leoank from an auth key that is not valid, waits for the backend to settle,
# sends SIGTERM and watches for 15 s; a daemon still alive is sampled (macOS `sample`) and
# killed. The binary is $SANDHI (default: `sandhi` on PATH). Writes run.log and, when
# sampled, sample.txt beside itself (or into $YAJNA_RUN_DIR). Joins no device: the key is
# refused. Temporary state goes to a directory removed at the end.
set -uo pipefail
out=${YAJNA_RUN_DIR:-$(cd "$(dirname "$0")" && pwd)}
exec > >(tee "$out/run.log") 2>&1
bin=${SANDHI:-sandhi}
tmp=$(mktemp -d)
pid=
cleanup() { if [ -n "$pid" ]; then kill -9 "$pid" 2>/dev/null; fi; rm -rf "$tmp"; }
trap cleanup EXIT

echo "tskey-auth-not-a-key" >"$tmp/key"
cat >"$tmp/c.toml" <<TOML
state_dir = "$tmp/state"
socket = "$tmp/s.sock"
[backends.leoank.tailnet]
hostname = "sandhi-test-sigterm"
authkey_file = "$tmp/key"
ephemeral = true
TOML

echo "=== $("$bin" --version) at $(date -u +%FT%TZ)"
"$bin" --config "$tmp/c.toml" up >"$tmp/daemon.log" 2>&1 &
pid=$!
for _ in $(seq 120); do
  s=$("$bin" --config "$tmp/c.toml" status 2>/dev/null || true)
  case "$s" in *ready*|*failed*) break;; esac
  sleep 0.5
done
echo "status: $s"

echo "=== SIGTERM"
kill -TERM "$pid"
t0=$SECONDS
while kill -0 "$pid" 2>/dev/null && [ $((SECONDS - t0)) -lt 15 ]; do sleep 0.2; done
if kill -0 "$pid" 2>/dev/null; then
  echo "still alive $((SECONDS - t0)) s after SIGTERM; sampling"
  sample "$pid" 2 -file "$out/sample.txt" >/dev/null 2>&1
  echo "sample written"
else
  wait "$pid"; echo "exited $((SECONDS - t0)) s after SIGTERM, status $?"
  pid=
fi
echo "=== daemon log (last lines)"
grep -av '^\s*$' "$tmp/daemon.log" | grep -av 'AuthLoop\|LocalBackend' | tail -5
