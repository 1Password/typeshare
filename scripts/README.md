# Deploy binary to releases

This script is used to deploy a binary to the releases page of a GitHub repository.

## Pre-requisites
1. `gh cli` must be installed and authenticated.

Installation:
```sh
brew install gh
```

Authentication:
```sh
gh auth login
```

2. Make sure a release has already been created and triggered. To do this make sure to tag the commit with the version number.

```sh
git tag -a v1.12.0 -m "Release v1.12.0"
git push origin v1.12.0
```

3. Monitor the release pipeline here: https://github.com/gitarcode/typeshare/actions/workflows/release.yml

## Usage

```sh
Usage: ./build.sh [options]
Options:
  --version <version>    Version of the release
  --target <platform>    Target platform
                           Platforms:
                             * "aarch64-apple-darwin"
                             * "x86_64-apple-darwin"
                             * "aarch64-unknown-linux-gnu"
                             * "x86_64-unknown-linux-gnu"
                             * "x86_64-pc-windows-msvc"
```

## Example

```sh
./build.sh --version v1.12.0 --target aarch64-apple-darwin
```
