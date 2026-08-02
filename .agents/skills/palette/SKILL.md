---
name: palette
description: UX/UI design checklists, accessibility best practices, tooltips, disabled button states, and semantic widgets for libcosmic.
---

# COSMIC/libcosmic UI & UX Design Checklist

This skill documents design principles, accessibility guidelines, and usability best practices for building native COSMIC Desktop applications using `libcosmic` and `iced`.

## 1. Accessibility & Screen Reader Support
* **Tooltips for Icon-Only Buttons**: Standalone icon-only buttons (like `button::icon` or custom icons) must be wrapped inside `cosmic::widget::tooltip` with a descriptive, localized `text::body` and appropriate `Position` (like `Position::Top` or `Position::Right`).
* **No Actionless Buttons for Decorative Icons**: Do not use `button::icon` for purely static or decorative icons (like medium list icons). It creates a misleading keyboard focus block for screen readers. Use `cosmic::widget::icon::from_name(icon)` instead.
* **Destructive Actions**: Irreversible operations (delete, leave, ban, remove keyword) must use visually distinct destructive styling. To make a destructive icon-only button, use:
  `button::custom(cosmic::widget::icon::from_name("...")).class(cosmic::theme::Button::Destructive)`.

## 2. Interactive States & Visual Feedback
* **Disabled States for Primary Actions**: Always conditionally disable primary action buttons (like 'Send' or 'Create') and block form submission when the required inputs are empty or consist only of whitespace (`.trim().is_empty()`).
* **Tooltips for Disabled Actions**: When disabling a primary action button, wrap it in a `cosmic::widget::tooltip` explaining the missing requirement (e.g. "Enter an email address to add" or "Make changes to save").
* **Visual Feedback for Async Operations**: When adding async action triggers, disable the button and modify its text (e.g. showing "Loading...") or show a spinner to reflect the busy state, preventing double-clicks.
* **Density in Compact UI Bars**: Text-based action buttons like "Cancel" or "Close Thread" in compact spaces consume too much space. Prefer semantic `button::icon` variants (`window-close-symbolic`) with localized tooltips.

## 3. Localization & Copy
* **Zero Hardcoded Strings**: Never hardcode user-facing strings directly in UI code. Always use `crate::fl!("key-name")` and add the translation key/value in `res/i18n/en/cosmic_ext_constellation.ftl`. This applies to:
  - Form labels and placeholder text
  - Button text and tooltips
  - Status messages and fallbacks (like 'Unknown Room' or '✅ Verified')
  - Empty states

## 4. Layout & Visual Polish
* **Helpful Empty States**: An empty area (like when no room is selected or no matches are found in search) should not be blank text. Wrap the empty state in a `Column` containing a relevant large icon (e.g., `cosmic::widget::icon::from_name("edit-find-symbolic").size(64)`) positioned above localized helper text.
* **Dense Layout Borders**: When displaying high-density panels side-by-side, separate them using vertical and horizontal pane dividers (`divider::horizontal::default()`, `divider::vertical::default()`) rather than card containers, shadows, or background offsets.
