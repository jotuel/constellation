# Proposal

## Why

Settings subsection overview buttons (`category_row` and `header_card`) render title, summary, and chevron text in low-contrast near-white/light-grey in light themes due to `Button::ListItem` forcing `cosmic.list_button.on`. This gives active, functional navigation buttons a disabled, unreadable appearance.

## What Changes

- Update `category_row` and `header_card` in `src/settings/widgets/navigation.rs` to use a custom button style (`cosmic::theme::Button::Custom`) instead of `cosmic::theme::Button::ListItem`.
- Ensure the custom button style preserves transparent active background, theme list-item corner radii, and native hover/pressed feedback while leaving text and icon colors unconstrained so they inherit high-contrast contextual theme colors in both light and dark modes.
- Update `src/settings/widgets/tests.rs` to verify that category rows and header cards construct with the updated custom styling and remain interactive.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `settings-shared-widgets`: Ensure overview category navigation rows and overview header cards maintain readable text contrast and active interactive styling across light and dark system themes.
