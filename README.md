# swc-plugin-react-source-string

SWC plugin that adds `data-source="path:line"` attributes to JSX elements for debugging. Works with both HTML elements and configured UI library components.

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
          libraries: [
            "@radix-ui/react-dialog",
            "@radix-ui/react-slot",
            "lucide-react",
          ],
          excluded: ["Fragment", "Slot"],
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
>     ? [["swc-plugin-react-source-string", { libraries: [...], excluded: [...] }] as const]
>     : []),
> ],
> ```

## Plugin options

| Option      | Type       | Default | Description                                                                                          |
| ----------- | ---------- | ------- | ---------------------------------------------------------------------------------------------------- |
| `libraries` | `string[]` | `[]`    | Package names (or prefixes) to treat as UI libraries. Imports from these packages get `data-source`. |
| `excluded`  | `string[]` | `[]`    | Component/element names to skip (case-insensitive).                                                  |

### Example config

```json
{
  "libraries": [
    "@radix-ui/react-dialog",
    "@radix-ui/react-slot",
    "lucide-react"
  ],
  "excluded": ["Fragment", "Slot"]
}
```

## How it works

1. **HTML elements** (lowercase JSX tags like `<div>`, `<span>`) always receive a `data-source` attribute.
2. **UI library components** — when a component is imported from a package listed in `libraries`, its JSX usage gets `data-source` too.
3. **Excluded names** — tags/components listed in `excluded` are skipped (case-insensitive).
4. The attribute value is `relative/path/to/file.tsx:line`, making it easy to locate the source from DevTools.

### Before

```tsx
<div className="wrapper">
  <Dialog open={isOpen}>
    <DialogContent>Hello</DialogContent>
  </Dialog>
</div>
```

### After

```tsx
<div className="wrapper" data-source="src/components/Example.tsx:3">
  <Dialog open={isOpen} data-source="src/components/Example.tsx:4">
    <DialogContent data-source="src/components/Example.tsx:5">Hello</DialogContent>
  </Dialog>
</div>
```

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
    { libraries: ["@radix-ui/react-dialog"], excluded: ["Fragment"] },
  ],
],
```

## Related

- [babel-plugin-react-source-string](https://github.com/tanchu/babel-plugin-react-source-string) — Babel equivalent of this plugin

## License

MIT
