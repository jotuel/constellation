# Tasks

## 1. Custom List Item Button Styling

- [x] 1.1 Implement `list_item_button_style(radii: [f32; 4]) -> cosmic::theme::Button` in `src/settings/widgets/navigation.rs` using `cosmic::theme::Button::Custom` with transparent active background, native hover/press colors, and unconstrained (`None`) text and icon colors.
- [x] 1.2 Update `category_row` in `src/settings/widgets/navigation.rs` to apply `list_item_button_style(radii)` instead of `cosmic::theme::Button::ListItem(radii)`.
- [x] 1.3 Update `header_card` in `src/settings/widgets/navigation.rs` to apply `list_item_button_style(radii)` instead of `cosmic::theme::Button::ListItem(radii)`.

## 2. Verification and Testing

- [x] 2.1 Run settings widget unit tests in `src/settings/widgets/tests.rs` and update assertions for the custom button class if needed.
- [x] 2.2 Perform a smoke check on the running application or test harness to verify that settings overview rows display dark high-contrast text in light theme and legible bright text in dark theme.
