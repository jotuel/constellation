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
- **Keybinding & Navigation Dispatch (`keybind.rs`, `scroll.rs`)**: Evaluates desktop shortcut configurations, handles timeline pagination scroll offsets, and drives tab navigation.
- **Message Handlers (`handlers/`)**: Dispatches async task completions (file downloads, message sending results, room join confirmations).

## Boundaries

- **Allowed Dependencies**: `matrix`, `settings`, `view`, `utils`, `libcosmic`, `cosmic-config`.
- **Forbidden Reaches**: Does not directly perform raw network I/O or manipulate database stores; all homeserver interactions delegate through `matrix`.
- **Invariants**:
  - Mutual exclusion in auth flows: `AuthFlow` ensures password login, OIDC login, and QR code login cannot run concurrently.
  - Context drawer isolation: Settings pages and verification cards are rendered exclusively via `context_page`.
