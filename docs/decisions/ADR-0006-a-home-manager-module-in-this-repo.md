# ADR-0006: A home-manager module in this repo runs `sandhi up` as a user service, its config rendered from options and installed where clients look

Status: accepted
Date: 2026-11-21

## Context
Every neusis machine runs sandhi (ADR-0001), and on each the daemon must be up before ssh
calls `sandhi connect`. The module could live in neusis, next to the machines that use
it, or here, next to the config format it renders; saarthi keeps its modules in its own
repo (saarthi ADR-0041) so that a change to its flags and the module change together.
The daemon is per user (it holds the user's dutavas and a user-only socket, ADR-0005), so
it is a home-manager module, not a NixOS one. The clients (`sandhi connect`, run by ssh,
and `sandhi status`) find the daemon's socket by reading the same config file.

## Decision
- `homeModules.sandhi` lives in this repo (`nix/module.nix`), with options under
  `services.sandhi` mirroring the config: `backends.<name>.{domains, forwards, tailnet}`,
  `tailnet.{hostname, authKeyFile, ephemeral}`, `stateDir`, `socket`. A backend without
  a kind fails an assertion at evaluation.
- The TOML is rendered with `pkgs.formats.toml` and given to the daemon by its store path
  (`sandhi --config <store path> up`), so a changed config is a changed service and the
  switch restarts it. The same file is installed at `$XDG_CONFIG_HOME/sandhi/config.toml`,
  the clients' default.
- A launchd user agent on Darwin (`KeepAlive`), a systemd user service on Linux
  (`Restart=always`), and `sandhi` on PATH.
- `checks.module` evaluates the module for both platforms against stubs of the
  home-manager options it sets (no home-manager input), then runs the real `sandhi` on
  the rendered config, so a config the daemon would refuse fails the check.
- The ssh `ProxyCommand` stays out of the module: which domains go through sandhi is the
  machine's choice, made where its ssh config is (neusis).

## Consequences
- neusis takes sandhi as a flake input and imports the module; sandhi's flags and their
  module change in one commit here.
- The service restarts only when the daemon exits: a backend that failed to join stays
  failed while the daemon runs (ADR-0005), until a restart by hand or a config change.
- On Darwin an `authKeyFile` must be a fixed path: launchd does not expand the per-user
  temporary directory agenix uses by default.
