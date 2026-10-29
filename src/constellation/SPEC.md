---
id: constellation
type: module-design
status: draft
title: Constellation Core Module
parent: constellation-architecture
---

# Constellation Core Module

## Purpose

The `constellation` module serves as the primary Model-View-Update (Elm) coordinator and state hub for the application. It implements `cosmic::Application`, orchestrates global state transitions, processes typed application messages, subscribes to asynchronous runtime streams, and manages window context pages.

## Responsibility

- **Application Lifecycle & Wiring (`app.rs`)**: Implements `cosmic::Application` lifecycle methods (`init`, `update`, `view`, `subscription`, `context_page`). Manages window titles, header actions, and drawer toggles.
- **State & Message Definitions (`mod.rs`, `state.rs`)**: Defines the central `Constellation` application struct, the comprehensive `Message` enum, active navigation tabs (`Tab::Room`, `Tab::Thread`, `Tab::Search`), and mutually exclusive authentication states (`AuthFlow`).
- **Subscription Orchestration (`subscriptions.rs`)**: Bridges asynchronous Matrix event streams, room list vector diffs, typing status notifications, DBus IPC signals, call state updates, and global keyboard shortcuts into the iced event loop.
- **Keybinding & Navigation Dispatch (`keybind.rs`, `scroll.rs`)**: Evaluates desktop shortcut configurations, drives tab navigation, and maintains anchor-based timeline scroll positions and measured event-row jumps across layout reflows.
- **Message Handlers (`handlers/`)**: Dispatches async task completions (file downloads, message sending results, room join confirmations, permalink routing, and event-context timeline transitions).

## Decisions & Contracts

- **Event Permalink & Jump-to-Message Flow**:
  - Event permalinks parsed by `utils` (`PermalinkTarget::Event`) validate joined-room membership through `matrix` before switching views. Unjoined targets surface an error toast while keeping the current view rather than silently joining (joining requires explicit `PermalinkTarget::Join`), and cold-start links arriving while session restoration is still running defer any sign-in prompt until restoration settles.
  - Joined event targets and in-room message jumps (`JumpToMessageOrLoadContext`) switch to the room's main chat view (exiting any active thread or search tab while keeping open side drawers such as Pinned Messages in place); cross-room opens record `pending_event_focus` *after* room selection resets room state, suppressing default initial/unread bottom-snapping across `TimelineReset` and `TimelineInitFinished`.
  - When the target event is absent from the loaded timeline window, `LoadEventContext` builds and caches the `TimelineFocus::Event` timeline in `matrix` *before* activating `active_event_focus`, ensuring failed fetches surface a not-found toast without tearing down the live timeline and guarding completions against intervening room switches.
  - `JumpToMessage` always re-measures live widget-row bounds (`tl|e:<event_id>`) rather than using item-index ratios or cached pre-drawer coordinates, syncing observed viewport/content dimensions so side-drawer width changes and highlight padding do not trigger a competing layout-resize restore, retrying across initial render frames if measurement precedes layout, and recording `highlighted_event` (rendered by `view`) until room switch, `ReturnToLive`, logout, or a subsequent jump.

## Boundaries

- **Allowed Dependencies**: `matrix`, `settings`, `view`, `utils`, `libcosmic`, `cosmic-config`.
- **Forbidden Reaches**: Does not directly perform raw network I/O or manipulate database stores; all homeserver interactions delegate through `matrix`.
- **Invariants**:
  - Mutual exclusion in auth flows: `AuthFlow` ensures password login, OIDC login, and QR code login cannot run concurrently.
  - Context drawer isolation: Settings pages and verification cards are rendered exclusively via `context_page`.
