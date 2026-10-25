# egui-lucide

[Lucide](https://lucide.dev) SVG icons for [egui](https://github.com/emilk/egui), with runtime **color**, **stroke width**, and **size**.

Icons are generated from a pinned Lucide release (`1.61.0`) and compiled into the crate. You do not download SVGs at build time.

## Usage

Register egui's SVG loader once at startup (this crate does not do it for you):

```rust
egui_extras::install_image_loaders(&ctx);
```

`egui_extras` needs the `svg` feature:

```toml
egui_extras = { version = "0.36", features = ["svg"] }
egui-lucide = "0.1"
```

```rust
use egui_lucide::Lucide;

// Inherits `ui.visuals().text_color()` when color is unset.
ui.add(Lucide::House.size(18.0).stroke_width(1.75));

// Explicit color — works with `Image` / `ImageSource` (no `Ui` yet):
let image = Lucide::House
    .size(16.0)
    .color(egui::Color32::from_rgb(0x7a, 0xa2, 0xf7))
    .stroke_width(2.0)
    .image();
ui.add(image);
```

`LucideIcon::image()` returns `egui::Image<'static>`, so it plugs into widgets that take an image (for example `egui_elements::widgets::Label`).

## How styling works

Lucide SVGs use `stroke="currentColor"` and `stroke-width="2"` on a `0 0 24 24` viewBox. `resvg` (egui's SVG loader) does not reliably honor `currentColor`, so this crate wraps each icon's inner paths with a root `<svg>` that bakes in **color** and **stroke-width**.

**Size** is applied on the `Image` (`fit_to_exact_size`), not by rewriting path data. Stroke stays in viewBox units, same as [lucide.dev](https://lucide.dev).

The `bytes://` URI includes color and stroke and **ends with `.svg`** (`bytes://lucide/house.c=….sw=….svg`). `egui_extras`' SVG loader only matches that suffix (query strings after `.svg` are ignored). Changing color or stroke is a new cache key, so egui will not keep showing the old texture.

## Regenerating icons

From a Lucide GitHub archive (do not commit the tarball):

```sh
curl -sL -o /tmp/lucide-1.61.0.tar.gz \
  https://github.com/lucide-icons/lucide/archive/refs/tags/1.61.0.tar.gz
python3 scripts/generate.py --archive /tmp/lucide-1.61.0.tar.gz --version 1.61.0
```

That overwrites `src/generated.rs`. Bump `LUCIDE_VERSION` by passing `--version` to match the tag.

## License

- Crate code: MIT (`LICENSE-MIT`)
- Icon artwork: ISC, with a Feather-derived subset under MIT (`LICENSE-LUCIDE`)
