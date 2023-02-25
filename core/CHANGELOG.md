# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Chore

 - <csr-id-3c24838357ea5fe5f481a8744fb627add1da42f7/> Update versions to v1.1.0-prerelease01

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 48 commits contributed to the release over the course of 153 calendar days.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Update versions to v1.1.0-prerelease01 ([`3c24838`](https://github.com/1Password/typeshare/commit/3c24838357ea5fe5f481a8744fb627add1da42f7))
    - Merge pull request #27 from exoego/scala ([`5f1e585`](https://github.com/1Password/typeshare/commit/5f1e5851bd395af8a2b1cc25f763ca56fa9a1c8d))
    - Make format_generic_parameters take &mut self to match rest of Language API ([`02f4bc4`](https://github.com/1Password/typeshare/commit/02f4bc436eebc546217fa83624f93944eeb2ced2))
    - Fmt ([`ea1ec33`](https://github.com/1Password/typeshare/commit/ea1ec339305c1e6d5e7b2c208aa3da073e4e6930))
    - Update Scala to use mutable Language functions and the new ID formatter ([`e6b3f81`](https://github.com/1Password/typeshare/commit/e6b3f81c5d3ef9989ab1697d84dc5c0bbd07643a))
    - Merge branch 'main' of github.com:1Password/typeshare-staging into scala ([`8430a1f`](https://github.com/1Password/typeshare/commit/8430a1f0a8706359b903606b9a755115b1dcfbbb))
    - Merge pull request #66 from 1Password/jane/rework-attribute-parser ([`156052f`](https://github.com/1Password/typeshare/commit/156052f09f1845f5c73fccac67ba30e163a0a94e))
    - Remove final deprecated function, update comment parsing, and fix clippy ([`9524c7b`](https://github.com/1Password/typeshare/commit/9524c7b225c54c5b74922bc29511e07660f1e27e))
    - Fmt ([`b49bcb6`](https://github.com/1Password/typeshare/commit/b49bcb66d6e4be3a5c8e021448e92eaa35759c8a))
    - Reimplment get_decorators and remove parse_attr ([`dfdd465`](https://github.com/1Password/typeshare/commit/dfdd46599b24b5864324e9897624002cfca69d0c))
    - Fix clippy errors ([`e9202ff`](https://github.com/1Password/typeshare/commit/e9202ff161d3bf80e022e0eb8c431f8420724e6e))
    - Cleanup and removal of unused functions ([`37fc3be`](https://github.com/1Password/typeshare/commit/37fc3bef25d8a2d4bf67f6dc5494c85f1e5db3a6))
    - Update name-value attributes ([`6fe0037`](https://github.com/1Password/typeshare/commit/6fe00373914a84375881c1a00b8a84ebcd2daa4a))
    - Update is_skipped to use new logic ([`bbd7681`](https://github.com/1Password/typeshare/commit/bbd7681ae12a724fa037f4e540e0c12aa54b3fb5))
    - Implement get_serde_meta_items ([`4cf32f0`](https://github.com/1Password/typeshare/commit/4cf32f079318f927c32cc3d6f41ebffc36ee0d77))
    - Merge pull request #61 from Czocher/typescript-double-option ([`327c2d0`](https://github.com/1Password/typeshare/commit/327c2d01a1ec66481c402317076c8613f1e069cd))
    - Support the double option pattern for TypeScript ([`5d432fd`](https://github.com/1Password/typeshare/commit/5d432fd243ee0b8265b0d8a79e5446842c95e232))
    - Merge pull request #53 from adriangb/mutable-language ([`3de784a`](https://github.com/1Password/typeshare/commit/3de784a07b3f1a36dcd175c7d71ce1047f773f1b))
    - Merge pull request #1 from 1Password/fix-build-errors-in-mutable-language ([`5834c35`](https://github.com/1Password/typeshare/commit/5834c35820e22d8625ffd19e7ca580f57438a80e))
    - Add Scala snapshot tests for new tests ([`ba4af59`](https://github.com/1Password/typeshare/commit/ba4af591b2edc52b983069f53750cb81c787dd48))
    - Merge pull request #56 from 1Password/jane/kotlin/object-instead-of-class ([`42fad59`](https://github.com/1Password/typeshare/commit/42fad59c2fe8f6e05f77e285ae9c772cd1c481f8))
    - Merge branch 'main' into scala ([`3c006be`](https://github.com/1Password/typeshare/commit/3c006be4e465f6c7d6c41cfe9b9318abc349a8ee))
    - Fix build errors by moving borrows out of closures ([`75cb824`](https://github.com/1Password/typeshare/commit/75cb8241798716fb708d14dea77f99b61b771a34))
    - Merge pull request #52 from McAJBen/typescript_optional_type_alias ([`a0c7ceb`](https://github.com/1Password/typeshare/commit/a0c7ceb66eba742aa3a093e0205d12d432c33314))
    - Update tests ([`4634d87`](https://github.com/1Password/typeshare/commit/4634d870d4a4a566f89ec323f58e39449448b31b))
    - Fix typescript optional type alias formatting ([`b68ffb2`](https://github.com/1Password/typeshare/commit/b68ffb232b235f66cf9952b52bc83d86243e7881))
    - Empty structs should be represented as objects instead of classes in Kotlin ([`f1b3607`](https://github.com/1Password/typeshare/commit/f1b3607eb9bc95489a4aec4c793201921ccabc04))
    - Merge pull request #51 from 1Password/jane/support-unit-structs ([`c40cafe`](https://github.com/1Password/typeshare/commit/c40cafef9e03b1381843ef160f1ee2296913d26f))
    - Make the Language trait mutable ([`fdcf0c6`](https://github.com/1Password/typeshare/commit/fdcf0c69ebd0a26aa3fa2e2a87c818216258e859))
    - Merge pull request #10 from danieleades/clippy/general ([`e789fe7`](https://github.com/1Password/typeshare/commit/e789fe7a9f3014145a0eaa53ded24919b6ebff23))
    - Add support for unit structs ([`4fb9d9d`](https://github.com/1Password/typeshare/commit/4fb9d9dcd9f47598f2e470ed1f3bc91db45b8410))
    - Merge pull request #39 from 1Password/CerulanLumina/kotlin-immutable ([`58e15ec`](https://github.com/1Password/typeshare/commit/58e15ecdad90e80b18eb05d3e3ddc1e888fadeb5))
    - Update test data to match immutable Kotlin properties ([`3739341`](https://github.com/1Password/typeshare/commit/37393416764d233cb5484266c70b447fe9a4c742))
    - Change Kotlin to use val ([`c62880e`](https://github.com/1Password/typeshare/commit/c62880e8641c83bd408a743b2b12d03d0130ecb5))
    - Add Scala support ([`c067130`](https://github.com/1Password/typeshare/commit/c06713022ab07d28b33eea592f441b1701cf851a))
    - Invert boolean ([`4732875`](https://github.com/1Password/typeshare/commit/47328756876fca4b6a7bb72a39248bffa116dc6e))
    - Use 'String::new' to generate empty strings ([`602f964`](https://github.com/1Password/typeshare/commit/602f964ef69f2893e8bf2ab186db6a23c6978786))
    - Don't match on booleans ([`1c3b5ef`](https://github.com/1Password/typeshare/commit/1c3b5efc407e05c7ce35794e2fcc15aeacfb01a9))
    - Remove explicit 'iter' loops ([`e6d11ba`](https://github.com/1Password/typeshare/commit/e6d11ba62bc24d7cb1314a4675df722cdbf5f80f))
    - Use semicolons if nothing returned ([`217a322`](https://github.com/1Password/typeshare/commit/217a322ca205a5329dec2cbcf819e24d49344b00))
    - Collapse identical match arms ([`7a177c3`](https://github.com/1Password/typeshare/commit/7a177c3cfff033f6f585989ae670c1a428ee756e))
    - Use 'Self' to refer to own type ([`a1e90f1`](https://github.com/1Password/typeshare/commit/a1e90f1459f0a74d8180b33219054c2dafdeab69))
    - Fix CI ([`6bfcb03`](https://github.com/1Password/typeshare/commit/6bfcb03713db598ff1976957374c2648b28a3012))
    - Abbreviation -> acronym ([`2fe372e`](https://github.com/1Password/typeshare/commit/2fe372efa8fce720510c1c1b35a405d42e09ee05))
    - Fix CI ([`f9c51ca`](https://github.com/1Password/typeshare/commit/f9c51cac3997e515c200327a13f3571cb6b00479))
    - Add descriptions ([`d52bb0b`](https://github.com/1Password/typeshare/commit/d52bb0b66c6cfeaf0fa4652db8309874ed82dcc5))
    - Move version number from 0.1.0 -> 1.0.0 ([`f8e77b4`](https://github.com/1Password/typeshare/commit/f8e77b4e7f8c001c612c1f11aba05c248dc29787))
    - Initial port from private repo ([`fbf5aea`](https://github.com/1Password/typeshare/commit/fbf5aea145b7895f3a998db050bd972a9af79e05))
</details>

