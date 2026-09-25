# Proposal

## Why

Recent refactorings for User Settings and Room Settings independently created nearly identical UI patterns for overview category rows, header cards, avatar boxes, error banners, dirty-checking save buttons, and notification mode segmented selectors. This has resulted in duplicated closures and widgets across `src/settings/user/`, `src/settings/room/`, and `src/settings/space/`, as well as 130+ lines of brittle manual `impl Clone` boilerplate across state structs caused by uncloneable widget models. Extracting these patterns into a dedicated `src/settings/widgets/` submodule eliminates code duplication, restores `#[derive(Clone)]` on settings states, and establishes standardized UI primitives before refactoring Space Settings and App Settings.

## What Changes

- **Create `src/settings/widgets/` Submodule**:
  - `navigation.rs`:
    - `category_row`: Standardized list-item button row displaying title, live summary, and a trailing `go-next-symbolic` chevron with theme corner radius styling.
    - `header_card`: Interactive overview card with avatar element, title3, caption subtitle, and navigation chevron.
    - `avatar_box`: 64x64 fixed image or container placeholder with centered fallback text.
  - `feedback.rs`:
    - `view_error`: Helper returning a dismissable `settings::section` error banner when an error is present.
    - `save_button`: Standardized save button handling in-progress saving state, dirty modification state, and disabled tooltips via `crate::utils::widget::tooltip_button`.
  - `notifications.rs`:
    - `NotificationModeSelector`: Encapsulated state and view for 3-way room/user notification mode selection (`All Messages`, `Mentions and Keywords Only`, `Mute`) using `cosmic::widget::segmented_control::horizontal`. Implements `Clone` (reconstructing model state) and `Default`, eliminating manual `impl Clone` on settings states.
  - `mod.rs`: Clean re-exports of all public settings widgets.
  - `tests.rs`: Unit tests for widget builders, dirty-state button states, and `NotificationModeSelector` synchronization and cloning.
- **Migrate Existing Settings Modules**:
  - `src/settings/user/`: Replace inline closures, manual profile cards, duplicate `create_notification_mode_model`, and manual `impl Clone for State` with `NotificationModeSelector` and shared widgets.
  - `src/settings/room/`: Replace duplicate closures, header cards, duplicate notification models, and manual `impl Clone for State` with shared widgets.
  - `src/settings/space/`: Replace duplicate `view_error`, `view_save_button`, and avatar placeholder logic with shared widgets.

## Capabilities

### New Capabilities
- `settings-shared-widgets`: Shared UI components and state selectors for COSMIC settings panels, including overview navigation items, feedback and save controls, and notification mode segmented selectors.

### Modified Capabilities
None (existing spec-level user-settings and room-settings behaviors are preserved).

## Impact

- **Affected Code**: `src/settings/widgets/*`, `src/settings/mod.rs`, `src/settings/user/{state, view, update}.rs`, `src/settings/room/{state, view, update}.rs`, `src/settings/space/view.rs`.
- **APIs**: Introduces `crate::settings::widgets::{category_row, header_card, avatar_box, view_error, save_button, NotificationModeSelector}`.
- **Dependencies**: Uses existing `cosmic` and `libcosmic` widgets (`segmented_control`, `settings::section`, `button`, `text`, `image`). No new external crates required.
- **Performance & Maintainability**: Removes ~150 lines of boilerplate manual `Clone` implementations and deduplicates ~250 lines of UI view closures across settings modules.
