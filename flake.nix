{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, crane, pre-commit, ... }@inputs:
    let
      systems =
        [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

      fenixToolchain = fenix:
        fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ];

      # Memoize nixpkgs for different platforms
      nixpkgsFor = forAllSystems (system:
        import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default self.overlays.default ];
        });
    in {
      overlays.sysbadge = final: prev: { };
      overlays.default = self.overlays.sysbadge;

      legacyPackages = nixpkgsFor;

      packages = forAllSystems (system: { });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
          shell = { lib, stdenv, mkShell, fenix, rust-analyzer-nightly, gdb
            , cargo-watch, cargo-edit, cargo-outdated, cargo-asm, libiconv
            , flip-link, probe-run }:
            mkShell {
              nativeBuildInputs = [
                (fenixToolchain fenix)
                rust-analyzer-nightly
                cargo-watch
                cargo-edit
                cargo-outdated
                cargo-asm
                flip-link
                probe-run
              ] ++ lib.optional stdenv.isLinux gdb
                ++ lib.optional stdenv.isDarwin libiconv;
              inherit (self.checks.${system}.pre-commit) shellHook;
            };
        in {
          default = pkgs.callPackage shell {
            gdb = pkgs.gdb.override { pythonSupport = true; };
          };
        });

      formatter = forAllSystems (system: nixpkgsFor.${system}.nixfmt);

      checks = forAllSystems (system: {
        pre-commit = pre-commit.lib.${system}.run {
          src = ./.;
          tools.rustfmt = nixpkgsFor.${system}.fenix.complete.rustfmt;
          hooks = {
            rustfmt.enable = true;
            nixfmt.enable = true;
            actionlint.enable = true;
          };
        };
      });
    };
}
