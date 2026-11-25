# Working on swc-plugin-react-source-string

An SWC plugin (Rust, compiled to WebAssembly) that adds `data-source="path:line"` to every JSX element, plus an optional dev overlay (`overlay: true`). It is used with Next.js. It has the same options and output as [vite-plugin-react-source-string](https://github.com/tanchu/vite-plugin-react-source) and [babel-plugin-react-source-string](https://github.com/tanchu/babel-plugin-react-source-string).

## Branches

- **`next-16`** is the working branch: open PRs against it. It builds against the `swc_core` version of Next.js 16.4.x and publishes to npm under `latest` and `next-16`.
- `main` is not released from.
- An SWC plugin only loads in a bundler with the same plugin ABI. A mismatch fails at startup with `Mismatch` or `Failed to deserialize program received from host`. The `sync-swc` workflow bumps `swc_core` when Next.js does.

## Layout

- `src/lib.rs`: the transform and its unit tests.
- `overlay.js`: the overlay runtime, imported into every tagged file when `overlay` is on. **Generated, do not edit by hand** (see below).
- `swc_plugin_react_source_string.wasm`: release build output, published to npm.

## Commands

The Rust toolchain and the `wasm32-wasip1` target are pinned in `rust-toolchain.toml`.

```bash
cargo test --lib
cargo clippy --target wasm32-wasip1 -- -D warnings
cargo build --release --target wasm32-wasip1
```

## Rules

- **`overlay.js` comes from the Vite plugin.** Its source is `src/client.ts` in [vite-plugin-react-source](https://github.com/tanchu/vite-plugin-react-source). Change it there, then regenerate from that repository: `pnpm build && node scripts/gen-overlay.mjs ../swc-plugin-react-source/overlay.js`. The file must stay a plain script with no imports or exports.
- **The overlay import goes after directives**, so `"use client"` stays the first statement.
- **Options and output stay identical across the three plugins.** A new option or a change to what gets tagged needs the same change in the Vite and Babel plugins, and in the agent skill at [tanchu/react-source-skills](https://github.com/tanchu/react-source-skills).
- **Do not move the source into accessibility attributes** (`title`, `aria-*`). `title` can replace the accessible name and breaks screen readers and role-based tests.
- **Fragments are never tagged** (`<>`, `<Fragment>`, `<React.Fragment>`).
- **Commits follow [Conventional Commits](https://www.conventionalcommits.org/).** release-please reads them: `fix:` makes a patch release, `feat:` a minor one, `docs:`, `test:`, `ci:` and `chore:` make none. Merging the release PR publishes to npm, so leave that to the maintainer.
