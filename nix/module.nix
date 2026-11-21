# The sandhi daemon as a home-manager module (ADR-0006).
#
# Runs `sandhi up` for the logged-in user: a launchd user agent on Darwin, a systemd user
# service on Linux, restarted whenever it exits. The config is rendered from the options
# below, given to the daemon by its store path (a changed config is a changed service, so
# the switch restarts it), and also installed at `$XDG_CONFIG_HOME/sandhi/config.toml`,
# where `sandhi connect` and `sandhi status` read it to find the daemon's socket.
#
# `self` is this flake, for the default package.
self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.sandhi;
  toml = pkgs.formats.toml { };

  backend =
    b:
    {
      inherit (b) domains forwards;
    }
    // lib.optionalAttrs (b.tailnet != null) {
      tailnet = lib.filterAttrs (_: v: v != null) {
        inherit (b.tailnet) hostname ephemeral;
        authkey_file = b.tailnet.authKeyFile;
      };
    };
  configFile = toml.generate "sandhi.toml" (
    lib.filterAttrs (_: v: v != null) {
      state_dir = cfg.stateDir;
      inherit (cfg) socket;
      backends = lib.mapAttrs (_: backend) cfg.backends;
    }
  );
  args = [
    (lib.getExe cfg.package)
    "--config"
    "${configFile}"
    "up"
  ];
in
{
  options.services.sandhi = {
    enable = lib.mkEnableOption "the sandhi daemon, this machine on several networks at once";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.sandhi;
      defaultText = lib.literalExpression "sandhi.packages.\${system}.sandhi";
      description = ''
        The package providing the `sandhi` binary. It is also put on the user's PATH, for
        `sandhi connect` (ssh's `ProxyCommand`) and `sandhi status`.
      '';
    };

    backends = lib.mkOption {
      default = { };
      description = ''
        The networks this machine is a member of, one dutavas each. `<peer>.<name>` reaches
        a peer of backend `<name>`, `<peer>.<domain>` one of each of its `domains`; a name
        is owned by one backend only (ADR-0005).
      '';
      example = lib.literalExpression ''
        {
          leoank = {
            domains = [ "tail1234.ts.net" ];
            forwards = [ { port = 22; to = "127.0.0.1:22"; } ];
            tailnet.authKeyFile = "/run/agenix/leoankTsAuthKey";
          };
        }
      '';
      type = lib.types.attrsOf (
        lib.types.submodule {
          options = {
            domains = lib.mkOption {
              type = lib.types.listOf lib.types.str;
              default = [ ];
              description = "DNS domains this backend owns besides its own name, such as its MagicDNS domain.";
            };
            forwards = lib.mkOption {
              type = lib.types.listOf (
                lib.types.submodule {
                  options = {
                    port = lib.mkOption {
                      type = lib.types.port;
                      description = "The port callers on the backend reach this machine on.";
                    };
                    to = lib.mkOption {
                      type = lib.types.str;
                      example = "127.0.0.1:22";
                      description = "The local service each caller is connected to, `host:port`.";
                    };
                  };
                }
              );
              default = [ ];
              description = "Local services published into this backend.";
            };
            tailnet = lib.mkOption {
              description = "The backend is a tailnet, with an embedded tsnet dutavas.";
              type = lib.types.nullOr (
                lib.types.submodule {
                  options = {
                    hostname = lib.mkOption {
                      type = lib.types.nullOr lib.types.str;
                      default = null;
                      description = "The machine's name on the tailnet. Null: this machine's hostname.";
                    };
                    authKeyFile = lib.mkOption {
                      type = lib.types.nullOr lib.types.str;
                      default = null;
                      example = "/run/agenix/leoankTsAuthKey";
                      description = ''
                        File holding an auth key, for the first join and after the tailnet
                        logs the dutavas out. On Darwin give a fixed path: launchd does not
                        expand the per-user temporary directory agenix defaults to.
                      '';
                    };
                    ephemeral = lib.mkOption {
                      type = lib.types.bool;
                      default = false;
                      description = "The tailnet removes the device soon after it goes offline: for tests.";
                    };
                  };
                }
              );
              default = null;
            };
          };
        }
      );
    };

    stateDir = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Where each dutavas keeps its state. Null: `$XDG_STATE_HOME/sandhi`.";
    };

    socket = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = ''
        The control socket. Null: `$XDG_RUNTIME_DIR/sandhi.sock`, else in the state
        directory. macOS caps socket paths at 104 bytes.
      '';
    };

    logDir = lib.mkOption {
      type = lib.types.str;
      default = "/tmp";
      description = ''
        Directory of the Darwin logs, `<logDir>/sandhi.{out,err}.log`. On Linux the output
        goes to the user journal: `journalctl --user -u sandhi`.
      '';
    };
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        assertions = lib.mapAttrsToList (name: b: {
          assertion = b.tailnet != null;
          message = "services.sandhi.backends.${name}: set the kind of network (tailnet)";
        }) cfg.backends;
        home.packages = [ cfg.package ];
        xdg.configFile."sandhi/config.toml".source = configFile;
      }

      (lib.mkIf pkgs.stdenv.hostPlatform.isDarwin {
        launchd.agents.sandhi = {
          enable = true;
          config = {
            ProgramArguments = args;
            KeepAlive = true;
            RunAtLoad = true;
            StandardErrorPath = "${cfg.logDir}/sandhi.err.log";
            StandardOutPath = "${cfg.logDir}/sandhi.out.log";
          };
        };
      })

      (lib.mkIf pkgs.stdenv.hostPlatform.isLinux {
        systemd.user.services.sandhi = {
          Unit.Description = "sandhi, this machine on several networks at once";
          Service = {
            ExecStart = lib.escapeShellArgs args;
            Restart = "always";
            RestartSec = 5;
          };
          Install.WantedBy = [ "default.target" ];
        };
      })
    ]
  );
}
