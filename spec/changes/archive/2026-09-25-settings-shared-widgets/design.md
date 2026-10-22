# Design

## Context

See `proposal.md` for motivation. The settings views across `src/settings/user/`, `src/settings/room/`, and `src/settings/space/` currently duplicate list-item category rows, header cards, 64px avatar containers, error banners, and dirty-state save buttons. Additionally, `cosmic::widget::segmented_button::SingleSelectModel` does not implement `Clone`, forcing `user::State` and `room::State` to maintain 50-80 line manual `impl Clone` blocks that copy every field individually.

## Goals / Non-Goals

**Goals:**
- Extract shared settings components into a dedicated `src/settings/widgets/` submodule (`navigation.rs`, `feedback.rs`, `notifications.rs`, `mod.rs`).
- Encapsulate `SingleSelectModel` inside a `NotificationModeSelector` struct that implements `Default` and `Clone`, allowing `user::State` and `room::State` to use `#[derive(Clone)]`.
- Migrate `src/settings/user/`, `src/settings/room/`, and `src/settings/space/` to consume the shared widgets with zero visual or behavioral regressions.
- Provide comprehensive unit tests for widget construction, state transitions, and cloning.

**Non-Goals:**
- Implementing subpages for Space Settings or App Settings (these are subsequent tracks that will build on top of these shared widgets).
- Modifying general-purpose app widgets in `src/utils/widget.rs`.
- Changing matrix-sdk notification mode semantics or API calls.

## Decisions

### 1. Dedicated `src/settings/widgets/` Submodule vs Monolithic File or `utils/`
- **Decision**: Create a submodule under `src/settings/widgets/` partitioned into `navigation.rs`, `feedback.rs`, and `notifications.rs`, re-exported via `src/settings/widgets/mod.rs` and `src/settings/mod.rs`.
- **Rationale**: Keeps settings-specific UI components grouped in their domain while avoiding a bloated single file. Keeps `src/utils/widget.rs` clean and focused on application-wide primitives (e.g., general tooltip helpers).
- **Alternatives Considered**:
  - `src/settings/widgets.rs` single file: Simpler layout, but mixes unrelated concerns (notification segmented controls, navigation rows, error banners) into a single 400+ line file.
  - `src/utils/widget/settings.rs`: Pollutes global utility crate with settings-specific presentation logic.

### 2. Encapsulated `NotificationModeSelector` with Custom `Clone`
- **Decision**: Encapsulate the `mode: Option<RoomNotificationMode>`, `model: SingleSelectModel`, and `entities: [Entity; 3]` into a dedicated `NotificationModeSelector` struct that implements `Default` and `Clone`.
  ```rust
  #[derive(Debug)]
  pub struct NotificationModeSelector {
      pub mode: Option<matrix_sdk::notification_settings::RoomNotificationMode>,
      pub model: cosmic::widget::segmented_button::SingleSelectModel,
      pub entities: [cosmic::widget::segmented_button::Entity; 3],
  }
  ```
- **Rationale**: When `NotificationModeSelector::clone()` is called, it constructs a fresh `SingleSelectModel`, registers the 3 standard entities, and activates the one matching `self.mode`. This encapsulates the uncloneable UI model at the leaf level, immediately enabling `#[derive(Clone)]` on `user::State` and `room::State` and deleting ~130 lines of error-prone manual cloning.
- **Alternatives Considered**:
  - Storing only `Option<RoomNotificationMode>` in state and recreating `SingleSelectModel` on every render: Creates avoidable heap allocations on every view frame, violating project performance guidelines.
  - Retaining manual `impl Clone for State`: Fragile; requires updating the manual clone implementation whenever any field is added to settings states.

### 3. Generic Message Dispatch for View Builders
- **Decision**: Widget builder functions accept generic `M: Clone + 'static` message types:
  - `category_row<'a, M>(title, summary, on_press: M) -> Element<'a, M>`
  - `header_card<'a, M>(avatar, title, subtitle, on_press: M) -> Element<'a, M>`
  - `save_button<'a, M>(is_saving, has_changes, on_save: M) -> Element<'a, M>`
  - `view_error<'a, M>(error, on_dismiss: M) -> Option<Element<'a, M>>`
- **Rationale**: Allows `user::Message`, `room::Message`, and `space::Message` to be passed directly without requiring wrapping or conversion closures.
- **Alternatives Considered**:
  - Unifying messages under a shared enum: Impractical and violates MVU module isolation in Elm architecture.

## Risks / Trade-offs

- **[Risk] Entity ID desynchronization on selector clone** → *Mitigation*: The `Clone` implementation for `NotificationModeSelector` rebuilds both the model and the `entities` array simultaneously via `Self::new(self.mode)`, ensuring entity references always match the active model instance.
- **[Risk] Styling drift across settings panels** → *Mitigation*: All navigation elements use `cosmic::theme::Button::ListItem` with `cosmic::theme::active().cosmic().corner_radii.radius_m`, guaranteeing identical corner radii and hover states across light and dark system themes.
- **[Risk] Accidental regressions in existing tests** → *Mitigation*: Run existing unit and view smoke tests in `src/settings/user/tests.rs` and `src/settings/room/tests.rs` after migration, in addition to new tests in `src/settings/widgets/tests.rs`.

## Migration Plan

1. **Scaffold Submodule**: Add `src/settings/widgets/{mod, navigation, feedback, notifications, tests}.rs` and export `pub mod widgets;` in `src/settings/mod.rs`.
2. **Implement Components & Tests**: Build the widgets and verify behavior with unit tests in `src/settings/widgets/tests.rs`.
3. **Migrate User Settings**: Update `src/settings/user/` to replace `create_notification_mode_model`, manual `Clone`, and inline view closures.
4. **Migrate Room Settings**: Update `src/settings/room/` to replace `create_notification_mode_model`, manual `Clone`, and inline view closures.
5. **Migrate Space Settings**: Update `src/settings/space/view.rs` to replace duplicate `view_error`, `view_save_button`, and avatar placeholder.
6. **Verification**: Run `cargo test` across all targets to verify regression-free migration.
