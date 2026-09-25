# Proposal

## Why

App Settings is currently the only settings interface in Constellation implemented as a monolithic single file (`src/settings/app.rs`) presenting all controls (appearance toggles, typing notifications, diagnostic session errors, cache maintenance, and keyboard shortcuts) in a single continuous scroll. Unlike User Settings, Room Settings, and Space Settings, App Settings does not follow the multi-file module pattern (`mod.rs`, `state.rs`, `message.rs`, `update.rs`, `view.rs`, `tests.rs`), lacks a hierarchical overview index with live configuration summaries, and cannot leverage stack-based back-navigation through `settings_stack`. Modularizing App Settings into an overview index and focused subpages completes the architectural standardization across all Constellation settings domains.

## What Changes

- **Modularize `src/settings/app/` Directory**:
  - Replace the monolithic `src/settings/app.rs` with `src/settings/app/` containing `mod.rs`, `state.rs`, `message.rs`, `update.rs`, `view.rs`, and `tests.rs`, matching the standard MVU pattern used across `user`, `room`, and `space` settings.
- **App Settings Overview Index**:
  - Replace the single-scroll layout with a clean overview index using `category_row` from `src/settings/widgets/`:
    - `AppAppearance`: Visual presentation and message formatting (compact mode, markdown rendering, sync indicator, video autoplay, hide threaded messages) with live status summaries.
    - `AppNotifications`: Typing indicators and session error diagnostic log with individual and clear-all dismiss actions.
    - `AppMaintenance`: Media cache management and direct drill-down to keyboard shortcuts.
- **Centralized Settings Navigation Stack Integration**:
  - Add `AppAppearance` and `AppNotifications` variants to `SettingsPanel` (reusing existing `Shortcuts`).
  - Add `Message::OpenPanel(crate::SettingsPanel)` to `src/settings/app/message.rs` and route through `src/constellation/handlers/update.rs`.
  - Update `handle_open_settings` in `src/constellation/handlers/rooms.rs` so that drilling into any app subpage pushes `[SettingsPanel::App, subpage]` onto `settings_stack`, enabling automatic `< Back` navigation in `ContextDrawer::actions` and via the Escape key.
  - Wire context drawer titles and subpage view dispatch in `src/constellation/app.rs`.
- **Subpage View Decomposition**:
  - Decompose views into dedicated subpage functions: `view_overview()`, `view_appearance_page()`, `view_notifications_page()`, and `view_maintenance_page()`.
  - Retain `view(&self)` forwarding to `view_overview()` for backward compatibility.
- **Localization**:
  - Add Fluent strings for App subpage titles and overview category summaries in `res/i18n/en/constellation.ftl`.

## Capabilities

### New Capabilities
- `app-settings`: Multi-page hierarchical settings interface, modular subpages for appearance, notifications/diagnostics, and cache maintenance, and navigation stack history for app configuration.

### Modified Capabilities
None (brownfield project; no existing specs under `openspec/specs/app-settings/`).

## Impact

- **Affected Code**: `src/settings/app.rs` replaced by `src/settings/app/{mod, state, message, update, view, tests}.rs`, `src/settings/mod.rs`, `src/constellation/{mod, app, handlers/update, handlers/rooms}.rs`.
- **UI / UX**: Context drawer presents a compact overview index for application settings instead of a monolithic scroll; subpage navigation is consistent via `< Back` and the Escape key.
- **Dependencies**: Leverages existing libcosmic widgets and `src/settings/widgets/category_row`.
- **Localization**: Adds Fluent string IDs for app subpage titles and overview summaries in `res/i18n/en/constellation.ftl`.
