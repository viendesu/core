{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { flake-utils, fenix, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem(system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = with pkgs.fenix; combine [
          ((fromToolchainName { name = "1.87"; sha256 = "sha256-KUm16pHj+cRedf8vxs/Hd2YWxpOrWZ7UOrwhILdSJBU="; }).withComponents [
            "cargo"
            "rustc"
            "rust-src"
            "rust-analyzer"
            "clippy"
            "llvm-tools-preview"
          ])
          default.rustfmt
        ];
        llvm = pkgs.llvmPackages_20;
      in
        {
          devShells.default = pkgs.mkShell {
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            packages = [ rust ] ++ (with pkgs; [
              libclang.lib
            ]);
            buildInputs = with pkgs; [
              stdenv.cc.cc.lib
            ];
          };
        }
    );
}
