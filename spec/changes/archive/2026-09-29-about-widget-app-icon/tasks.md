# Tasks

## 1. Configure About Widget Application Icon

- [x] 1.1 In `src/view/about.rs`, update `about_info()` to configure the `About` builder with `cosmic::widget::icon::from_svg_bytes(crate::CONSTELLATION_ICON)` instead of `icon::from_name(crate::Constellation::APP_ID)`.

## 2. Test and Validate

- [x] 2.1 In `src/view/about.rs`, update unit tests to assert that `about_info()` populates the icon with an SVG handle (`Data::Svg`) and verify `test_about_widget_renders()` executes without error.
- [x] 2.2 Run unit tests via `cargo test view::about` and project linting via `cargo clippy --all-targets` to verify clean compilation and absence of warnings.
