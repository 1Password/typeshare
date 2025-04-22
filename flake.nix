# Nix Flake for installing typeshare outside of the NixPkgs source tree.
#
# Thank you to figsoda, who originally wrote the package build and install
# instructions. I (savannidgerinel) copied that stanza from the nixpkgs
# repository.
# https://github.com/NixOS/nixpkgs/blob/nixos-unstable/pkgs/development/tools/rust/typeshare/default.nix#L32
#
# To use this in your repository,
{
  description = "Create types in Rust and convert them to other languages";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        versionFromCargo = path: (builtins.fromTOML (builtins.readFile (path + "/Cargo.toml"))).package.version;
        license = let licenses = pkgs.lib.licenses; in [licenses.asl20 /* or */ licenses.mit ];
        homepage = "https://github.com/1password/typeshare";
      in {
        packages = rec {
          typeshare2 = pkgs.rustPlatform.buildRustPackage {
            pname = "typeshare2";
            version = versionFromCargo ./app/cli;
            # version = (builtins.fromTOML (builtins.readFile ./app/cli/Cargo.toml)).package.version;
            src = pkgs.lib.cleanSource ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
            nativeBuildInputs = [ pkgs.installShellFiles ];

            postInstall = ''
              installShellCompletion --cmd typeshare2 \
                --bash <($out/bin/typeshare2 completions bash) \
                --fish <($out/bin/typeshare2 completions fish) \
                --zsh <($out/bin/typeshare2 completions zsh)
            '';

            meta = {
              description = "Command Line Tool for generating language files with typeshare";
              inherit homepage;
              # TODO: restore this
              # changelog = "https://github.com/1password/typeshare/blob/v${version}/CHANGELOG.md";
              inherit license;
            };
          };

          typeshare-snapshot-test = pkgs.rustPlatform.buildRustPackage {
            pname = "typeshare-snapshot-test";
            version = versionFromCargo ./typeshare-snapshot-test;
            src = pkgs.lib.cleanSource ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
            # nativeBuildInputs = [ pkgs.installShellFiles ];

            # postInstall = ''
            #   installShellCompletion --cmd typeshare2 \
            #     --bash <($out/bin/typeshare2 completions bash) \
            #     --fish <($out/bin/typeshare2 completions fish) \
            #     --zsh <($out/bin/typeshare2 completions zsh)
            # '';

            meta = {
              description = "Snapshot testing tool for typeshare";
              inherit homepage;
              # TODO: restore this
              # changelog = "https://github.com/1password/typeshare/blob/v${version}/CHANGELOG.md";
              inherit license;
            };
          };

          default = typeshare2;
        };
      }
    );
}
