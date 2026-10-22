# Design

## Context

In `src/view/about.rs`, the `about_info()` constructor configures the `cosmic::widget::about::About` metadata model used by the About context drawer. Currently, line 10 calls:
```rust
.icon(icon::from_name(crate::Constellation::APP_ID))
```
`libcosmic`'s `About` widget accepts an `Option<widget::icon::Handle>`. When rendered via `cosmic::widget::about::about`, it extracts this handle to render the application header image.

Constellation already embeds its primary SVG icon (`res/const.svg`) into the binary via `pub const CONSTELLATION_ICON: &[u8] = include_bytes!("../res/const.svg");` in `src/lib.rs`. This asset is already used by `src/view/app.rs` for the empty state watermark view.

## Goals / Non-Goals

**Goals:**
- Configure `About::default()` in `src/view/about.rs` with `cosmic::widget::icon::from_svg_bytes(crate::CONSTELLATION_ICON)`.
- Ensure the application icon renders reliably in the About drawer across all environments (local development, flatpak, distribution packages, standalone AppImage).
- Add unit test coverage in `src/view/about.rs` verifying that the `About` model contains the SVG icon handle and renders without error.

**Non-Goals:**
- Modifying desktop entries (`res/fi.joonastuomi.Constellation.desktop`) or packaging scripts.
- Changing `libcosmic`'s upstream About widget layout or icon dimensions.
- Introducing dynamic filesystem lookups for external icon files at runtime.

## Decisions

### 1. Sourcing the icon from embedded SVG bytes
Use `cosmic::widget::icon::from_svg_bytes(crate::CONSTELLATION_ICON)`.
- *Rationale*: Guarantees zero-allocation asset bundling at compile time, eliminating dependence on system-installed icon themes in `/usr/share/icons/` or `XDG_DATA_DIRS`. `crate::CONSTELLATION_ICON` is already linked and statically available in the binary.
- *Alternatives considered*:
  - `cosmic::widget::icon::from_name(crate::Constellation::APP_ID)`: Requires system-level icon theme installation; fails during development or portable execution.
  - Adding fallback paths to `Named`: Complex and still requires disk I/O, whereas the SVG is already bundled in memory.

### 2. Testing icon presence in `src/view/about.rs`
Update `test_about_info_metadata()` to assert that `format!("{about:?}")` includes the SVG icon handle (`Data::Svg`). Keep `test_about_widget_renders()` to confirm the widget builds an element from the model.
- *Rationale*: Explicitly catches any accidental regression where `icon(...)` is removed or replaced with an unresolved lookup.

## Risks / Trade-offs

- **[Memory footprint of embedded SVG]** → `res/const.svg` is 934 bytes, already embedded in `src/lib.rs` for the main view. Reusing `crate::CONSTELLATION_ICON` adds no additional binary bloat.
