# Proposal

## Why

In `libcosmic`, when `ContextDrawer::actions` is populated with a `< Back` button, the drawer header automatically splits into two rows: the navigation row (`< Back` and `Close >`) and a second row for the drawer title (`text::title4(title)`). Following recent settings modularization into subpages, settings subpage views retained their original `settings::section().title(...)` calls from when they were sections in monolithic views. Consequently, subpages display identical or near-identical titles stacked directly on top of each other: first as the drawer title in the header, and immediately below as the first section title. Removing redundant section titles on subpages aligns with libcosmic's two-row header design and eliminates visual clutter.

## What Changes

- Remove redundant section titles from single-section settings subpages across Application, Room, Space, and User settings where the section title duplicates the context drawer title.
- On multi-section subpages (such as Room Profile and User Profile), remove the duplicate section title from the primary profile section while retaining descriptive section titles for distinct subsections (such as "Room Aliases" and "Linked Emails & Phone Numbers").
- Preserve distinct section subtitles on complex multi-section pages (such as User Sessions and Space Management) where multiple functional blocks coexist under a broader drawer title.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `app-settings`: Update subpage view requirements to specify that subpages render settings groups without redundant section titles duplicating the drawer title.
- `room-settings`: Update room subpage isolation requirements so single-section subpages and primary profile sections omit redundant section headers.
- `space-settings`: Update space subpage requirements so the profile and access subpages omit redundant section headers duplicating the drawer title.
- `user-settings`: Update user subpage requirements so the primary profile section and single-purpose subpages omit section headers duplicating the drawer title.

## Impact

- Affected files: `src/settings/app/view.rs`, `src/settings/room/view.rs`, `src/settings/space/view.rs`, `src/settings/user/view.rs`.
- UI/UX: Subpages gain vertical space and a cleaner visual hierarchy; toggles and settings rows sit directly beneath the context drawer title without repetitive headings.
- No public Matrix SDK API, data model, or keybinding changes.
