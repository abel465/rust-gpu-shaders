{
  description = "Rust GPU Shaders";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix/3116ee073ab3931c78328ca126224833c95e6227";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit overlays system; };

        rustPkg = fenix.packages.${system}.latest.withComponents [
          "rust-src" 
          "rustc-dev" 
          "llvm-tools-preview"
          "cargo"
          "rustc"
        ];
      in
      {
        devShell = with pkgs; mkShell rec {

          nativeBuildInputs = [ pkg-config ];
          
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";

          buildInputs = [
            rustPkg
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
            vulkan-tools
            wayland
            libxkbcommon
          ];
        };
      });
}
