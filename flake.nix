{
  description = "Rust GPU Shaders";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix/3116ee073ab3931c78328ca126224833c95e6227";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [fenix.overlays.default];
      pkgs = import nixpkgs {inherit overlays system;};

      rustPkg = fenix.packages.${system}.latest.withComponents [
        "rust-src"
        "rustc-dev"
        "llvm-tools-preview"
        "cargo"
        "clippy"
        "rustc"
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
      ];
      shadersCompilePath = "$HOME/.cache/rust-gpu-shaders";
    in rec {
      rustGpuShaders = rustPlatform.buildRustPackage {
        pname = "rust-gpu-shaders";
        version = "0.0.0";
        src = ./.;
        cargoHash = "";
        cargoLock.lockFile = ./Cargo.lock;
        cargoLock.outputHashes = {
          "rustc_codegen_spirv-0.9.0" = "sha256-uZn1p2pM5UYQKlY9u16aafPH7dfQcSG7PaFDd1sT4Qc=";
        };
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
      packages.default = pkgs.writeShellScriptBin "rust-gpu-shaders" ''
        export CARGO_TARGET_DIR="${shadersCompilePath}"
        exec -a "$0" "${rustGpuShaders}/bin/runner" "$@"
      '';
      apps.default = {
        type = "app";
        program = "${packages.default}/bin/rust-gpu-shaders";
      };
      devShell = with pkgs;
        mkShell {
          nativeBuildInputs = [rustPkg];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
    });
}
