# Proposal

## Why

Space Settings currently renders its configuration in two disconnected panels: a flat continuous scroll for profile and discovery settings (`SettingsPanel::Space`), and a separate detached panel for room hierarchy management (`SettingsPanel::ManageSpaceRooms`). Unlike User Settings and Room Settings, Space Settings lacks a unified overview index, cannot display live configuration summaries (such as child room counts or discovery access modes), and does not support hierarchical back-navigation through `settings_stack`. Modularizing Space Settings into an Overview index and focused subpages (`SpaceProfile`, `SpaceAccess`, `ManageSpaceRooms`) backed by the centralized settings navigation stack brings architectural consistency across all constellation settings interfaces and significantly improves navigation usability for spaces.

## What Changes

- **Space Settings Overview Index**:
  - **Space Header Card**: A prominent header card displaying the space avatar (via `avatar_box`), space name, and canonical alias or space ID with list-item styling and a drill-down chevron navigating to `SpaceProfile`.
  - **Category Rows with Live Summaries**: Distinct category rows (via `category_row`) with descriptive titles, live status summaries, and drill-down chevrons navigating to:
    - `SpaceProfile`: Avatar upload/change, space name, topic, space ID, and canonical alias management.
    - `SpaceAccess`: Discovery and access rules (public discoverability toggle, invite-only toggle).
    - `ManageSpaceRooms`: Space hierarchy, room filtering, child room ordering, suggested status toggling, child join rules, and adding child rooms/subspaces by ID.
- **Centralized Settings Navigation Stack Integration**:
  - Add `SpaceProfile` and `SpaceAccess` variants to `SettingsPanel`.
  - Add `Message::OpenPanel(crate::SettingsPanel)` to `src/settings/space/message.rs` and route through `src/constellation/handlers/update.rs`.
  - Update `handle_open_settings` in `src/constellation/handlers/rooms.rs` so that drilling into any space subpage pushes `[SettingsPanel::Space, subpage]` onto `settings_stack`, enabling standard `< Back` navigation in `ContextDrawer::actions` and via the Escape key.
  - Wire context drawer titles and subpage view dispatch in `src/constellation/app.rs`.
- **Subpage View Modularization & Shared Widget Integration**:
  - Decompose `src/settings/space/view.rs` into dedicated subpage functions: `view_overview()`, `view_profile_page()`, `view_access_page()`, and `view_manage()`.
  - Standardize error banners (`view_error`) and dirty-checking save buttons (`save_button`) across each subpage where modifications occur.
- **Fluent Localization**:
  - Add localization strings for space subpage titles and overview category summaries in `res/i18n/en/constellation.ftl`.

## Capabilities

### New Capabilities
- `space-settings`: Multi-page hierarchical navigation, overview status summaries, and modular subpages for space profile, discovery/access rules, and room hierarchy management.

### Modified Capabilities
None (brownfield project; no existing specs under `openspec/specs/space-settings/`).

## Impact

- **UI / UX**: Context drawer presents a compact overview index for spaces instead of a single flat scroll; back navigation is seamless via the drawer header and Escape key.
- **Code Architecture**: Modularizes `src/settings/space/view.rs` into dedicated subpages; integrates Space Settings with `settings_stack` alongside User and Room Settings.
- **Dependencies**: Leverages existing libcosmic widgets and the newly extracted `src/settings/widgets/` submodule (`category_row`, `header_card`, `avatar_box`, `view_error`, `save_button`).
- **Localization**: Adds Fluent string IDs for space subpage titles and overview summaries in `res/i18n/en/constellation.ftl`.
