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

  outputs = inputs@{ self, nixpkgs, flake-utils }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
        typeshare = with pkgs;
          rustPlatform.buildRustPackage rec {
          pname = "typeshare";
          version = (builtins.fromTOML (builtins.readFile ./cli/Cargo.toml)).package.version;

          src = lib.cleanSource ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          nativeBuildInputs = [ installShellFiles ];

          buildFeatures = [ "go" ];

          postInstall = ''
            installShellCompletion --cmd typeshare \
              --bash <($out/bin/typeshare completions bash) \
              --fish <($out/bin/typeshare completions fish) \
              --zsh <($out/bin/typeshare completions zsh)
          '';

          meta = with lib; {
            description = "Command Line Tool for generating language files with typeshare";
            homepage = "https://github.com/1password/typeshare";
            changelog = "https://github.com/1password/typeshare/blob/v${version}/CHANGELOG.md";
            license = with licenses; [ asl20 /* or */ mit ];
          };
        };
    in rec {
      packages.default = typeshare;
    });
}
