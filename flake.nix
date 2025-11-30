{
  description = "Wichtel Loser - Secret Santa Randomizer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default;
        
        wichtelLoser = pkgs.rustPlatform.buildRustPackage {
          pname = "wichtel_loser";
          version = "0.1.0";
          
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
          
          postInstall = ''
            mkdir -p $out/share/wichtel_loser
            cp -r templates $out/share/wichtel_loser/
            cp -r static $out/share/wichtel_loser/
          '';
          
          meta = with pkgs.lib; {
            description = "A festive Secret Santa randomizer web application";
            homepage = "https://github.com/Vcele/wichtel_loser";
            license = licenses.mit;
          };
        };
      in
      {
        packages = {
          default = wichtelLoser;
          wichtel_loser = wichtelLoser;
        };
        
        apps.default = {
          type = "app";
          program = "${wichtelLoser}/bin/wichtel_loser";
        };
        
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            rust-analyzer
            pkg-config
            openssl
          ];
          
          shellHook = ''
            echo "🎄 Wichtel Loser Development Environment"
            echo "Run 'cargo run' to start the server"
          '';
        };
        
        nixosModules.default = { config, lib, pkgs, ... }:
          with lib;
          let
            cfg = config.services.wichtelloser;
          in
          {
            options.services.wichtelloser = {
              enable = mkEnableOption "Wichtel Loser Secret Santa service";
              
              port = mkOption {
                type = types.port;
                default = 8080;
                description = "Port to listen on";
              };
              
              address = mkOption {
                type = types.str;
                default = "127.0.0.1";
                description = "Address to bind to";
              };
            };
            
            config = mkIf cfg.enable {
              systemd.services.wichtelloser = {
                description = "Wichtel Loser Secret Santa Service";
                wantedBy = [ "multi-user.target" ];
                after = [ "network.target" ];
                
                environment = {
                  BIND_ADDRESS = "${cfg.address}:${toString cfg.port}";
                };
                
                serviceConfig = {
                  ExecStart = "${wichtelLoser}/bin/wichtel_loser";
                  WorkingDirectory = "${wichtelLoser}/share/wichtel_loser";
                  Restart = "always";
                  RestartSec = 5;
                  DynamicUser = true;
                };
              };
            };
          };
      }
    );
}
