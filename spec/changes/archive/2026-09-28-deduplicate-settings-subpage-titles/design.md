# Design

## Context

See `proposal.md` for motivation. Constellation manages contextual panels using `cosmic::app::context_drawer::ContextDrawer` in `src/constellation/app.rs`. When `settings_stack.len() > 1`, Constellation attaches a `< Back` button via `drawer.actions(...)`. In `libcosmic`, providing `actions` triggers a two-row header structure: Row 1 holds navigation actions (`< Back` and `Close >`), and Row 2 renders the drawer title (`text::title4(title)`).

Following the modularization of Application, Room, Space, and User settings into subpages, each subpage view was extracted from earlier monolithic views where sections were preceded by `settings::section().title(...)`. In subpages, this results in the same title appearing in Row 2 (as the drawer title) and Row 3 (as the section title).

## Goals / Non-Goals

**Goals:**
- Eliminate duplicated and redundant section titles on settings subpages.
- Establish a consistent visual hierarchy across Application, Room, Space, and User settings subpages.
- Preserve distinct subsection headings on pages containing multiple heterogeneous settings categories.

**Non-Goals:**
- Modifying `libcosmic`'s `ContextDrawer` widget or attempting to pack the subpage title into Row 1 next to the back button (Approach B maintains standard upstream libcosmic drawer behavior).
- Changing root overview index pages or their category row definitions.
- Restructuring internal state management or message dispatch for settings.

## Decisions

### 1. Adopt Upstream Two-Row Header Design (Approach B)

**Decision:** Maintain `libcosmic`'s standard two-row layout for context drawers with actions, and resolve duplication by removing redundant section titles in subpage views rather than forcing the title into the actions row.

**Rationale:**
- Upstream libcosmic explicitly moved drawer titles to a second row when actions are present (commits `b6c6d1c` and `37ae722`) to accommodate long translated titles without crowding or wrapping against `< Back` and `Close >` buttons on narrow drawers (~360px).
- Leaving drawer title rendering to `ContextDrawer::title(...)` keeps header layout native and consistent with other COSMIC applications.

**Alternatives considered:**
- *Pack title into `drawer.actions(...)`*: Packing `< Back` and `title4` in a single horizontal row would eliminate Row 2, but risks text wrapping or truncation on narrow drawers with longer localized strings.

### 2. Omit `.title(...)` on Single-Section Subpages

**Decision:** On subpages that represent a single configuration topic, instantiate `settings::section()` without calling `.title(...)`.

**Rationale:**
- In `libcosmic::widget::settings::Section`, `header` defaults to `None`. When `.title(...)` is omitted, no heading element is rendered, and settings items render immediately beneath the drawer title.
- Applies to:
  - App Appearance (`view_appearance_page` in `src/settings/app/view.rs`)
  - App Maintenance (`view_maintenance_page` in `src/settings/app/view.rs`)
  - Room Notifications (`view_notifications_page` in `src/settings/room/view.rs`)
  - Room Permissions (`view_permissions_page` in `src/settings/room/view.rs`)
  - Room Packs (`view_packs_page` in `src/settings/room/view.rs`)
  - Space Profile (`view_profile_page` in `src/settings/space/view.rs`)
  - Space Access (`view_access_page` in `src/settings/space/view.rs`)
  - User Packs (`view_packs_page` in `src/settings/user/view.rs`)

### 3. Remove Title from Primary Section on Multi-Section Profile Pages

**Decision:** In `RoomProfile` and `UserProfile`, remove the redundant section title from the primary avatar/name section (`view_profile()`), while keeping descriptive section titles for secondary sections.

**Rationale:**
- In `RoomProfile`, the drawer title is "Room Profile". Having Section 1 also titled "Room Profile" is redundant. Removing Section 1's title allows the avatar and name inputs to sit cleanly under the drawer title, while Section 2 retains its distinct title "Room Aliases" (`room-aliases`).
- In `UserProfile`, the drawer title is "Profile & Identity". Section 1 contains avatar and display name; removing the redundant "Profile" title leaves avatar and name under the drawer title, while Section 2 retains its distinct title "Emails & Phone Numbers" (`emails-and-phone-numbers`).

### 4. Retain Distinct Section Headings on Multi-Category Pages

**Decision:** On subpages with multiple distinct functional areas under a broader drawer title, retain section headers that categorize each area:
- `AppNotifications`: Retain session error log controls under the notifications toggle.
- `UserNotifications`: Retain "Default Notification Settings" and "Keyword Notifications".
- `UserPrivacy`: Retain "Privacy & Preferences" and "Ignored Users".
- `UserSessions`: Retain "Devices & Sessions", "Interactive Verification", and "Cross-Signing".
- `UserAccount`: Retain "Change Password" and "Deactivate Account".
- `SpaceManagement`: Retain "Space Hierarchy" and "Add Child Room/Space".

**Rationale:**
- When multiple distinct tasks share a subpage, section headings provide essential navigational structure and visual separation.

## Risks / Trade-offs

- **Visual spacing when section has no header**: In `libcosmic`, `Section` without a header renders `children` directly.
  - *Mitigation*: Verified that `settings::section()` without `.title(...)` adds only list-column padding without empty header gaps or broken borders.
- **Test assertions on view structures**: Existing smoke tests in `tests.rs` instantiate views without inspecting string contents.
  - *Mitigation*: Run `cargo test settings` and verify subpage view rendering across all domains.
