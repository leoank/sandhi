# Tests of the home-manager module, run as `checks.module`. The module is evaluated
# against stubs of the home-manager options it sets, so the flake needs no home-manager
# input; both platforms are evaluated wherever the check runs. A failing evaluation test
# fails evaluation and names itself. The rendered config is then read by the real
# `sandhi`, so a file the daemon would refuse fails the check.
{
  pkgs,
  lib,
  nixpkgs,
  self,
  sandhi,
}:
let
  stubHm =
    { lib, ... }:
    let
      anything = lib.mkOption {
        type = lib.types.attrsOf lib.types.anything;
        default = { };
      };
    in
    {
      options = {
        assertions = lib.mkOption {
          type = lib.types.listOf lib.types.anything;
          default = [ ];
        };
        home.packages = lib.mkOption {
          type = lib.types.listOf lib.types.package;
          default = [ ];
        };
        xdg.configFile = anything;
        launchd.agents = anything;
        systemd.user.services = anything;
      };
    };

  eval =
    system: extra:
    let
      p = import nixpkgs { inherit system; };
      pkg = p.writeShellScriptBin "sandhi" "exit 0";
    in
    {
      bin = lib.getExe pkg;
      inherit pkg;
      config =
        (lib.evalModules {
          modules = [
            stubHm
            self.homeModules.sandhi
            {
              _module.args.pkgs = p;
              services.sandhi = {
                enable = true;
                package = pkg;
              }
              // extra;
            }
          ];
        }).config;
    };

  full = {
    socket = "/nonexistent/sandhi.sock";
    backends = {
      leoank = {
        domains = [ "tail1234.ts.net" ];
        forwards = [
          {
            port = 22;
            to = "127.0.0.1:22";
          }
        ];
        tailnet = {
          hostname = "rogue";
          authKeyFile = "/run/agenix/leoankTsAuthKey";
        };
      };
      runxi.tailnet.ephemeral = true;
    };
  };

  argv =
    c:
    if c.launchd.agents ? sandhi then
      c.launchd.agents.sandhi.config.ProgramArguments
    else
      lib.splitString " " c.systemd.user.services.sandhi.Service.ExecStart;
  # The rendered config, as the daemon is given it.
  rendered = c: builtins.elemAt (argv c) 2;

  tests = lib.concatMapAttrs (
    tag: system:
    let
      base = eval system { };
      rich = eval system full;
      isDarwin = lib.hasSuffix "darwin" system;
      a = argv rich.config;
    in
    {
      "test-${tag}-service" = {
        expr =
          if isDarwin then
            let
              s = base.config.launchd.agents.sandhi;
            in
            {
              on = s.enable && s.config.RunAtLoad;
              keep = s.config.KeepAlive;
              linux = base.config.systemd.user.services ? sandhi;
            }
          else
            let
              s = base.config.systemd.user.services.sandhi;
            in
            {
              on = s.Install.WantedBy == [ "default.target" ];
              keep = s.Service.Restart == "always";
              darwin = base.config.launchd.agents ? sandhi;
            };
        expected = {
          on = true;
          keep = true;
        }
        // (if isDarwin then { linux = false; } else { darwin = false; });
      };
      "test-${tag}-command" = {
        expr = {
          argv = [ (builtins.elemAt a 0) (builtins.elemAt a 1) (builtins.elemAt a 3) ];
          sameFile = rendered rich.config == "${rich.config.xdg.configFile."sandhi/config.toml".source}";
          onPath = rich.config.home.packages == [ rich.pkg ];
          asserted = builtins.all (x: x.assertion) rich.config.assertions;
        };
        expected = {
          argv = [ rich.bin "--config" "up" ];
          sameFile = true;
          onPath = true;
          asserted = true;
        };
      };
    }
  ) {
    darwin = "aarch64-darwin";
    linux = "x86_64-linux";
  };

  noKind = (eval "x86_64-linux" { backends.a.domains = [ ]; }).config.assertions;
  assertionTests.test-a-backend-needs-a-kind = {
    expr = map (x: x.assertion) noKind;
    expected = [ false ];
  };

  failures = lib.runTests (tests // assertionTests);
  host = eval pkgs.stdenv.hostPlatform.system full;
in
if failures != [ ] then
  throw "module tests failed: ${builtins.toJSON failures}"
else
  # `status` loads the config and then reaches for its socket: an error about the socket
  # means sandhi read every field, the socket among them.
  pkgs.runCommand "sandhi-module" { } ''
    cat ${rendered host.config}
    if msg=$(${lib.getExe sandhi} --config ${rendered host.config} status 2>&1); then
      echo "status succeeded without a daemon: $msg"; exit 1
    fi
    echo "$msg"
    case "$msg" in
      *"reaching sandhi at /nonexistent/sandhi.sock"*) touch $out ;;
      *) exit 1 ;;
    esac
  ''
