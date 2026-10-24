---
id: settings
type: module-design
status: draft
title: Settings Subsystem Module
parent: constellation-architecture
---

# Settings Subsystem Module

## Purpose

The `settings` module provides a modular, multi-page settings architecture hosted inside the COSMIC context drawer. Each configuration category operates as an isolated Sub-MVU (Model-View-Update) component with its own message routing, navigation stack history, and persistence handler.

## Responsibility

- **Sub-MVU Orchestration (`mod.rs`)**: Hosts settings state inside `Constellation` and routes category-specific actions to their respective update loops.
- **User Settings (`user/`)**: Manages user profile identity, connected identifiers, default notification policies, privacy/ignored users, active cryptographic sessions/cross-signing, password change, and sticker packs (governed by submodule `user-settings`).
- **Room Settings (`room/`)**: Manages room profile information, topic, canonical/alternative aliases, room-specific notification modes, access rules/encryption, roles and permissions power levels, member moderation, custom room image packs, and room lifecycle actions (governed by submodule `room-settings`).
- **Space Settings (`space/`)**: Manages space profile identity, discoverability, and child room/subspace hierarchy (governed by submodule `space-settings`).
- **App Settings (`app/`)**: Manages appearance toggles (compact mode, markdown rendering, sync indicator), diagnostic session error logs, cache clearing, and shortcut configuration (governed by submodule `app-settings`).
- **Shared Settings Widgets (`widgets/`)**: Provides standardized, high-contrast components for overview navigation rows, header profile cards, uniform avatar containers, save buttons, error alerts, and 3-way notification mode segmented selectors (governed by submodule `settings-shared-widgets`).
- **Keyboard Shortcuts (`shortcuts.rs`)**: Provides the UI and key capture mechanism for configuring application shortcuts.
- **Configuration Persistence (`config.rs`)**: Serializes and restores settings using `cosmic-config`.

## Boundaries

- **Allowed Dependencies**: `matrix`, `utils`, `libcosmic`, `cosmic-config`.
- **Forbidden Reaches**: Settings subpages must not directly manipulate root application window state outside of drawer navigation events.
- **Invariants**:
  - Navigation Stack Consistency: Category views maintain stack-based history enabling drill-down and pop back via the drawer header action or the Escape key.
  - Visual Contrast & Theme Compliance: All settings widgets and navigation rows must use high-contrast list item styling that remains readable in both light and dark modes.
  - Subpage Header Deduplication: Individual settings subpages must not duplicate the context drawer title with redundant section headers.
