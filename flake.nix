{
  description = "Personal cover letter template and management CLI for Typst";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      imports = [
        inputs.treefmt-nix.flakeModule
      ];
      perSystem =
        { config
        , pkgs
        , ...
        }:
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

          runtimeDeps = [
            pkgs.typst
          ];
        in
        {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.makeWrapper ];

            postInstall = ''
              wrapProgram $out/bin/${cargoToml.package.name} \
                --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps}
            '';
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [ config.treefmt.build.devShell ];

            shellHook = ''
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}

              # Auto-symlink the repo as a local Typst package
              DATA_DIR="''${XDG_DATA_HOME:-$HOME/.local/share}"
              TARGET="$DATA_DIR/typst/packages/local/cover-letter/0.1.0"
              if [ ! -L "$TARGET" ] || [ "$(readlink "$TARGET")" != "$(pwd)" ]; then
                mkdir -p "$(dirname "$TARGET")"
                rm -rf "$TARGET"
                ln -s "$(pwd)" "$TARGET"
                echo "cover-letter package linked -> $TARGET"
              fi
            '';

            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              cargo-watch
              cargo-edit
              rust-analyzer
            ];

            buildInputs = runtimeDeps ++ [ pkgs.helix ];
          };

          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };
        };
    };
}
