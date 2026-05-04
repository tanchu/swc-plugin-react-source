# swc-plugin-react-source-string

SWC plugin that adds `data-source="path:line"` attributes to **every JSX element** for debugging — HTML tags, React components, icons, anything.

Rust-based equivalent of [babel-plugin-react-source-string](https://github.com/tanchu/babel-plugin-react-source-string) — designed for use with Next.js and Rspack.

## Installation

Choose the tag that matches your bundler:

| Bundler | Install command |
|---|---|
| **Next.js 16.x** | `npm install swc-plugin-react-source-string@next-16` |
| **Next.js 15.x** | `npm install swc-plugin-react-source-string@next-15` |
| **Rspack** | `npm install swc-plugin-react-source-string@rspack` |

> The SWC plugin ABI is tightly coupled to `swc_core` inside the bundler.
> Using the wrong tag will cause a `Mismatch` error at startup.

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

**Tip:** You probably only want this in development:

```ts
swcPlugins: [
  ...(process.env.NODE_ENV === "development"
    ? [["swc-plugin-react-source-string", {
          excluded: ["Fragment"],
          root: process.cwd()
        }]]
    : []),
],
```

## Plugin options

| Option     | Type       | Default | Description                                              |
| ---------- | ---------- | ------- | -------------------------------------------------------- |
| `excluded` | `string[]` | `[]`    | Component/element names to skip (case-insensitive).      |
| `root`     | `string`   | —       | Project root for relative paths. Use `process.cwd()`. Without it paths will be absolute. |

## How it works

The plugin adds a `data-source` attribute to **every** JSX opening element — both HTML tags (`<div>`, `<span>`) and React components (`<Dialog>`, `<Pencil>`, `<Link>`). Elements listed in `excluded` are skipped.

The attribute value is `relative/path/to/file.tsx:line`, making it easy to locate any DOM node back to its source from DevTools.

### Before

```tsx
// Example.jsx
<div className="wrapper">
  <Pencil size={16} />
  <Dialog open={isOpen}>
    <DialogContent>Hello</DialogContent>
  </Dialog>
</div>
```

### After

```html
<div class="wrapper" data-source="src/components/Example.tsx:1">
  <svg xmlns="http://www.w3.org/2000/svg" data-source="src/components/Example.tsx:2">
  </svg>
  <div data-source="src/components/Example.tsx:3">
    <div data-source="src/components/Example.tsx:4">Hello</div>
  </div>
</div>
```

> Components that spread props to their root element (e.g. lucide-react icons,
> Radix UI primitives, Next.js `<Link>`) will forward `data-source` to the DOM.
> Components that don't — simply ignore the extra prop; no runtime errors.

## Compatibility table

Each npm dist-tag is built against the exact `swc_core` version required by that bundler.
The plugin is automatically updated when the upstream bundler bumps its `swc_core`.

| npm tag | Bundler | `swc_core` | Rust toolchain |
|---|---|---|---|
| `next-16` · `latest` | Next.js 16.x | `57.0.0` | `nightly-2025-05-06` |
| `next-15` | Next.js 15.x | `50.x` | `nightly-2025-05-06` |
| `rspack` | Rspack latest | varies | `nightly-2025-05-06` |

## Building from source

Requires [Rust](https://rustup.rs/). The `rust-toolchain.toml` pins the nightly
version and WASM target automatically.

```bash
cargo build --release --target wasm32-wasip1
```

Output: `target/wasm32-wasip1/release/swc_plugin_react_source_string.wasm`

To use a local build instead of the npm package, point Next.js config directly to the `.wasm` file:

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
