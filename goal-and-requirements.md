---
id: constellation-goal
type: goal-and-requirements
status: draft
title: Constellation Goal and Requirements
---

# Constellation Goal and Requirements

## Goal

Provide a native, high-performance Matrix messaging client designed specifically for the System76 COSMIC desktop environment, integrating deeply with COSMIC design tokens, system theming, and desktop protocols.

## Problem

Users on the COSMIC desktop environment lack a native Matrix client that adheres to COSMIC's look, feel, and Wayland architecture. Most existing clients are Electron-based or built with GTK/Qt, which do not blend seamlessly with libcosmic styling, window controls, and system service contracts.

## Audience

COSMIC desktop users and Matrix communities seeking a native, memory-efficient desktop client with robust end-to-end encryption, room and space navigation, and rich media communication.

## Capabilities

- **Matrix Authentication**: Standard password login, OIDC/SSO authorization supporting Matrix Authentication Service (MAS) callbacks, and interactive QR code login with cryptographic secret transfer.
- **End-to-End Encryption (E2EE)**: End-to-end encrypted messaging, interactive device verification (SAS emoji comparison and QR code scanning), and secure credential storage via the system Secret Service keyring (`oo7`).
- **Room & Space Navigation**: Dual-pane navigation with space hierarchies, joined room grouping, room discovery, and room creation.
- **Timeline & Messaging**: Message timeline rendering with markdown and plain text previews, threaded conversations, message editing, redactions, emoji reactions, media uploads, and OpenGraph link previews.
- **Context Drawer Settings**: Hierarchical settings navigation with stack-based back navigation across user settings, room settings, space settings, and application configuration.
- **Desktop System Integration**: Fluent (`fl!`) localization, semantic light and dark theming, single-instance execution with DBus argument delegation, custom launch URI handling (`fi.joonastuomi.constellation`), and UnifiedPush notification delivery.
- **Media & Signaling**: In-app video playback via GStreamer (optional `video-player` feature), image viewing, custom sticker/emoji packs (MSC2545), and LiveKit call signaling (MSC4143).

## Non-Goals

- **Multi-Toolkit Desktop Parity**: Constellation does not target GNOME/GTK, KDE/Qt, or cross-platform operating systems; it is built intentionally and exclusively for the COSMIC desktop using `libcosmic`.
- **Multi-Protocol Bridging**: Constellation does not implement native IRC, XMPP, or third-party proprietary chat protocols; all communications route through the Matrix protocol.
- **Built-in Homeserver**: Constellation does not embed or manage homeserver processes; it functions strictly as a Matrix client connecting to external Matrix homeservers.

## Principles

- **Model-View-Update (Elm Architecture)**: Keep state transitions (`update`) and UI renders (`view`) pure and isolated; asynchronous tasks communicate strictly via typed messages.
- **Semantic Theming**: Never hardcode colors; UI components must use `theme.palette` tokens to render correctly across system light and dark modes.
- **Mandatory i18n Localization**: Every user-facing string must be declared in Fluent resource files and rendered via `fl!`.
- **Secret Hygiene**: Sensitive credentials, access tokens, and encryption secrets must be stored securely using Secret Service and never printed to diagnostic logs.
