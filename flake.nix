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
  flake-utils.lib.eachSystem ["x86_64-linux"] (system:
    let pkgs = import nixpkgs { inherit system; };
        typeshare = { version_, cargoHash_, packageHash }:
          with pkgs;
          rustPlatform.buildRustPackage rec {
          pname = "typeshare";
          version = version_;

          src = fetchFromGitHub {
            owner = "1password";
            repo = "typeshare";
            rev = "v${version}";
            hash = packageHash;
          };

          cargoHash = cargoHash_;

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
# If you are doing a release and want to update this, follow these steps:
#
# Copy one of the `typeshare-?_?` stanzas. Replace both of the hashes with empty strings.
#
# Run `nix build .#typeshare-?_?`, replacing the question marks with your new version.
#
# The build will fail due to hash mismatches. On the first failure, copy
# the hash that Nix says into the `packageHash` parameter. On the second
# failure, copy the hash into `cargoHash_`.
#
# For instance:
# nix build .#typeshare-1_5
# warning: Git tree '/home/savanni/src/typeshare' is dirty
# warning: found empty hash, assuming 'sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA='
# warning: found empty hash, assuming 'sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA='
# error: hash mismatch in fixed-output derivation '/nix/store/mik2pkv2c0mxbizgwqn7h9kr6npxakww-source.drv':
#          specified: sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
#             got:    sha256-Zmb6GZVtjx/PXOT1vaxKjPObY902pRqttOYExDx5UvI=
# error: 1 dependencies of derivation '/nix/store/k7s9fhxw9hpa88rrhrvfn34xsvk8b73q-typeshare-1.5.0.drv' failed to build
# 
# typeshare on  main [!+?] via 🦀 
# ❯ nix build .#typeshare-1_5
# warning: Git tree '/home/savanni/src/typeshare' is dirty
# warning: found empty hash, assuming 'sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA='
# error: hash mismatch in fixed-output derivation '/nix/store/k1ws54bs6cmfqdysjb1p0ls5wsyy2f5f-typeshare-1.5.0-vendor.tar.gz.drv':
#          specified: sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
#             got:    sha256-83LAZ7b1j/iBnYmY0oSSWDH0w7WPU1O85X+IBwSe1bs=
# error: 1 dependencies of derivation '/nix/store/805bj0nphw5136byhhxncl2gy1bhkjhq-typeshare-1.5.0.drv' failed to build
# 
# typeshare on  main [!+?] via 🦀 took 6s 
# ❯ nix build .#typeshare-1_5
# warning: Git tree '/home/savanni/src/typeshare' is dirty
# [1/0/1 built, 0.0 MiB DL] building typeshare-1.5.0 (unpackPhase): unpacking source archive /nix/store/6mbpmy7mk1z4k06xpsy6vb7g4
# 

      packages.default = packages.typeshare-1_5;

      packages = rec {
        typeshare-1_5 = typeshare {
          version_ = "1.5.0";
          cargoHash_ = "sha256-83LAZ7b1j/iBnYmY0oSSWDH0w7WPU1O85X+IBwSe1bs=";
          packageHash = "sha256-Zmb6GZVtjx/PXOT1vaxKjPObY902pRqttOYExDx5UvI=";
        };
        typeshare-1_4 = typeshare {
          version_ = "1.4.0";
          cargoHash_ = "sha256-hF+1v9bHioKQixg0C46ligLy/ibU+iI/H85g4wQhne4=";
          packageHash = "sha256-TGs7Czq13ghifKUhoz+n9I4UlOrzQosWTwBqBWv572E=";
        };
        typeshare-1_3 = typeshare {
          version_ = "1.3.0";
          cargoHash_ = "sha256-55DBzItGgUs6TroDeOAJPd7Koy4cyUV8SdqxUhKXwrU=";
          packageHash = "sha256-rP5d85/wGNimzOgsNDaX/QHZsGU5HoBAJsrETBKtRF4=";
        };
        typeshare-1_2 = typeshare {
          version_ = "1.2.0";
          cargoHash_ = "sha256-kMmjuPR5h2sVcnilUVt0SEZYcOEgXzM8fPC6Ljg6+d0=";
          packageHash = "sha256-zY1Z2TT1D3mgnnepRih88U+tpPQWWnAtxt5yAVuoBbk=";
        };
        typeshare-1_0_1 = typeshare {
          version_ = "1.0.1";
          cargoHash_ = "sha256-6mwcBeH3d6My6bjUF1KaqTH/C+qk6jYZ/+v8bWwXI3A=";
          packageHash = "sha256-cK3XISg8SEXbHQHnumXCZD4oFjt4QT/uC4MrpBpnAxU=";
        };
      };
    });
}
