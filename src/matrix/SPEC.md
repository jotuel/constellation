---
id: matrix
type: module-design
status: draft
title: Matrix Engine Module
parent: constellation-architecture
---

# Matrix Engine Module

## Purpose

The `matrix` module encapsulates all Matrix protocol communication, encryption operations, background synchronization, and local persistence. It acts as the interface between Constellation and `matrix-rust-sdk`.

## Responsibility

- **Client Session & Sync (`mod.rs`)**: Owns the `matrix_sdk::Client`, `SyncService`, and `RoomListService`. Manages sliding sync streams, dynamic room entry controllers, and session serialization.
- **Authentication & Keyring (`auth.rs`)**: Manages session restoration, password login, OIDC client registration and token refresh, and QR code login with cryptographic secret exchange. Persists credentials securely via `oo7` Secret Service keyring.
- **Timeline & Event State (`timeline_state.rs`)**: Subscribes to room timeline diffs, manages focused thread timelines, handles back-pagination, and formats timeline events.
- **Messaging & Media (`messaging.rs`, `image_pack.rs`)**: Dispatches room messages, edits, redactions, emoji reactions, typing indicators, media uploads, and MSC2545 custom sticker/emoji packs.
- **Space & Room Hierarchy (`spaces.rs`, `sync_room_data.rs`)**: Resolves space parent/child relations, room ordering, power level permissions, member moderation, and canonical aliases.
- **Search Services (`search.rs`)**: Executes local message searches, room-scoped message queries, and public room directory queries.
- **VoIP Calls (`calls.rs`)**: Coordinates LiveKit call signaling under MSC4143.

## Boundaries

- **Allowed Dependencies**: `utils`, `matrix-sdk`, `matrix-sdk-ui`, `livekit`, `oo7`, `ruma`.
- **Forbidden Reaches**: Must not import `view` or `settings`. Must not construct UI widgets or iced views.
- **Invariants**:
  - Store Keyring Coherence: If the keyring key is reset or invalid, lazy cache and search indices must be cleared to avoid store MAC decryption failures.
  - Secret Isolation: Authentication tokens and encryption passphrases must remain wrapped in `SecretString` or zeroized structures and never logged.
