# Proposal

## Why

Constellation currently lacks an in-app "About" page, making it difficult for users to inspect the application version, review license terms, identify contributors, or access official links such as the repository, bug tracker, and sponsor pages. `libcosmic` provides a first-class, standard `about` feature and widget designed specifically for COSMIC desktop applications, which Constellation has not yet enabled in its dependency configuration.

## What Changes

- Enable the `about` feature for `libcosmic` in `Cargo.toml`.
- Define application metadata for Constellation (version, developer, license, repository, issue tracker, donation links) matching `res/fi.joonastuomi.Constellation.metainfo.xml`.
- Add an "About" option to the header user menu (`MenuAct::About`).
- Add an "About" category row to the App Settings overview index.
- Add `SettingsPanel::About` to the settings navigation stack, presenting the native `cosmic::app::context_drawer::about` context drawer with working external URL dispatch and stack back-navigation.
- Add necessary Fluent i18n keys for the About menu entry and settings category.

## Capabilities

### New Capabilities
- `about-page`: Standard application metadata, versioning, developer credits, license information, and project links presented via libcosmic's native about widget and context drawer integration.

### Modified Capabilities
- `app-settings`: Update the root App Settings overview index to include an About category entry linking to the About subpage within the navigation stack.

## Impact

- **Dependencies**: Adds `"about"` feature to `libcosmic` in `Cargo.toml`.
- **Application State & Navigation**: Extends `SettingsPanel` with `About` variant and `MenuAct` with `About` variant in `src/constellation/mod.rs`.
- **Context Drawer**: Integrates `cosmic::app::context_drawer::about` in `src/constellation/app.rs` with URL opener handler and navigation controls.
- **UI & i18n**: Updates App Settings overview in `src/settings/app/view.rs`, header menu in `src/constellation/app.rs`, and adds localization strings in `res/i18n/en/constellation.ftl`.
