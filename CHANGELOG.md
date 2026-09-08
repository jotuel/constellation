# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-09-08

### New Features

#### Chat & Tabs

- **Tabbed chat UI** — Open multiple chat rooms simultaneously in tabs across the chat header. Switch between open rooms, close tabs with `Ctrl+W` or tab close buttons, and access tab-level secondary actions (e.g. mark as read, leave room, copy permalink).
- **Open Graph link previews** — URLs displayed in chat messages automatically fetch and render rich Open Graph link previews (title, description, site name, domain, and thumbnail image) in a card container directly below the message text. Direct image URLs (including WebP, PNG, JPEG, GIF, SVG, and AVIF) automatically render as image preview cards.
- **Native text selection** — Replaced custom rich text rendering with libcosmic's native `SelectableText`, enabling standard text selection and copying across messages in the timeline.
- **Unread-room cards on empty state** — When no room is selected, the main chat pane displays joined rooms with unread messages as clickable cards (with avatar, name, and unread count) below the placeholder, serving as a quick jump-off point.
- **Composer drag-and-drop** — Files can now be dragged onto the message composer to attach them, in addition to the existing file-picker button.
- **Inline markdown links** — Markdown hyperlinks (`[text](url)`) are rendered as clickable Link widgets inside message bubbles in all rendering modes, rather than only as plain text.
- **Tasklists in markdown** — GitHub-flavored task lists (`- [ ]` / `- [x]`) now render as interactive checkboxes in markdown messages.
- **Copy permalink buttons** — Added copy-permalink buttons for individual messages and for rooms.

#### Navigation

- **Spaces in the COSMIC navigation bar** — Replaced the dedicated space switcher column with libcosmic's built-in navigation bar. Joined spaces are listed with both their display name and avatar (with a fallback icon for avatar-less spaces), alongside an "All rooms" entry. The navigation panel can be toggled from the header, and the room/space creation menu is now accessible from the header.
- **Unselect Space in switcher** — Added the ability to deselect the active space in the space switcher to view all rooms or close the space switcher.
- **Resizable main view section** — The split divider between the room switcher sidebar and the main chat area is now draggable and resizable using libcosmic's `PaneGrid`. The sidebar split ratio is persisted in COSMIC config.

#### Search

- **In-room message search** — Replaced the client-side fuzzy filter (which only searched the currently loaded timeline window) with a full-text search of the entire room history. Results include sender, timestamp, and body, with the surrounding context jumpable directly from a hit.
- **Server-side search with local fallback** — Search now prefers the homeserver's `/search` endpoint and falls back to the local seshat index when the server doesn't support it, so search works regardless of homeserver capability.
- **Cross-room (global) search** — When the search bar is active with no room selected, queries run across all joined rooms via the local seshat index. Each hit shows its room of origin, and a scope toggle (All / Direct Messages / Groups) narrows the working set. Clicking a result opens the originating room and jumps to the message.
- **Search pagination** — Message search results load in batches with a "Load more" control to page through additional hits.
- **Debounced public-rooms directory search** — Typing in the public rooms/spaces directory now debounces before querying, so fast typing no longer hammers the homeserver.

#### Media

- **In-app video playback** — Received video messages can now be played directly in the chat behind a "Play Video" button, with pause/resume control. Playback is powered by GStreamer, so any format your GStreamer plugins decode works. Requires GStreamer at runtime; the feature is controlled by the `video-player` Cargo feature (on by default).
- **Autoplay videos** — New "Autoplay Videos" setting (on by default): received videos start playing inline as soon as they arrive, looping and muted like GIFs. Toggle it off for manual, audio-enabled playback behind a play button. When the sender provided one, the video's thumbnail is shown on the play button.
- **Save received media to disk** — Image, file, video, and audio messages carry a download icon in the message action row; pressing it opens a save dialog and writes the actual file (decryption for encrypted rooms happens transparently).

#### Matrix Integration

- **Start DM from user permalink** — Opening a `matrix:` user permalink now starts a direct message with that user instead of erroring.
- **`matrix:` URI scheme handler** — Registered the `matrix:` URI scheme at the OS level (`.desktop` + metainfo), so `matrix.to` and matrix permalinks open Constellation directly.
- **Real QR code login (MSC4108)** — Replaced the non-functional QR login stub (which generated an invalid `matrix.to` URL no scanner could read) with the real MSC4108 sign-in flow from matrix-rust-sdk. The QR now encodes a valid binary `QrCodeData` payload that an existing Matrix device (e.g. Element Mobile) can scan to grant login, with full secure-channel establishment, check-code confirmation, OAuth device-authorization, and end-to-end encryption secret transfer.

#### Settings & Shortcuts

- **Customizable keyboard shortcuts** — Added a dedicated Keyboard Shortcuts settings page where users can view, record, and reset keybindings. Rebindable actions include room navigation, space-switcher toggle, timeline scrolling, and search focus, with global dispatch ensuring shortcuts do not fire while typing in input fields.
- **COSMIC Config persistence** — User settings are now stored via the COSMIC Config system instead of a custom JSON file, integrating with `cosmic-settings` and following the COSMIC desktop convention. (Note: existing `config.json` files from earlier alphas are not migrated — settings reset to defaults on first launch after upgrade.)

### Bug Fixes

- **Timeline scroll anchoring** — Anchored timeline scroll positions across window resizing, reflows, and remounts so that incoming messages or view changes do not cause jarring scroll jumps.
- **Empty state vs. unread state** — Fixed an issue where the empty state could erroneously display both unread cards and the empty placeholder simultaneously.
- **Search query in window title** — The active search query is now displayed in the window title while filtering room messages.
- **OIDC login actually completes** — OIDC/OAuth login (and QR login, which depends on it) previously hung on "Waiting for browser": the client was never registered with the homeserver's MAS, and the redirect URI was rejected for a non-lowercase scheme and double-slash form. Both are fixed — login now performs dynamic client registration and uses the MAS-compliant `fi.joonastuomi.constellation:/callback` redirect URI. A dedicated error is surfaced when the homeserver doesn't support OAuth.
- **Closed a homeserver URL spoofing vulnerability** — Homeserver input was matched with a loose `starts_with("http://127.0.0.1")` check, allowing look-alike hosts like `127.0.0.1.attacker.com` or `localhost@attacker.com` to bypass the localhost allowance. URLs are now parsed and validated by exact host (and userinfo is stripped), closing the spoofing vector across password, OIDC, and QR login.
- **Markdown links parsed in plain-text mode** — Fixed markdown links not being extracted when a message rendered in plain-text mode.
- **Video and audio messages rendered** — Received video and audio messages previously fell through to the plain-text fallback and showed only as text; they now render as proper media messages with download (and, for video, playback) actions.
- **Safe UTF-8 string slicing in Open Graph parser** — Fixed a potential panic when parsing Open Graph metadata or truncating descriptions from web pages containing multi-byte UTF-8 characters and emojis.
- **HTTP header compatibility for link previews** — Included standard `Accept` and `Accept-Language` headers in preview fetch requests to prevent HTTP 403 errors on strict hosts (such as Codeberg).
- **Avatar and file upload size limits** — Enforced size limits during file and avatar uploads to prevent excessive memory usage and potential out-of-memory crashes.
- **DBus name conversion** — Fixed a potential panic when converting invalid or unexpected characters in DBus service names.
- **SVG icon legibility** — Improved contrast and stroke visibility in application SVG icons.

### Security

- **Removed insecure secret-storage fallback** — Matrix credentials (session tokens, store passphrase) no longer fall back to being written as plaintext files when Keyring is unavailable; the failure now bubbles up as an error instead of silently leaking secrets to disk.
- **Homeserver fallback URL sanitization** — Stripped nested schemes and invalid prefixes in fallback homeserver URL handling to prevent HTTP downgrade attacks.
- **Restricted external URL schemes** — Prevented opening non-HTTP/HTTPS URLs when clicking external links to avoid unintended protocol execution.
- **HTTPS-only Open Graph requests** — Enforced HTTPS-only connections for Open Graph web requests, preventing cleartext HTTP transmission and interception.
- **Owner-only video temp file permissions** — Explicitly set `0o600` (read/write by owner only) permissions on temporary video media files on Unix platforms immediately upon creation.

### Performance Improvements

- **Incremental Open Graph link previews** — Updated timeline diff processing to incrementally fetch previews only for newly arrived URLs rather than refetching existing timeline previews.
- **Deferred space navigation rebuilds** — Used a dirty flag to avoid rebuilding the space navigation tree when room or space state has not changed.
- **Parallelized space children fetching** — Fetched space children concurrently using `try_join_all`, speeding up space hierarchy resolution.
- **Batched media fetches in search** — Batched avatar and media requests during search result loading to reduce request overhead.
- **O(1) timeline event lookup** — Replaced O(N) reverse search traversal with O(1) hash map lookups for timeline events during rendering.
- **Optimized room name and avatar resolution** — Reduced string clones and memory allocations during room name and avatar cache lookups.
- **Async store operations and avatar uploads** — Switched blocking filesystem I/O to asynchronous Tokio operations when clearing the Matrix store or uploading avatars.

## [0.1.0] - 2026-07-09

First alpha release. Usable, but expect bugs, missing features, and breaking changes before the eventual 1.0.


### New Features

- **UnifiedPush notifications** — Added support for UnifiedPush background notification handler, allowing real-time push notifications.
- **Room members & pinned messages panels** — Added collapsible side panels for viewing room members and pinned messages in the chat view.
- **Stable timeline scrolling** — Implemented stable timeline scrolling to prevent jumpy scroll behavior when new messages arrive.
- **Start from oldest unread** — Automatically scroll to and start from the oldest unread message when re-joining a chat room.
- **Plain-text URL parsing** — Added support for parsing plain-text URLs into clickable links in message bubbles across all rendering modes.
- **QR code login** — Implemented secure QR code login using a custom QR code scanner widget.
- **Location sharing** — Added support for viewing and sending shared locations.
- **MatrixRTC (LiveKit)** — Added experimental support for MatrixRTC group calls powered by LiveKit.
- **Multi-line chat editor** — Improved the message composer to support writing and editing multi-line messages easily.

### Bug Fixes

- Fixed a panic on start-up related to search index database corruption by automatically clearing the search index on cryptographic key mismatch (invalid MAC) or fresh store creation.
- Fixed an issue where trigger-happy system notifications would cause nested runtime panics by switching to the async notification API.
- Fixed reaction emoji rendering and interactions in chat bubbles.
- Strip reply fallback quotes from room list message previews to keep previews clean and legible.
- Fixed device verification status checks, enabled incoming room key requests, and moved the verification UI to a more intuitive location.
- Fixed a bug where message previews would display raw newline characters instead of space separation.
- Fixed a date divider bug to ensure date headers are only displayed for days containing actual, visible messages in the timeline.

### User Interface & Experience

- **Localized settings** — Fully translated User Settings and User Notification Settings into multiple languages.
- **Improved settings layout** — Stacked inputs and controls in settings pages to fit cleanly on narrow screens and mobile layouts.
- **Visual dividers** — Added subtle horizontal and vertical pane dividers to improve workspace boundaries in multi-pane layouts.
- **Unified timeline composer** — Redesigned the chat composer with a cohesive card-based UI that matches the timeline theme.
- **ListItem room lists** — Styled the sidebar room list with clean, consistent `ListItem` widgets.
- **Icon buttons in compact spaces** — Replaced text buttons with streamlined icon-only buttons in compact spaces and for destructive actions.
- **Search empty state** — Added localized helper text and clear illustration when search results are empty.
- **Close button tooltips** — Added helpful tooltips to the close buttons for the emoji picker and full-screen image viewer.

### Performance Improvements

- **Optimized localization allocations** — Prevented string allocation bottlenecks in `view_item` and view loops by caching and passing localized strings by reference.
- **Zero-allocation timeline items** — Pre-computed `TimelineEventItemId` and cached room event identifiers in the render loop to eliminate per-frame heap allocations.
- **Optimized thread rendering** — Pre-calculated thread root IDs and thread counts in background data models to avoid allocations during view traversal.
- **Optimized room name resolution** — Avoided string allocations per frame when resolving active room names in the main UI thread.
- **Media cache lookup optimization** — Eliminated unnecessary heap-allocated string copies during media cache lookups.
