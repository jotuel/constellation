---
id: utils
type: module-design
status: draft
title: Utility Foundations Module
parent: constellation-architecture
---

# Utility Foundations Module

## Purpose

The `utils` module provides foundational, cross-cutting services and helpers used across the application. It encapsulates desktop protocol bridges, localization infrastructure, text parsing, URL preview generation, and background push notification handling.

## Responsibility

- **Localization Infrastructure (`i18n.rs`)**: Initializes Fluent language packs through `i18n-embed`, hooks into desktop system language preferences, and powers the `fl!` localization macro.
- **DBus Session IPC (`ipc.rs`)**: Implements session bus IPC over `zbus`. Handles single-instance checking and forwards OIDC callbacks or Matrix permalinks from secondary processes to the primary instance.
- **Unified Timeline Items (`item.rs`)**: Wraps Matrix timeline events in `ConstellationItem` abstractions to decouple presentation formatting from raw SDK types.
- **OpenGraph & Media Previews (`og.rs`)**: Scrapes OpenGraph HTML metadata from web links, fetches preview images asynchronously, and caches results to enrich chat timeline links.
- **Permalink Handling (`permalink.rs`)**: Parses Matrix URI formats (`matrix.to`, `matrix:`, and native `fi.joonastuomi.constellation://` schemes) into typed targets (rooms, events, users), normalizing `http://matrix.to` schemes and stripping non-`via` client query hints (such as `client=...`) that `ruma`'s strict parser would reject.
- **Text & Markdown Previews (`preview.rs`, `rich_text.rs`)**: Parses markdown syntax into cached rich text representations for timeline message items and text previews, matching bare `http://`/`https://` URLs ahead of inline emphasis delimiters so Matrix room and base64 event IDs containing `_` or `~` are extracted intact.
- **Search Query Parsing (`search_query.rs`)**: Tokenizes search strings, extracts filter prefixes, and builds structured query filters.
- **UnifiedPush Daemon (`unified_push.rs`)**: Handles push gateway registrations, endpoint preferences, and runs the standalone headless push notification daemon on `--notify` invocations.

## Boundaries

- **Allowed Dependencies**: External utility libraries (`zbus`, `reqwest`, `image`, `notify-rust`, `unifiedpush`, `i18n-embed`).
- **Forbidden Reaches**: Must not import `constellation`, `view`, or `settings`. Must remain strictly decoupled from application state and UI logic.
- **Invariants**:
  - Headless Notification Threading: The headless notification handler must execute on a separate OS thread with its own single-threaded Tokio runtime to prevent nested `block_on` panics.
  - Localization Integrity: All Fluent resource lookups must fall back gracefully to the default English locale when a specific translation key is missing.
