---
id: constellation-architecture
type: architecture-design
status: draft
title: Constellation System Architecture
parent: constellation-goal
---

# Constellation System Architecture

## Drivers

- **COSMIC Desktop Integration**: Deep adherence to System76 COSMIC desktop conventions, Wayland protocols, and `libcosmic` widget ergonomics.
- **Model-View-Update (Elm) Paradigm**: Predictable state management and testability via strict unidirectional data flow.
- **Robust Protocol Compliance**: Matrix Rust SDK integration for E2EE, sliding sync / room list service, MSC4143 calls, and MSC2545 image packs.
- **Fault-Tolerant Desktop Lifecycle**: Single-instance coordination, headless background notification handling, and reliable secret persistence.

## Topology & Module Dependencies

```
                    ┌─────────────┐
                    │ src/main.rs │
                    └──────┬──────┘
                           │
       ┌───────────────────┼────────────────────┐
       ▼                   ▼                    ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ constellation│◄───┤     view     │    │ unified_push │
│   (App MVU)  │    │ (Rendering)  │    │  (Headless)  │
└──────┬───────┘    └──────┬───────┘    └──────────────┘
       │                   │
       ├──────────────┬────┘
       ▼              ▼
┌──────────────┐┌──────────────┐
│   settings   ││    matrix    │
│  (Sub-MVUs)  ││   (Engine)   │
└──────┬───────┘└──────┬───────┘
       │               │
       └───────┬───────┘
               ▼
        ┌──────────────┐
        │    utils     │
        │ (Foundations)│
        └──────────────┘
```

- **`src/main.rs`**: Application entry point. Installs crypto providers, initializes Fluent localization, checks single-instance DBus lock, handles headless notification relaunch (`--notify`), and boots `cosmic::app::run::<Constellation>`.
- **`src/constellation/`**: Coordinates the root Elm loop (`app.rs`), maintains global state (`state.rs`), handles messages (`mod.rs`), subscribes to asynchronous streams (`subscriptions.rs`), and dispatches shortcuts (`keybind.rs`).
- **`src/view/`**: Pure render functions for the application shell, chat timeline, switcher sidebar, login dialogs, verification cards, and about view. Consumes state and emits `Message`.
- **`src/matrix/`**: Wraps `matrix-rust-sdk`. Owns client authentication, session encryption, room list sync, timeline events, media uploads, calls, and search.
- **`src/settings/`**: Modular sub-MVU components for user, room, space, and application settings hosted in the COSMIC context drawer.
- **`src/utils/`**: Foundational helpers: Fluent i18n (`i18n.rs`), DBus session IPC (`ipc.rs`), item wrappers (`item.rs`), OpenGraph scraping (`og.rs`), Matrix permalink parsing (`permalink.rs`), and markdown previews (`preview.rs`).

## Key Decisions

### Isolated Runtime for Headless Push Notifications
UnifiedPush execution occurs via a dedicated background thread with its own `tokio::runtime::current_thread` runtime when launched with `--notify`. Because `unifiedpush` internals invoke `block_on`, executing it within Tokio's multi-threaded worker pool triggers a fatal "Cannot start a runtime from within a runtime" panic. Separating the runtime completely ensures safe synchronous blocking in headless daemon mode.

### DBus Single-Instance Enforcement
Constellation uses `zbus` on the user session bus (`fi.joonastuomi.Constellation`) to enforce a single running instance. When a secondary instance launches:
1. It queries DBus proxy `name_has_owner`.
2. If already running, it delegates command-line launch URIs (OIDC callbacks or permalinks) to the existing instance via DBus method calls.
3. The secondary process exits immediately without rendering a window or touching local databases.

### MAS-Compliant OIDC Callback URI Format
The desktop handler registers the `x-scheme-handler/fi.joonastuomi.constellation` mime scheme. For Matrix Authentication Service (MAS) compatibility, OAuth 2.0 callbacks use the lowercase single-slash format `fi.joonastuomi.constellation:/callback`. Internal permalink wrappers use double-slash `fi.joonastuomi.constellation://open?url=...`. Both are parsed and routed by `parse_launch_uri`.

### Global Allocator (mimalloc)
Constellation links against `mimalloc` as the global allocator (`#[global_allocator] static GLOBAL: MiMalloc = MiMalloc;`). This mitigates memory fragmentation and overhead under sustained high-throughput timeline event ingestion and image decoding.

### Linker Optimization for Low-Memory Systems
`.cargo/config.toml` passes `-Wl,--no-keep-memory` to the linker, relieving virtual address space and memory pressure during release link-time optimization (LTO).

## Invariants

- **Pure View Layer**: View functions in `src/view/` and settings subpages must remain pure rendering functions; they may not mutate state or initiate uncoordinated network requests.
- **Semantic Palette Theming**: Hardcoded RGB/HEX color literals in UI layouts are prohibited. All widgets must derive styling from `theme.palette` tokens to guarantee legibility across light and dark modes.
- **Mandatory Fluent Strings**: No hardcoded English strings may be displayed in user-facing UI. All text must use the `fl!` macro linked to `res/i18n/<lang>/constellation.ftl`.
- **Keyring & SQLite Store Coherence**: When the Secret Service keyring entry is reset or invalidated, corresponding SQLite cache and search directories must be invalidated to prevent decryption MAC failures.
