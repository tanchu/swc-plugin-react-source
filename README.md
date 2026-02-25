# swc-plugin-react-source-string

SWC plugin that adds `data-source="path:line"` attributes to **every JSX element** for debugging — HTML tags, React components, icons, anything.

Rust-based equivalent of [babel-plugin-react-source-string](https://github.com/tanchu/babel-plugin-react-source-string) — designed for use with Next.js SWC compiler.

## Installation

```bash
npm install swc-plugin-react-source-string
```

## Usage with Next.js

Add the plugin to `next.config.ts` (or `next.config.js`):

```ts
// next.config.ts
import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  experimental: {
    swcPlugins: [
      [
        "swc-plugin-react-source-string",
        {
          excluded: ["Fragment", "Slot"],
          root: process.cwd(),
        },
      ],
    ],
  },
};

export default nextConfig;
```

> **Tip:** You probably only want this in development. Wrap the plugin entry
> with a condition:
>
> ```ts
> swcPlugins: [
>   ...(process.env.NODE_ENV === "development"
>     ? [["swc-plugin-react-source-string", { excluded: ["Fragment"], root: process.cwd() }] as const]
>     : []),
> ],
> ```

## Plugin options

| Option     | Type       | Default | Description                                              |
| ---------- | ---------- | ------- | -------------------------------------------------------- |
| `excluded` | `string[]` | `[]`    | Component/element names to skip (case-insensitive).      |
| `root`     | `string`   | —       | Optional. Project root for relative paths (use `process.cwd()`). Without it paths will be absolute. |

### Example config

```json
{
  "excluded": ["Fragment", "Slot"]
}
```

## How it works

The plugin adds a `data-source` attribute to **every** JSX opening element — both HTML tags (`<div>`, `<span>`) and React components (`<Dialog>`, `<Pencil>`, `<Link>`). Elements listed in `excluded` are skipped.

The attribute value is `relative/path/to/file.tsx:line`, making it easy to locate any DOM node back to its source from DevTools.

When `root` is provided, file paths are relative to the project root. Without it, the plugin falls back to the SWC experimental context `cwd`, or uses absolute paths.

### Before

```tsx
<div className="wrapper">
  <Pencil size={16} />
  <Dialog open={isOpen}>
    <DialogContent>Hello</DialogContent>
  </Dialog>
</div>
```

### After

```tsx
<div className="wrapper" data-source="src/components/Example.tsx:1">
  <Pencil size={16} data-source="src/components/Example.tsx:2" />
  <Dialog open={isOpen} data-source="src/components/Example.tsx:3">
    <DialogContent data-source="src/components/Example.tsx:4">Hello</DialogContent>
  </Dialog>
</div>
```

> Components that spread props to their root element (e.g. lucide-react icons,
> Radix UI primitives, Next.js `<Link>`) will forward `data-source` to the DOM.
> Components that don't — simply ignore the extra prop; no runtime errors.

## Compatibility

The SWC plugin ABI is tightly coupled to specific versions of `swc_core`, `@swc/core`, and Next.js.
**You must use the correct combination**, otherwise the plugin will fail to load at runtime.

| `swc_core` (Cargo.toml) | Rust toolchain        | `@swc/core`   | Next.js  |
| ------------------------ | --------------------- | ------------- | -------- |
| `36.x`                   | `nightly-2025-05-06`  | `1.11.x`      | `15.5.x` |

> To target a different Next.js version, align `swc_core` in `Cargo.toml` with the
> version from the [official SWC plugins repo](https://github.com/swc-project/plugins)
> matching your `@swc/core` / `@next/swc` version, and update `rust-toolchain.toml`
> to match the nightly used at that commit.

## Building from source

Requires [Rust](https://rustup.rs/). The `rust-toolchain.toml` pins the nightly
version and WASM target automatically.

```bash
cargo build --release --target wasm32-wasip1
```

Output: `target/wasm32-wasip1/release/swc_plugin_react_source_string.wasm`

To use the local build instead of the npm package, point Next.js config to the
`.wasm` path directly:

```ts
import path from "path";

swcPlugins: [
  [
    path.resolve(__dirname, "./target/wasm32-wasip1/release/swc_plugin_react_source_string.wasm"),
    { excluded: ["Fragment"], root: process.cwd() },
  ],
],
```

## Related

- [babel-plugin-react-source-string](https://github.com/tanchu/babel-plugin-react-source-string) — Babel equivalent of this plugin

## License

MIT
