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

      sourcesFilter = craneLib: lib: name: type:
        let
          baseName = baseNameOf (toString name);
          memory = baseName == "memory.x";
          web = lib.any (suffix: lib.hasSuffix suffix baseName) [ ".html" ];
        in craneLib.filterCargoSources name type || memory || web;

      cleanSources = craneLib: src: lib:
        lib.cleanSourceWith {
          src = (lib.cleanSource src);
          filter = (sourcesFilter craneLib lib);
        };

    in {
      overlays.sysbadge_fw = final: prev:
        let
          rustTarget = target:
            final.rust.toRustTargetSpec (final.lib.systems.elaborate target);

          commonArgs = craneLib:
            { lib ? final.lib, stdenv ? final.pkgs.stdenv, SDL2 ? null
            , libiconv ? final.pkgs.libiconv, toolchain, fw ? true }:
            let

              src = cleanSources craneLib (craneLib.path ./.)
                lib; # craneLib.cleanCargoSource (craneLib.path ./.);
            in {
              pname = "sysbadge-fw";
              inherit src;

              doCheck = false;
              checkPhaseCargoCommand = "";

              cargoVendorDir = craneLib.vendorMultipleCargoDeps {
                inherit (craneLib.findCargoFiles src) cargoConfigs;
                cargoLockList = [
                  ./Cargo.lock

                  # Unfortunately this approach requires IFD (import-from-derivation)
                  # otherwise Nix will refuse to read the Cargo.lock from our toolchain
                  # (unless we build with `--impure`).
                  #
                  # Another way around this is to manually copy the rustlib `Cargo.lock`
                  # to the repo and import it with `./path/to/rustlib/Cargo.lock` which
                  # will avoid IFD entirely but will require manually keeping the file
                  # up to date!
                  #"${toolchain}/lib/rustlib/src/rust/Cargo.lock"
                  ./rust-lock.toml
                ];
              };

              cargoExtraArgs = if fw then
                "-Z build-std=compiler_builtins,core,alloc --target thumbv6m-none-eabi"
              else
                "-Z build-std=compiler_builtins,core,alloc,std --target ${
                  rustTarget stdenv.targetPlatform.system
                }";

              buildInputs = if fw then
                [ ]
              else
                [ SDL2 ] ++ lib.optional stdenv.isDarwin libiconv;
            };
          cargoArtifacts = lib: toolchain:
            lib.buildDepsOnly ((commonArgs lib {
              fw = false;
              inherit toolchain;
            }) // {
              doCheck = false;
            });
        in {
          sysbadge_simulator = final.callPackage
            ({ lib, stdenv, fenix, SDL2, libiconv }:
              let
                system = stdenv.targetPlatform.system;
                toolchain = (fenixToolchain fenix);
                craneLib = crane.lib.${system}.overrideToolchain toolchain;
              in craneLib.buildPackage ((commonArgs craneLib {
                inherit lib stdenv SDL2 libiconv toolchain;
                fw = false;
              }) // {
                cargoArtifacts = cargoArtifacts craneLib toolchain;
                pname = "sysbadge-fw-simulator";
                cargoExtraArgs =
                  "-Z build-std=compiler_builtins,core,alloc,std --target ${
                    rustTarget stdenv.targetPlatform.system
                  } --package sysbadge-simulator";
              })) { };
          sysbadge_fw_unwraped = final.callPackage
            ({ lib, stdenv, fenix, flip-link, elf2uf2-rs, libiconv }:
              let
                system = stdenv.targetPlatform.system;
                toolchain = (fenixToolchain fenix);
                craneLib = crane.lib.${system}.overrideToolchain toolchain;
              in craneLib.buildPackage ((commonArgs craneLib {
                inherit lib stdenv toolchain;
                fw = true;
              }) // {
                cargoArtifacts = cargoArtifacts craneLib toolchain;
                pname = "sysbadge-fw";
                nativeBuildIputs = [ flip-link ];
                buildInputs = [ flip-link ]
                  ++ lib.optional stdenv.isDarwin libiconv;
                cargoExtraArgs =
                  "-Z build-std=compiler_builtins,core,alloc --target thumbv6m-none-eabi --package sysbadge-fw";

                postInstallPhases = ''
                  mkdir -p $out/share/sysbadge
                  mv $out/bin/sysbadge-fw $out/share/sysbadge/sysbadge.elf
                  rm -r $out/bin

                  ${elf2uf2-rs}/bin/elf2uf2-rs $out/share/sysbadge/sysbadge.elf $out/share/sysbadge/sysbadge.uf2
                '';
              })) { };
          sysbadge_fw = final.callPackage
            ({ runCommand, sysbadge_fw_unwraped, elf2uf2-rs }:
              runCommand "sysbadge-fw" { buildInputs = [ elf2uf2-rs ]; } ''
                mkdir -p $out/share/sysbadge
                cp ${sysbadge_fw_unwraped}/bin/sysbadge-fw $out/share/sysbadge/sysbadge.elf
                elf2uf2-rs $out/share/sysbadge/sysbadge.elf $out/share/sysbadge/sysbadge.uf2
              '') { };
          sysbadge_wasm_unwraped = final.callPackage ({ lib, stdenv, fenix }:
            let
              system = stdenv.targetPlatform.system;
              toolchain = (fenixToolchain fenix);
              craneLib = crane.lib.${system}.overrideToolchain toolchain;
            in craneLib.buildPackage ((commonArgs craneLib {
              inherit lib stdenv toolchain;
              fw = false;
            }) // {
              cargoArtifacts = cargoArtifacts craneLib toolchain;
              pname = "sysbadge-wasm-unwraped";
              cargoExtraArgs =
                "-Z build-std=compiler_builtins,core,alloc,std,panic_abort --target wasm32-unknown-unknown --package sysbadge-web";
            })) { };
        };
      overlays.sysbadge_web = final: prev: {
        sysbadge_images = final.callPackage
          ({ runCommandNoCC, sysbadge_simulator }:
            runCommandNoCC "sysbadge-images" {
              buildInputs = [ sysbadge_simulator ];
            } ''
              mkdir -p $out/share/sysbadge

              EG_SIMULATOR_DUMP=$out/share/sysbadge/home.png sysbadge-simulator
              EG_SIMULATOR_DUMP=$out/share/sysbadge/version.png sysbadge-simulator B

              EG_SIMULATOR_DUMP=$out/share/sysbadge/one.png sysbadge-simulator C
            '') { };
        sysbadge_wasm = final.callPackage
          ({ runCommand, sysbadge_wasm_unwraped, wasm-bindgen-cli }:
            runCommand "sysbadge-wasm" {
              buildInputs = [ wasm-bindgen-cli ];
            } ''
              mkdir -p $out/share/sysbadge

              wasm-bindgen ${sysbadge_wasm_unwraped}/lib/sysbadge_web.wasm --out-dir $out/share/sysbadge/bundler --target bundler
            '') { };

        sysbadge_web = final.callPackage
          ({ mkYarnPackage, yarn2nix, runCommand, sysbadge_wasm }:
            let
              yarnLock = ./web/yarn.lock;
              yarnNix = if builtins ? currentSystem then
                let pkgs = nixpkgsFor.${builtins.currentSystem};
                in pkgs.runCommand "yarn.nix" { } ''
                  ${pkgs.yarn2nix}/bin/yarn2nix --lockfile ${yarnLock} --no-patch > $out
                ''
              else
                runCommand "yarn.nix" { } ''
                  ${yarn2nix}/bin/yarn2nix --lockfile ${yarnLock} --no-patch > $out
                '';
            in mkYarnPackage {
              name = "sysbadge-web";
              src = ./web;
              packageJSON = ./web/package.json;
              SYSBADGE_WASM_PATH = "${sysbadge_wasm}/share/sysbadge";

              buildPhase = ''
                export HOME=$(mktemp -d)
                yarn --offline build
              '';
              installPhase = ''
                mkdir -p $out
                cp -r deps/sysbadge-web/dist/* $out/
              '';
              distPhase = "true";

              inherit yarnNix yarnLock;
            }) { };
      };
      overlays.default = final: prev:
        (self.overlays.sysbadge_fw final prev)
        // (self.overlays.sysbadge_web final prev);

      legacyPackages = nixpkgsFor;

      packages = forAllSystems (system: {
        inherit (nixpkgsFor.${system})
          sysbadge_simulator sysbadge_fw sysbadge_wasm_unwraped sysbadge_wasm
          sysbadge_web probe-run;
      });

      apps = forAllSystems (system: {
        simulator = {
          type = "app";
          program = "${
              self.packages.${system}.sysbadge_simulator
            }/bin/sysbadge-simulator";
        };
        run = {
          type = "app";
          program = let
            script =
              self.legacyPackages.${system}.writeScript "sysbadge-probe" ''
                #!${self.legacyPackages.${system}.bash}/bin/bash
                exec ${
                  self.packages.${system}.probe-run
                }/bin/probe-run --chip rp2040 ${
                  self.packages.${system}.sysbadge_fw
                }/bin/sysbadge-fw
              '';
          in toString script;
        };
        default = self.apps.${system}.run;
      });

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
          shell = { lib, stdenv, mkShell, fenix, rust-analyzer-nightly, gdb
            , cargo-watch, cargo-edit, cargo-outdated, cargo-asm, libiconv
            , flip-link, probe-run, SDL2, just, yarn, wasm-bindgen-cli
            , elf2uf2-rs, libusb1, gtk4, glib, gdk-pixbuf, pkg-config }:
            mkShell {
              nativeBuildInputs = [
                (fenixToolchain fenix)
                #rust-analyzer-nightly
                cargo-watch
                cargo-edit
                cargo-outdated
                cargo-asm
                just
                flip-link
                probe-run
                elf2uf2-rs
                SDL2
                libusb1
                gtk4
                glib
                gdk-pixbuf
                pkg-config

                yarn
                wasm-bindgen-cli
              ] ++ lib.optional stdenv.isLinux gdb
                ++ lib.optional stdenv.isDarwin libiconv;
              inherit (self.checks.${system}.pre-commit) shellHook;
              NODE_OPTIONS = "--openssl-legacy-provider";
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
