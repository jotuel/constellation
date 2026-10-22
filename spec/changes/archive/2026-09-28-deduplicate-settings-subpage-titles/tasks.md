# Tasks

## 1. Application Settings Subpages

- [x] 1.1 Remove duplicate section titles from `view_appearance_page` and `view_maintenance_page` in `src/settings/app/view.rs`, and verify with `cargo test settings::app`.

## 2. Room Settings Subpages

- [x] 2.1 Remove duplicate section title from `view_profile` in `view_profile_page` while preserving the distinct `view_aliases` title in `src/settings/room/view.rs`.
- [x] 2.2 Remove duplicate section titles from `view_notifications_page`, `view_permissions_page`, and `view_image_packs` in `src/settings/room/view.rs`, and verify with `cargo test settings::room`.

## 3. Space Settings Subpages

- [x] 3.1 Remove duplicate section title from `view_profile` in `view_profile_page` and `view_discovery` in `view_access_page` in `src/settings/space/view.rs`, and verify with `cargo test settings::space`.

## 4. User Settings Subpages

- [x] 4.1 Remove duplicate section title from `view_profile` in `view_profile_page` while preserving the `view_3pids` title, and remove duplicate section title from `view_subscribed_packs` in `src/settings/user/view.rs`, verifying with `cargo test settings::user`.

## 5. Verification

- [x] 5.1 Run full test suite covering all settings modules and navigation stack behavior (`cargo test settings && cargo test constellation::tests::test_user_settings_navigation_stack`) to ensure clean compilation and test execution.
