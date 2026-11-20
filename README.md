# swc-plugin-react-source-string

SWC plugin that adds `data-source="path:line"` attributes to **every JSX element** for debugging — HTML tags, React components, icons, anything.

Rust-based equivalent of [babel-plugin-react-source-string](https://github.com/tanchu/babel-plugin-react-source-string) — designed for use with Next.js and Rspack.

## Installation

Choose the tag that matches your bundler:

| Bundler | Install command |
|---|---|
| **Next.js 16.4.x** | `npm install swc-plugin-react-source-string@next-16` |
| **Next.js 16.1.7 – 16.3.8** | `npm install swc-plugin-react-source-string@0.4.0` |
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
| `libraries` | `string[]` | — (tag all) | Tag only components imported from these packages or their subpaths (`@mui/material/Button` matches `@mui/material`); HTML elements are always tagged. |
| `preferCallSite` | `boolean` | `false` | Put the attribute before spread props, so a `data-source` passed in from the parent wins: wrapper components point to where they are used. See [Call site or wrapper](#call-site-or-wrapper-prefercallsite). |

## How it works

The plugin adds a `data-source` attribute to **every** JSX opening element — both HTML tags (`<div>`, `<span>`) and React components (`<Dialog>`, `<Pencil>`, `<Link>`). Elements listed in `excluded` are skipped; with `libraries` set, only components imported from those packages (named, default or namespace imports, so `<UI.Button>` counts when `UI` comes from one of them) are tagged.

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

## Call site or wrapper: `preferCallSite`

By default the attribute points to the JSX in your code that renders the DOM node. With wrapper components
that is the wrapper's own file:

```tsx
// src/components/MyButton.tsx
export function MyButton(props) {
  return <Button variant="contained" {...props} />;   // line 2
}

// src/pages/Checkout.tsx
<MyButton onClick={pay}>Pay</MyButton>                // line 42
```

Both elements are tagged. `<MyButton>` gets `data-source="src/pages/Checkout.tsx:42"` and passes it down through
`{...props}`, but inside the wrapper the plugin adds its own attribute *after* the spread, so it overrides the one
from the page. The DOM shows `src/components/MyButton.tsx:2`.

With `preferCallSite: true` the plugin puts its attribute *before* spread props, so a value coming from the parent
wins and the DOM shows `src/pages/Checkout.tsx:42`:

```js
// next.config.ts
const nextConfig: NextConfig = {
  experimental: {
    swcPlugins: [["swc-plugin-react-source-string", { root: process.cwd(), preferCallSite: true }]],
  },
};
```

- Elements without spread props are tagged exactly as before.
- With several wrappers that all spread their props (`Page` → `Toolbar` → `MyButton` → `Button`), the outermost call site wins.
- A wrapper that does not pass its props on to the DOM element (picks `className`, `onClick` explicitly) keeps pointing to itself: the value from the call site never reaches the DOM.

**When it helps**

- **Design systems and UI kits.** A shared `<MyButton>` or `<Card>` is used hundreds of times. The default only tells you
  "this is MyButton"; the call site tells you which screen and which line rendered *this* button.
- **Bug reports and QA.** Copy the attribute of a broken element from DevTools and you land in the page code that
  configures it — its props, text and handlers — instead of the shared component.
- **AI coding agents.** Point an agent at an element on the page and it edits that usage (say, the label on the checkout
  page) rather than changing a wrapper that the whole app depends on.

Keep the default when you are working on the wrappers themselves: then you want to land in the wrapper's file.

## Compatibility table

Each npm dist-tag is built against the exact `swc_core` version required by that bundler.
The plugin is automatically updated when the upstream bundler bumps its `swc_core`.

| npm tag | Bundler | `swc_core` | Rust toolchain |
|---|---|---|---|
| `next-16` · `latest` | Next.js 16.4.x | `79.0.0` | `nightly-2026-08-20` |
| `next-15` | Next.js 15.x | `50.x` | `nightly-2025-05-06` |
| `rspack` | Rspack latest | varies | `nightly-2025-05-06` |

Next.js 16.4.0 changed the plugin ABI: `0.5.x` fails on 16.3.x with
`Failed to deserialize program received from host`, and `0.4.0` fails the same way on 16.4.x.

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
