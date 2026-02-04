{
  description = "Rust GPU Shaders";

  inputs = {
    fenix.url = "github:nix-community/fenix/19c910fe3de1768e64e76b5ee6daa346e6b07410";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    fenix,
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {
        pkgs,
        system,
        ...
      }: let
        rustPkg = with fenix.packages.${system};
          combine [
            targets.wasm32-unknown-unknown.latest.rust-std
            (latest.withComponents
              [
                "rust-src"
                "rustc-dev"
                "llvm-tools-preview"
                "cargo"
                "clippy"
                "rustc"
                "rustfmt"
                "rust-analyzer"
              ])
          ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustPkg;
          rustc = rustPkg;
        };
        buildInputs = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          vulkan-loader
          vulkan-tools
          wayland
          libxkbcommon
          libgcc.lib
        ];
        shadersCompilePath = "$HOME/.cache/rust-gpu-shaders";
        template = rustPlatform.buildRustPackage {
          pname = "template";
          version = "0.0.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoLock.outputHashes = {
            "rustc_codegen_spirv-0.9.0" = "sha256-KUznKMWnOwYHF7v5qnziLnuJHQPDOfR98uBVZnvZykw=";
          };
          buildNoDefaultFeatures = true;
          buildFeatures = ["use-compiled-tools"];
          dontCargoSetupPostUnpack = true;
          postUnpack = ''
            mkdir -p .cargo
            cat "$cargoDeps"/.cargo/config.toml | sed "s|cargo-vendor-dir|$cargoDeps|" >> .cargo/config.toml
            # HACK(eddyb) bypass cargoSetupPostPatchHook.
            export cargoDepsCopy="$cargoDeps"
          '';
          nativeBuildInputs = [pkgs.makeWrapper];
          configurePhase = ''
            export SHADERS_DIR="$out/repo/shaders"
            export SHADERS_TARGET_DIR=${shadersCompilePath}
          '';
          fixupPhase = ''
            cp -r . $out/repo
            wrapProgram $out/bin/runner \
              --set LD_LIBRARY_PATH $LD_LIBRARY_PATH:$out/lib:${nixpkgs.lib.makeLibraryPath buildInputs} \
              --set PATH $PATH:${nixpkgs.lib.makeBinPath [rustPkg]}
          '';
        };
      in rec {
        packages.default = pkgs.writeShellScriptBin "template" ''
          export CARGO_TARGET_DIR="${shadersCompilePath}"
          exec -a "$0" "${template}/bin/runner" "$@"
        '';
        apps.default = {
          type = "app";
          program = "${packages.default}/bin/template";
        };
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = [rustPkg wasm-pack nodejs git-lfs];
            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          };
      };
    };
}
