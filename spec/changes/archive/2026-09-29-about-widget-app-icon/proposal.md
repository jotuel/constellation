# Proposal

## Why

The About context drawer currently attempts to resolve its icon by looking up the desktop App ID (`fi.joonastuomi.Constellation`) in the system icon theme via `icon::from_name`. In environments where Constellation is not installed to system icon directories (such as local development runs, standalone binaries, or uninstalled testing), the lookup fails to resolve the desktop asset, causing the About widget to render without the application icon.

## What Changes

- Update `crate::view::about::about_info()` to provide Constellation's bundled SVG icon (`crate::CONSTELLATION_ICON`) to the `About` builder using `cosmic::widget::icon::from_svg_bytes`.
- Add assertions in `src/view/about.rs` unit tests verifying that the configured `About` metadata model contains an icon handle and renders correctly with the bundled SVG asset.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `about-page`: Clarify that the application about presentation displays the application icon sourced from the bundled application icon asset.

## Impact

- **Affected areas**: `src/view/about.rs` (`about_info()` and unit tests).
- **Breaking changes**: None.
- **Dependencies**: None; reuses existing `cosmic::widget::icon::from_svg_bytes` and `crate::CONSTELLATION_ICON`.
