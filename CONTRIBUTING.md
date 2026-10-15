# Contributing to Aether

Thanks for your interest in Aether! Bug reports, feature ideas and pull requests are welcome.

## Before you start

- For anything larger than a small fix, please open an issue first so we can agree on the approach.
- Report security problems privately, see [SECURITY.md](SECURITY.md).
- See [ARCHITECTURE.md](ARCHITECTURE.md) for an overview of the project structure.

## Development setup

Requirements and the basic build commands are in the [README](README.md#building-from-source).

Before opening a pull request, make sure these pass locally:

```sh
bun run lint:rust                 # clippy (warnings are errors)
cargo test --workspace            # Rust tests
cd apps/frontend
bun run lint                      # types and eslint must pass (architecture report is informational)
bun run test --run                # frontend tests
```

The pre-commit hooks (lefthook) run the main checks for the files you changed. Please do not bypass them.

## Commits and pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, ...).
- Write commit messages, PR titles and PR descriptions in English.
- Keep pull requests focused: one logical change per PR.
- Do not reference issues or pull requests of other repositories in commits or PR descriptions.

## Licensing of contributions

Aether is licensed under [GPL-3.0-only](LICENSE). The plugin API crates in `packages/core` (`aether-core-plugin-api`, `register-schema`, `register-schema-derive`) are licensed under `MIT OR Apache-2.0`. Your contribution is licensed under the license of the part of the project it modifies.

By submitting a pull request you agree to the following (this is the project's Contributor License Agreement):

1. You wrote the contribution yourself, or you otherwise have the right to submit it under these terms (for example, it is not copied from code under an incompatible license, and your employer does not own it).
2. You keep the copyright in your contribution.
3. You grant the project maintainer a perpetual, worldwide, non-exclusive, royalty-free, irrevocable license to use, modify, sublicense and distribute your contribution **under any license, including a different open source or proprietary license**. This lets the maintainer change the project's license in the future. Versions already released under the GPL stay available under the GPL.
4. You understand that your contribution and your name or username in the commit history are public.

If you cannot agree to these terms, please do not submit a pull request; open an issue instead. The pull request template asks you to confirm this agreement explicitly.

## Minecraft and third-party content

Aether is not affiliated with Mojang or Microsoft. Do not add Minecraft game files, assets or logos to the repository, and do not add features whose purpose is to bypass Minecraft's purchase or authentication requirements.
