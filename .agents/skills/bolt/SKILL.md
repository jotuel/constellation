---
name: bolt
description: Performance optimization log and guidelines (RwLock contention, case-insensitive string filtering, reusing Vec allocations, etc.) for COSMIC/Rust applications.
---

# COSMIC/Rust Performance Optimization Log

This skill documents performance optimizations, lessons, and best practices for developing high-performance Rust applications under libcosmic/iced.

## 1. Concurrency & Locking
* **RwLock Contention on State Trees**: Iterating and executing locking checks repeatedly (e.g. `is_in_space_sync` per room) causes massive locking overhead.
  - *Action:* Implement bulk queries (e.g. `is_in_space_bulk`) to retrieve state checks in a single read lock. Batch filtering logic using iterators inside a scoped lock guard avoids massive pre-allocation and atomic lock thrashing.
* **std::mem::take Safety**: When passing a reused `Vec` buffer via `std::mem::take` into a method that acquires a lock (e.g., `filter_in_space_bulk_sync`), clearing the buffer *before* acquiring the lock causes UI flickers if the lock fails.
  - *Action:* Always call `.clear()` on the buffer *inside* the success branch of the lock to preserve data consistency.

## 2. Zero-Allocation String & Search Filtering
* **Case-Insensitive ASCII Fast-Path**: Calling `.to_lowercase()` unconditionally inside a hot loop causes severe heap allocation.
  - *Action:* Check if the query is ASCII. If it is, use byte-wise windowed comparisons (`haystack.as_bytes().windows(len).any(|w| w.eq_ignore_ascii_case(query_bytes))`) instead of converting strings to lowercase.
* **Unrolling First-Byte Match**: The `.windows()` search can be sped up by checking the first byte of the query, converting it to upper/lower case, and only performing full-slice comparisons when the first byte matches.
* **Avoid redundant `haystack.is_ascii()` checks**: If the query is pure ASCII, UTF-8 multi-byte characters (values >= 128) can never spuriously match ASCII bytes, so no pre-check is needed.
* **Correct Fallback Initialization**: Do NOT initialize search fallbacks to an empty string (`""`). An empty string always evaluates to true in `.contains()`, matching everything. Use `Option<String>` pattern: `let fallback = (!is_ascii).then(|| query.to_lowercase());`.
* **Reusing Vec allocations**: Avoid creating vectors via `Vec::new()` and `.collect()` in search updates. Use `std::mem::take` to clear and extend existing vectors in the application state.

## 3. Render Loop (`view()`) Optimization
* **Avoid Message Clones**: Do not build messages inside `view()` by cloning large data structures (e.g. `Message::StartReply(item.clone())`). Instead, pass lightweight identifiers (like `TimelineEventItemId`) and lookup data in `update()`.
* **Pass Identifiers by Reference**: Pass `matrix_sdk::ruma::OwnedUserId` to helper functions as references (`Option<&UserId>`). Call `.to_owned()` only at the exact site of message construction inside action handlers.
* **Static String Messages in View**: In render loops for static collections (like emojis), accept `&'static str` inside `view()`, and allocate `to_string()` only inside the `update()` handler on user interaction.
* **Avoid `.to_string()` for Map Lookups**: Do not call `.to_string()` on temporary keys solely to query a cache (like `HashMap<String, V>`). Map queries accept borrowed keys (`&str`) naturally.
* **Precompute Derived Collections**: Avoid aggregations or `.count()` traversals inside the `view()` loop, which turns rendering into `O(N^2)`. Precompute maps/counts in the state and pass them down.
* **Avoid `fl!` Localized String Allocations**: Avoid calling `crate::fl!()` unconditionally in rendering. Localized messages dynamically allocate strings. Wrap them in a closure or `unwrap_or_else` block to evaluate lazily.
* **Pre-evaluate Expensive JSON/SDK properties**: Methods like `.thread_root()` that parse JSON and allocate IDs should be run exactly once during model initialization (e.g. inside `ConstellationItem::new`), not on every frame in the render loop.
