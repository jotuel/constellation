---
id: view
type: module-design
status: draft
title: View Layer Module
parent: constellation-architecture
---

# View Layer Module

## Purpose

The `view` module provides pure rendering functions that project application state onto `libcosmic` UI widgets. It defines the layout, visual styling, and event binding for the entire desktop presentation layer.

## Responsibility

- **Application Shell (`app.rs`)**: Renders the main window header bar, search bar, active tab indicators, context drawer wrapper, and dialog containers.
- **Room & Space Switcher (`switcher.rs`)**: Renders the left sidebar containing space icons, joined room hierarchies, activity-sorted rooms, unread badges, and room/space creation modals.
- **Chat Timeline & Composer (`chat.rs`)**: Renders timeline event items (messages, replies, threads, reactions, images, audio, video player, location cards), typing indicators, message composer, and the emoji/sticker picker.
- **Login & Onboarding (`login.rs`)**: Renders authentication interfaces for homeserver selection, password login, OIDC browser authorization banners, and interactive QR code onboarding.
- **Interactive Verification (`verification.rs`)**: Displays cryptographic device verification interfaces, including SAS emoji comparisons, number confirmations, and QR code scanning.
- **Diagnostics & Error Display (`error.rs`)**: Renders dismissable error alerts and notification banners.
- **Application About (`about.rs`)**: Renders application metadata, version information, licensing, and links (coordinates with submodule `about-page`).

## Boundaries

- **Allowed Dependencies**: `constellation`, `matrix` (public view model types), `utils`, `libcosmic`.
- **Forbidden Reaches**: No state mutations, no direct file or network I/O, no async runtime spawning.
- **Invariants**:
  - Purity: View functions must remain deterministic renders mapping `&Constellation` to `Element<'_, Message>`.
  - Semantic Color Resolution: Colors must resolve through `theme.palette` to maintain consistency across system themes.
  - Mandatory Localization: All text elements must resolve via Fluent `fl!` keys.
