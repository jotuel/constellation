---
name: cosmic
description: Guidelines, best practices, UI automation, and performance profiling for System76 COSMIC desktop applications and applets using libcosmic.
license: MIT
metadata:
  version: "2026-09-28"
  source: "https://github.com/jotuel/cosmic-skills"
---

# COSMIC Desktop Development, UI Automation, and Profiling

This skill provides comprehensive guidance, domain knowledge, best practices, UI automation workflows, and profiling tools for developing applications and applets for the System76 COSMIC Desktop Environment (COSMIC DE) using Rust and `libcosmic`.

## When to Use This Skill
Use this skill when:
- Creating, modifying, or reviewing native COSMIC applications or applets using `libcosmic`.
- Implementing Model-View-Update (MVU / Elm architecture) patterns or custom widget styling.
- Automating Wayland UI interactions and capturing screenshots for visual regression or layout inspection.
- Profiling execution bottlenecks, CPU hotspots, or memory heap allocations.

## Subskill References

For in-depth documentation and domain guides, refer to the following reference documents in the `references/` directory:

* **[Development Guide](references/development.md)** (`references/development.md`):
  - Architecture: Model-View-Update (MVU) pattern in `libcosmic` and `iced`.
  - Design & Widgets: Proper usage of `cosmic::widget::button::custom` with `ListItem` styling, selection highlights, and avoiding blocky buttons.
  - Theming: Semantic theme palettes (`theme.palette.*`), avoiding hardcoded colors, and accessing system themes via `self.core.system_theme()`.
  - Cosmic Stack: Overview of `cosmic-text`, `cosmic-config`, `cosmic-theme`, and `cosmic-comp`.
  - Troubleshooting: Feature flags in `Cargo.toml`, theming propagation, and decrypting lazy store mismatches (e.g. SQLite vs cache directories).

* **[UI Automation Guide](references/ui-automation.md)** (`references/ui-automation.md`):
  - Wayland input synthesis via `wtype` (virtual keyboard simulation).
  - Native screenshot capture via `cosmic-screenshot`.
  - Visual layout verification (detecting text overflow, clipped buttons, missing separators, and focus indicators).
  - Automated test execution using `scripts/test_ui.sh`.

* **[Profiling Guide](references/profiling.md)** (`references/profiling.md`):
  - CPU execution hotspot analysis using `samply` and Firefox Profiler.
  - Memory heap allocation and leak tracking using `heaptrack` and `heaptrack_gui`.
  - Automated profiling sessions under deterministic UI load using `scripts/profile_run.sh`.

## Bundled Automation Scripts

Executable helper scripts located in `scripts/`:

* **`scripts/test_ui.sh`**: Builds the target application, starts it in the background, simulates Wayland keyboard input, captures a layout screenshot to `./scratch/screens/ui_state_target.png`, and terminates the process.
* **`scripts/profile_run.sh [samply|heaptrack]`**: Builds the binary in release mode with debug symbols (`RUSTFLAGS="-g"`), launches the specified profiler, runs simulated interaction under load, captures verification screenshots, and generates profiling output.

## Quick-Reference Development Rules

1. **Never Hardcode Colors**: Always use semantic colors from `theme.palette` (e.g., `theme.palette.primary`, `theme.palette.background`) so the UI responds properly to light/dark themes.
2. **List & Sidebar Item Styling**: Do not use generic buttons in room lists, sidebars, or settings menus. Use:
   ```rust
   cosmic::widget::button::custom(content)
       .class(cosmic::theme::Button::ListItem(radii))
       .selected(is_selected)
   ```
3. **Theme Access**: Access current system theme and metrics via `self.core.system_theme()` if not passed directly into view helpers.
4. **Encrypted & Lazy Stores**: When invalidating or clearing primary store credentials/databases, ensure all secondary lazy-loaded cache directories are cleared concurrently to prevent decryption/MAC failures.
