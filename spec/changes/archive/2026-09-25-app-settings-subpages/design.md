# Design

## Context

See `proposal.md` for motivation. While User Settings, Room Settings, and Space Settings have all transitioned to multi-file directory structures with overview indexes, live summaries, and stack-based back navigation, App Settings remains a monolithic 300+ line single file (`src/settings/app.rs`) presenting all configuration options in a continuous scroll. This design standardizes App Settings to match the rest of Constellation's settings architecture.

## Goals / Non-Goals

**Goals:**
- Replace `src/settings/app.rs` with a modular `src/settings/app/` directory containing `mod.rs`, `state.rs`, `message.rs`, `update.rs`, `view.rs`, and `tests.rs`.
- Provide an overview index displaying category rows for Appearance & Display, Notifications & Diagnostics, and Maintenance & Shortcuts using `src/settings/widgets/category_row` with live status summaries.
- Add `AppAppearance`, `AppNotifications`, and `AppMaintenance` to `SettingsPanel`.
- Integrate App Settings subpages into `settings_stack` so that `< Back` in `ContextDrawer::actions` and the `Escape` shortcut pop back to the App Settings overview.
- Maintain existing config auto-persistence on setting changes (`AppSettingChanged`).

**Non-Goals:**
- Changing underlying schema or serialization in `src/settings/config.rs`.
- Modifying keyboard shortcuts implementation in `src/settings/shortcuts.rs`.

## Decisions

### 1. Module Structure Matching Other Settings Domains
- **Decision**: Decompose `src/settings/app.rs` into:
  - `src/settings/app/mod.rs`: Module exports and public re-exports (`Message`, `SessionError`, `State`).
  - `src/settings/app/state.rs`: `SessionError`, `State`, `from_config()`, and `push_session_error()`.
  - `src/settings/app/message.rs`: `Message` enum, including `OpenPanel(crate::SettingsPanel)`.
  - `src/settings/app/update.rs`: `State::update()`.
  - `src/settings/app/view.rs`: `view_overview()`, `view_appearance_page()`, `view_notifications_page()`, `view_maintenance_page()`, and `view()`.
  - `src/settings/app/tests.rs`: Unit tests for state updates and view smoke tests.
- **Rationale**: Follows the exact structure used in `user/`, `room/`, and `space/`, making codebase navigation predictable.

### 2. `SettingsPanel` Additions and Navigation Routing
- **Decision**: Add `AppAppearance`, `AppNotifications`, and `AppMaintenance` variants to `SettingsPanel` in `src/constellation/mod.rs`. Re-use existing `SettingsPanel::Shortcuts`.
- **Decision**: In `src/constellation/handlers/rooms.rs`:
  ```rust
  SettingsPanel::AppAppearance
  | SettingsPanel::AppNotifications
  | SettingsPanel::AppMaintenance => {
      if self.settings_stack.first() == Some(&SettingsPanel::App) {
          if self.settings_stack.len() > 1 {
              self.settings_stack.pop();
          }
          self.settings_stack.push(panel.clone());
      } else {
          self.settings_stack = vec![SettingsPanel::App, panel.clone()];
      }
  }
  ```
- **Rationale**: Consistent with the stack manipulation used for room and space subpages, ensuring any direct navigation to an app subpage preserves `SettingsPanel::App` as the stack root.

### 3. Message Routing for Subpages
- **Decision**: Add `Message::OpenPanel(crate::SettingsPanel)` to `src/settings/app/message.rs`. In `src/constellation/handlers/update.rs`:
  ```rust
  Message::AppSettings(crate::settings::app::Message::OpenPanel(panel)) => {
      self.handle_open_settings(panel)
  }
  ```
- **Rationale**: Allows category rows in the App Settings overview to dispatch navigation directly without coupling the settings submodule to parent application state.

### 4. Overview Index and Subpage Views
- **Decision**: Implement the overview index in `src/settings/app/view.rs`:
  - Category row for `AppAppearance`: Live summary listing key active flags (e.g. compact mode, markdown).
  - Category row for `AppNotifications`: Live summary showing typing notification state and session error count.
  - Category row for `AppMaintenance`: Live summary indicating cache and shortcuts.
- **Decision**: Dedicated subpage views:
  - `view_appearance_page(&self)`: Compact mode, markdown, sync indicator, video autoplay, and hide threaded messages.
  - `view_notifications_page(&self)`: Typing notifications toggle and session errors log with dismiss controls.
  - `view_maintenance_page(&self)`: Clear cache button and keyboard shortcuts launcher button.

## Risks / Trade-offs

- **[Risk] Call site disruption during file replacement** → *Mitigation*: Re-export `Message`, `SessionError`, and `State` at `crate::settings::app::*` with identical visibility.
- **[Risk] Shortcuts navigation divergence** → *Mitigation*: The shortcuts button in `view_maintenance_page` continues to dispatch `Message::OpenShortcuts`, pushing `SettingsPanel::Shortcuts` onto the existing stack seamlessly.

## Migration Plan

1. **Add `SettingsPanel` Variants**: Update `src/constellation/mod.rs`.
2. **Decompose `src/settings/app/`**: Create directory and files (`mod.rs`, `state.rs`, `message.rs`, `update.rs`, `view.rs`, `tests.rs`), removing old `src/settings/app.rs`.
3. **Wire Handlers**: Update `handlers/update.rs` for `OpenPanel` and `handlers/rooms.rs` for `settings_stack`.
4. **Wire App View Dispatch**: Map drawer titles and subpage renders in `src/constellation/app.rs`.
5. **Add Fluent Strings**: Update `res/i18n/en/constellation.ftl`.
6. **Testing**: Add unit tests in `src/settings/app/tests.rs` and navigation stack tests in `src/constellation/tests.rs`.
