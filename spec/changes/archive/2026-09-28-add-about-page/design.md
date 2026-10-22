# Design

## Context

Constellation is a native Matrix client built with `libcosmic` on the COSMIC desktop. `libcosmic` provides an `about` feature that exposes `cosmic::widget::about::About` and `cosmic::app::context_drawer::about`. Constellation's context drawer architecture uses a navigation stack (`settings_stack: Vec<SettingsPanel>`) to support drilling down into subpages with back-button support and Escape key dismissal.

External URL dispatching is already wired into the application message loop via `Message::OpenUrl(String)` using `open::that(url)` inside an async task.

## Goals / Non-Goals

**Goals:**
- Enable the `"about"` feature on `libcosmic` in `Cargo.toml`.
- Provide a dedicated `about_info()` constructor configured with Constellation's official application metadata, versioning, license, and project links.
- Add `SettingsPanel::About` to the settings drawer navigation stack.
- Support opening the About page from both the header user menu (`MenuAct::About`) and the App Settings overview index.
- Wire link activation callbacks to `Message::OpenUrl` so users can open the project repository, bug tracker, donation page, and license terms in their browser.
- Support back navigation when accessed via the App Settings hierarchy.

**Non-Goals:**
- Network-based fetching of dynamic contributor lists or sponsor tiers at runtime.
- Creating a bespoke custom widget layout when `libcosmic` already supplies the official COSMIC desktop About design.
- Standalone multi-window About dialogs; COSMIC conventions use context drawers.

## Decisions

### 1. Enable `about` feature on `libcosmic`
Enable `"about"` in `Cargo.toml` under `libcosmic` dependency features.
- *Rationale*: Grants access to `cosmic::widget::about` and `cosmic::app::context_drawer::about`.
- *Alternatives considered*: Hand-crafting an About page layout from basic widgets. Rejected because `libcosmic`'s native widget conforms to COSMIC design guidelines, handles responsive spacing, and standardizes section headers across desktop apps.

### 2. Centralized `about_info()` builder
Create a helper function `crate::view::about::about_info()` that constructs and returns `cosmic::widget::about::About`.
- *Configuration*:
  - Name: "Constellation"
  - App ID / Icon: `fi.joonastuomi.Constellation` (`widget::icon::from_name(APP_ID)`)
  - Version: `env!("CARGO_PKG_VERSION")`
  - Author: "Joonas Tuomi"
  - License: "Apache-2.0"
  - License URL: `"https://www.apache.org/licenses/LICENSE-2.0"`
  - Developers: `[("Joonas Tuomi", "me@joonastuomi.fi")]`
  - Links:
    - ("Repository", `"https://github.com/jotuel/constellation"`)
    - ("Support / Issues", `"https://github.com/jotuel/constellation/issues"`)
    - ("Donate", `"https://github.com/sponsors/jotuel"`)
- *Rationale*: Centralizing metadata in a constructor keeps it consistent, easy to update alongside releases, and decoupled from the drawer rendering logic.
- *Alternatives considered*: Storing an `About` instance in `Constellation` state. Rejected because `About` is lightweight to build on demand when the drawer is rendered, avoiding unnecessary persistent memory or lifecycle synchronization.

### 3. Drawer integration via `SettingsPanel::About`
Add `SettingsPanel::About` variant to `SettingsPanel`. In `src/constellation/app.rs`, when the active panel is `SettingsPanel::About`, render `cosmic::app::context_drawer::about(&about, |url| Message::OpenUrl(url.to_string()), Message::CloseSettings)`.
- *Title & Actions*: Set `.title(crate::fl!("about"))` on the drawer. If `self.settings_stack.len() > 1` (navigated from App Settings overview), attach the standard back button (`Message::SettingsBack`) via `.actions(...)`.
- *Rationale*: Reuses the existing drawer lifecycle, Escape key handling, and back-stack semantics.

### 4. Menu and App Settings overview entry points
- Add `MenuAct::About` to `MenuAct`, mapped to `Message::OpenSettings(SettingsPanel::About)`. Insert an "About" button into the header user dropdown menu.
- Add an "About" `category_row` in `src/settings/app/view.rs` with version summary pointing to `Message::OpenPanel(crate::SettingsPanel::About)`.
- *Rationale*: Provides discoverability both for quick access from the global header and within application settings.

## Risks / Trade-offs

- **[Feature flag drift]** → If `about` is not enabled in all feature matrix combinations (e.g. `--no-default-features`), compilation could fail if code unconditionally references `cosmic::widget::about`.
  - *Mitigation*: Ensure `libcosmic` includes `about` in its base dependency features list in `Cargo.toml`.
- **[External URL opening failures]** → Clicking external links could silently fail if the desktop lacks a default URI handler.
  - *Mitigation*: `Message::OpenUrl` dispatches through `open::that` inside Tokio background execution, preventing desktop freeze.
