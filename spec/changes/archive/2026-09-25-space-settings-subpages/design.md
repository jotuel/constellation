# Design

## Context

See `proposal.md` for motivation. Space Settings currently renders configuration in two disconnected panels (`SettingsPanel::Space` and `SettingsPanel::ManageSpaceRooms`), whereas User Settings and Room Settings provide unified overview indexes with live summaries, modular subpages, and back-navigation history managed by `settings_stack`. This design brings Space Settings into the same architectural pattern.

## Goals / Non-Goals

**Goals:**
- Provide a clean overview index for Space Settings featuring a space header card (avatar, name, alias/ID) and category rows with live status summaries.
- Modularize space settings into three focused subpages: `SpaceProfile`, `SpaceAccess`, and `ManageSpaceRooms`.
- Integrate all space subpages into the centralized `settings_stack` so that `< Back` button in `ContextDrawer::actions` and the `Escape` shortcut pop back to the space overview.
- Leverage the shared UI primitives from `src/settings/widgets/` (`header_card`, `category_row`, `avatar_box`, `view_error`, `save_button`).
- Maintain search query synchronization for filtering child rooms when drilling into `ManageSpaceRooms`.

**Non-Goals:**
- Introducing space permission power levels or member moderation tables in this change (can be added as follow-up subpages later).
- Altering the underlying Matrix client space hierarchy fetch or child addition logic.

## Decisions

### 1. `SettingsPanel` Variants and Navigation Structure
- **Decision**: Add `SpaceProfile` and `SpaceAccess` to `SettingsPanel` in `src/constellation/mod.rs`. Retain `ManageSpaceRooms` as the subpage for child rooms and hierarchy management.
- **Rationale**: Reuses the existing `ManageSpaceRooms` panel variant while cleanly partitioning profile and discovery/access settings into distinct subpages.
- **Alternatives Considered**:
  - Replacing `ManageSpaceRooms` with `SpaceRooms`: Unnecessary churn across call sites; aliasing or keeping `ManageSpaceRooms` preserves backward compatibility.

### 2. Message Routing via `OpenPanel`
- **Decision**: Add `Message::OpenPanel(crate::SettingsPanel)` to `src/settings/space/message.rs`.
- **Rationale**: Matches the exact convention used by `src/settings/user/` and `src/settings/room/`. In `src/constellation/handlers/update.rs`, `Message::SpaceSettings(crate::settings::space::Message::OpenPanel(panel))` routes directly to `self.handle_open_settings(panel)`.
- **Alternatives Considered**:
  - Emitting `crate::Message::OpenSettings` from within `space::State::update`: Creates circular dependencies between `settings::space` and the root `constellation` crate.

### 3. Stack Navigation in `rooms.rs`
- **Decision**: Update `handle_open_settings` in `src/constellation/handlers/rooms.rs`:
  ```rust
  SettingsPanel::ManageSpaceRooms
  | SettingsPanel::SpaceProfile
  | SettingsPanel::SpaceAccess => {
      if self.settings_stack.first() == Some(&SettingsPanel::Space) {
          if self.settings_stack.len() > 1 {
              self.settings_stack.pop();
          }
          self.settings_stack.push(panel.clone());
      } else {
          self.settings_stack = vec![SettingsPanel::Space, panel.clone()];
      }
  }
  ```
- **Rationale**: Guarantees that opening any space subpage from a menu, shortcut, or the overview always roots the stack at `SettingsPanel::Space`, enabling single-step `< Back` navigation.

### 4. Search Filter Synchronization
- **Decision**: When entering `SettingsPanel::ManageSpaceRooms` or typing in search while that panel is active, synchronize `self.search_query` with `self.space_settings.child_filter`.
- **Rationale**: Preserves the existing live room filtering capability in the hierarchy view while keeping the top-level space overview clean.

### 5. View Decomposition in `src/settings/space/view.rs`
- **Decision**: Decompose `src/settings/space/view.rs` into:
  - `view_overview(&self) -> Element<'_, Message>`
  - `view_profile_page(&self) -> Element<'_, Message>`
  - `view_access_page(&self) -> Element<'_, Message>`
  - `view_manage(&self) -> Element<'_, Message>`
  - `view(&self)` forwards to `self.view_overview()` for backward compatibility.

## Risks / Trade-offs

- **[Risk] Title and back button desynchronization** → *Mitigation*: Map all three subpages (`SpaceProfile`, `SpaceAccess`, `ManageSpaceRooms`) in `src/constellation/app.rs` for both context drawer title and view rendering.
- **[Risk] Unsaved changes when navigating between subpages** → *Mitigation*: Each subpage maintains independent dirty-state tracking with `save_button`, preventing state loss across subpage navigations.

## Migration Plan

1. **Enum & Message Extensions**: Add `SpaceProfile` and `SpaceAccess` to `SettingsPanel`; add `OpenPanel(SettingsPanel)` to `src/settings/space/message.rs`.
2. **Handlers & Routing**: Wire `OpenPanel` in `handlers/update.rs`; update `handle_open_settings` in `handlers/rooms.rs` and filter sync in `handlers/search.rs`.
3. **App Wiring**: Update context drawer title and panel view dispatch in `src/constellation/app.rs`.
4. **View Modularization**: Refactor `src/settings/space/view.rs` with `view_overview`, `view_profile_page`, `view_access_page`, and `view_manage`.
5. **Localization**: Add Fluent strings for titles and overview status summaries to `res/i18n/en/constellation.ftl`.
6. **Tests**: Add unit tests in `src/settings/space/tests.rs` and navigation stack tests in `src/constellation/tests.rs`.
