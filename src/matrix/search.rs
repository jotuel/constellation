use super::*;

impl MatrixEngine {
    pub async fn get_pinned_events(
        &self,
        room_id: &str,
    ) -> Result<Vec<matrix_sdk::ruma::OwnedEventId>> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        let pinned = room
            .get_state_event_static::<matrix_sdk::ruma::events::room::pinned_events::RoomPinnedEventsEventContent>()
            .await
            .ok()
            .flatten()
            .and_then(|e| e.deserialize().ok())
            .and_then(|ev| match ev {
                matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Sync(
                    matrix_sdk::ruma::events::SyncStateEvent::Original(ev),
                ) => Some(ev.content.pinned),
                matrix_sdk_base::deserialized_responses::SyncOrStrippedState::Stripped(
                    ev,
                ) => ev.content.pinned,
                _ => None,
            })
            .unwrap_or_default();
        Ok(pinned)
    }

    pub async fn fetch_pinned_event_details(
        &self,
        room_id: &str,
        event_id: &matrix_sdk::ruma::EventId,
    ) -> Result<PinnedEventInfo> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        let timeline_event = room.event(event_id, None).await?;
        let (sender_id, origin_server_ts, body) =
            extract_event_summary(&timeline_event).context("Failed to deserialize pinned event")?;
        let timestamp = format_timestamp(origin_server_ts);
        // Fetch sender member profile details for name and avatar
        let (sender_name, avatar_url) = if let Ok(Some(member)) = room.get_member(&sender_id).await
        {
            (
                member
                    .display_name()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| sender_id.to_string()),
                member.avatar_url().map(|u| u.to_string()),
            )
        } else {
            (sender_id.to_string(), None)
        };

        Ok(PinnedEventInfo {
            event_id: event_id.to_string(),
            sender_id: sender_id.to_string(),
            sender_name,
            avatar_url,
            timestamp,
            body,
        })
    }

    /// Replaces the room's `m.room.pinned_events` state with the given list.
    pub async fn set_pinned_events(
        &self,
        room_id: &str,
        pinned: Vec<matrix_sdk::ruma::OwnedEventId>,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        use matrix_sdk::ruma::events::room::pinned_events::RoomPinnedEventsEventContent;
        let content = RoomPinnedEventsEventContent::new(pinned);
        room.send_state_event(content).await?;
        Ok(())
    }

    pub async fn fetch_active_threads(&self, room_id: &str) -> Result<Vec<ActiveThreadInfo>> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        let opts = matrix_sdk::room::ListThreadsOptions {
            limit: matrix_sdk::ruma::UInt::new(50),
            ..Default::default()
        };
        let thread_roots_res = room.list_threads(opts).await;

        let mut sender_cache: std::collections::HashMap<
            matrix_sdk::ruma::OwnedUserId,
            (String, Option<String>),
        > = std::collections::HashMap::new();

        // Ensure the event cache is subscribed so ThreadEventCache can receive and persist events.
        let _ = client.event_cache().subscribe();

        match thread_roots_res {
            Ok(thread_roots) => {
                let mut active_threads = Vec::new();
                for event in thread_roots.chunk {
                    let Some(event_id) = event.event_id() else {
                        continue;
                    };
                    let Some((sender_id, origin_server_ts, body)) = extract_event_summary(&event)
                    else {
                        continue;
                    };

                    let timestamp = format_timestamp(origin_server_ts);
                    let num_replies = event
                        .thread_summary
                        .summary()
                        .map(|s| s.num_replies)
                        .unwrap_or(0);

                    let latest_activity =
                        event
                            .bundled_latest_thread_event()
                            .as_ref()
                            .and_then(|latest| {
                                latest.timestamp().map(format_timestamp).or_else(|| {
                                    extract_event_summary(latest)
                                        .map(|(_, ts, _)| format_timestamp(ts))
                                })
                            });

                    let (sender_name, avatar_url) = match sender_cache.entry(sender_id.clone()) {
                        std::collections::hash_map::Entry::Occupied(entry) => entry.get().clone(),
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            let profile =
                                if let Ok(Some(member)) = room.get_member(&sender_id).await {
                                    (
                                        member
                                            .display_name()
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| sender_id.to_string()),
                                        member.avatar_url().map(|u| u.to_string()),
                                    )
                                } else {
                                    (sender_id.to_string(), None)
                                };
                            entry.insert(profile).clone()
                        }
                    };

                    let (num_unread_messages, num_unread_notifications, num_unread_mentions) =
                        if let Ok((thread_cache, _)) =
                            client.event_cache().thread(&room_id_parsed, event_id).await
                        {
                            let msgs = thread_cache.num_unread_messages().await.unwrap_or(0);
                            let notifs = thread_cache.num_unread_notifications().await.unwrap_or(0);
                            let mentions = thread_cache.num_unread_mentions().await.unwrap_or(0);
                            (msgs, notifs, mentions)
                        } else {
                            (0, 0, 0)
                        };

                    active_threads.push(ActiveThreadInfo {
                        event_id: event_id.to_string(),
                        sender_id: sender_id.to_string(),
                        sender_name,
                        avatar_url,
                        timestamp,
                        body,
                        num_replies,
                        latest_activity,
                        num_unread_messages,
                        num_unread_notifications,
                        num_unread_mentions,
                    });
                }

                {
                    let mut inner = self.inner.write().await;
                    inner
                        .active_threads_cache
                        .insert(room_id_parsed.clone(), active_threads.clone());
                }

                Ok(active_threads)
            }
            Err(e) => {
                // Offline fallback: use cached active threads if available,
                // refreshing unread counts from local ThreadEventCache.
                let inner = self.inner.read().await;
                if let Some(cached) = inner.active_threads_cache.get(&room_id_parsed) {
                    let mut offline_threads = cached.clone();
                    drop(inner);

                    for item in &mut offline_threads {
                        if let Ok(event_id) = matrix_sdk::ruma::EventId::parse(&item.event_id)
                            && let Ok((thread_cache, _)) = client
                                .event_cache()
                                .thread(&room_id_parsed, &event_id)
                                .await
                        {
                            if let Ok(msgs) = thread_cache.num_unread_messages().await {
                                item.num_unread_messages = msgs;
                            }
                            if let Ok(notifs) = thread_cache.num_unread_notifications().await {
                                item.num_unread_notifications = notifs;
                            }
                            if let Ok(mentions) = thread_cache.num_unread_mentions().await {
                                item.num_unread_mentions = mentions;
                            }
                        }
                    }

                    Ok(offline_threads)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    pub async fn search_messages_in_room(
        &self,
        room_id: &str,
        query: &str,
        max_results: usize,
    ) -> Result<(Vec<MessageSearchResult>, bool)> {
        // Check the cached decision.
        let use_server = {
            let inner = self.inner.read().await;
            inner.server_search_supported.unwrap_or(true)
        };

        if use_server {
            match self.server_search(room_id, query, max_results, None).await {
                Ok((results, next_batch)) => {
                    // Cache support on first success.
                    let mut inner = self.inner.write().await;
                    inner.server_search_supported = Some(true);
                    let has_more = next_batch.is_some();
                    inner.active_search = Some(ActiveSearch::Server {
                        query: query.to_owned(),
                        room_id: room_id.to_owned(),
                        next_batch,
                    });
                    return Ok((results, has_more));
                }
                Err(SearchError::Unsupported) => {
                    // 404/405 — cache and fall through to local backfill.
                    let mut inner = self.inner.write().await;
                    inner.server_search_supported = Some(false);
                }
                Err(SearchError::Other(e)) => return Err(e),
            }
        }

        self.local_search_with_backfill(room_id, query, max_results)
            .await
    }

    /// Server-side `/search` via `POST /_matrix/client/v3/search`, scoped to a
    /// single room. Returns a typed error so the caller can distinguish
    /// "endpoint not supported" from real failures.
    async fn server_search(
        &self,
        room_id: &str,
        query: &str,
        max_results: usize,
        next_batch: Option<String>,
    ) -> std::result::Result<(Vec<MessageSearchResult>, Option<String>), SearchError> {
        let room_id_parsed =
            RoomId::parse(room_id).map_err(|e| SearchError::Other(anyhow::anyhow!(e)))?;
        let client = self.client().await;

        let parsed = crate::utils::search_query::parse_search_query(query);

        // If there is no text search term, homeservers typically reject the query.
        // Fall back to local search (Tantivy) which supports sender/date-only searches.
        if parsed.sanitized_query.is_empty()
            && (parsed.sender_filter.is_some()
                || parsed.date_after.is_some()
                || parsed.date_before.is_some())
        {
            return Err(SearchError::Unsupported);
        }

        use matrix_sdk::ruma::api::client::search::search_events::v3;

        let mut filter = matrix_sdk::ruma::api::client::filter::RoomEventFilter::default();
        filter.rooms = Some(vec![room_id_parsed.clone()]);
        filter.limit = Some(
            matrix_sdk::ruma::UInt::try_from(max_results).unwrap_or(matrix_sdk::ruma::UInt::MAX),
        );

        if let Some(sender_str) = &parsed.sender_filter {
            if let Ok(user_id) = matrix_sdk::ruma::UserId::parse(sender_str) {
                filter.senders = Some(vec![user_id.to_owned()]);
            } else if let Ok(members) = self.get_room_members(room_id).await {
                let clean_name = sender_str.trim_start_matches('@');
                let maybe_member = members.iter().find(|m| {
                    m.user_id
                        .split(':')
                        .next()
                        .is_some_and(|u| u.trim_start_matches('@').eq_ignore_ascii_case(clean_name))
                        || m.display_name
                            .as_deref()
                            .is_some_and(|d| d.eq_ignore_ascii_case(clean_name))
                });
                if let Some(m) = maybe_member
                    && let Ok(user_id) = matrix_sdk::ruma::UserId::parse(&m.user_id)
                {
                    filter.senders = Some(vec![user_id.to_owned()]);
                }
            }
        }

        let search_term = if !parsed.sanitized_query.is_empty() {
            parsed.sanitized_query.clone()
        } else {
            query.to_owned()
        };

        let mut criteria = v3::Criteria::new(search_term);
        criteria.filter = filter;
        criteria.keys = Some(vec![v3::SearchKeys::ContentBody]);
        let mut categories = v3::Categories::new();
        categories.room_events = Some(criteria);
        let mut request = v3::Request::new(categories);
        request.next_batch = next_batch;

        let response = client.send(request).await.map_err(|e| {
            let sdk_err = matrix_sdk::Error::from(e);
            if is_search_unsupported(&sdk_err) {
                SearchError::Unsupported
            } else {
                SearchError::Other(anyhow::anyhow!(sdk_err))
            }
        })?;
        let room_results = response.search_categories.room_events;

        let new_next_batch = room_results.next_batch.clone();

        let mut results = Vec::with_capacity(room_results.results.len());
        for raw_event in room_results.results.into_iter().filter_map(|r| r.result) {
            // The server returns decrypted plain-text events (it has the keys
            // for rooms we're joined to), so a single deserialize suffices.
            let event: matrix_sdk::ruma::events::AnyTimelineEvent = raw_event
                .deserialize()
                .map_err(|e| SearchError::Other(anyhow::anyhow!(e)))?;

            let event_id = event.event_id().to_owned();
            let sender_id = event.sender().to_owned();

            let body = match &event {
                matrix_sdk::ruma::events::AnyTimelineEvent::MessageLike(msg) => match msg {
                    matrix_sdk::ruma::events::AnyMessageLikeEvent::RoomMessage(
                        matrix_sdk::ruma::events::MessageLikeEvent::Original(
                            matrix_sdk::ruma::events::OriginalMessageLikeEvent { content, .. },
                        ),
                    ) => content.body().to_string(),
                    _ => "Unsupported message event type".to_string(),
                },
                _ => "Unsupported state event type".to_string(),
            };

            let ts_millis = u64::from(event.origin_server_ts().0);
            if let Some(dt) = chrono::DateTime::from_timestamp_millis(ts_millis as i64) {
                let event_date = dt.with_timezone(&chrono::Local).date_naive();
                if let Some(after) = parsed.date_after
                    && event_date < after
                {
                    continue;
                }
                if let Some(before) = parsed.date_before
                    && event_date > before
                {
                    continue;
                }
            }

            let timestamp = {
                chrono::DateTime::from_timestamp_millis(ts_millis as i64)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            };

            let plain_text = crate::preview::parse_plain_text(&body);
            let links = crate::preview::extract_links(&plain_text);

            results.push(MessageSearchResult {
                room_id: room_id_parsed.clone(),
                room_name: None,
                event_id,
                sender_id,
                body,
                timestamp,
                plain_text,
                links,
            });
        }

        Ok((results, new_next_batch))
    }

    /// Local fallback: paginate the room's full history backwards through the
    /// event cache (which auto-indexes events into the seshat store), then
    /// query the local search index.
    ///
    /// The backfill only does significant network work the first time a room is
    /// searched (the index is empty); on subsequent searches the seshat index
    /// already has the history, so `paginate_backwards` hits the on-disk event
    /// cache (fast, no network) and the local query returns instantly.
    async fn local_search_with_backfill(
        &self,
        room_id: &str,
        query: &str,
        _max_results: usize,
    ) -> Result<(Vec<MessageSearchResult>, bool)> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let room = {
            let inner = self.inner.read().await;
            inner
                .client
                .get_room(&room_id_parsed)
                .context("Room not found")?
        };
        let timeline = self.timeline(room_id).await?;

        // 1. Backfill: paginate backwards until the timeline start is reached.
        //    The network path feeds events into the seshat index automatically.
        //    Cap iterations as a safety valve against infinite loops on very
        //    large rooms (500 × 50 = 25 000 events).
        for _ in 0..500 {
            let reached_start = timeline.paginate_backwards(50).await?;
            if reached_start {
                break;
            }
        }

        // 2. Query the now-populated local seshat index.
        let parsed = crate::utils::search_query::parse_search_query(query);
        let mut resolved_sender = None;
        if let Some(sender_str) = &parsed.sender_filter {
            if matrix_sdk::ruma::UserId::parse(sender_str).is_ok() {
                resolved_sender = Some(sender_str.clone());
            } else if let Ok(members) = self.get_room_members(room_id).await {
                let clean_name = sender_str.trim_start_matches('@');
                if let Some(m) = members.iter().find(|m| {
                    m.user_id
                        .split(':')
                        .next()
                        .is_some_and(|u| u.trim_start_matches('@').eq_ignore_ascii_case(clean_name))
                        || m.display_name
                            .as_deref()
                            .is_some_and(|d| d.eq_ignore_ascii_case(clean_name))
                }) {
                    resolved_sender = Some(m.user_id.clone());
                }
            }
        }
        let tantivy_query = parsed.to_tantivy_query(resolved_sender.as_deref());
        let effective_query = if tantivy_query.is_empty() {
            query.to_owned()
        } else {
            tantivy_query
        };

        let mut search_stream = Box::pin(room.search_messages_events(effective_query));
        let Some(events_res) = search_stream.next().await else {
            let mut inner = self.inner.write().await;
            inner.active_search = Some(ActiveSearch::Local {
                room_id: room_id_parsed.clone(),
                search_stream: tokio::sync::Mutex::new(search_stream),
            });
            return Ok((Vec::new(), false));
        };
        let events = events_res?;

        // 3. Map TimelineEvents → MessageSearchResult using functional style.
        // Bolt Optimization: Functional chain avoids dynamic reallocations by size hint
        let results = events
            .into_iter()
            .filter_map(|e| map_timeline_event(&room_id_parsed, None, e).transpose())
            .collect::<Result<Vec<_>>>()?;

        let mut inner = self.inner.write().await;
        inner.active_search = Some(ActiveSearch::Local {
            room_id: room_id_parsed.clone(),
            search_stream: tokio::sync::Mutex::new(search_stream),
        });

        Ok((results, true))
    }

    pub async fn search_messages_in_room_next_batch(
        &self,
        max_results: usize,
    ) -> Result<(Vec<MessageSearchResult>, bool)> {
        // Bolt Optimization: Retrieve owned ActiveSearch to avoid holding
        // the RwLockWriteGuard of inner across await boundaries.
        let active_search = {
            let mut inner = self.inner.write().await;
            inner.active_search.take()
        };

        let Some(mut search) = active_search else {
            return Ok((Vec::new(), false));
        };

        let res = match &mut search {
            ActiveSearch::Local {
                room_id,
                search_stream,
            } => {
                let next_page = search_stream.lock().await.next().await;
                let events = match next_page {
                    Some(res) => res?,
                    None => Vec::new(),
                };
                let has_more = !events.is_empty();
                let results = events
                    .into_iter()
                    .filter_map(|e| map_timeline_event(room_id, None, e).transpose())
                    .collect::<Result<Vec<_>>>()?;

                Ok((results, has_more))
            }
            ActiveSearch::Server {
                query,
                room_id,
                next_batch,
            } => {
                if next_batch.is_none() {
                    Ok((Vec::new(), false))
                } else {
                    match self
                        .server_search(room_id, query, max_results, next_batch.clone())
                        .await
                    {
                        Ok((results, new_next_batch)) => {
                            *next_batch = new_next_batch;
                            let has_more = next_batch.is_some();
                            Ok((results, has_more))
                        }
                        Err(SearchError::Unsupported) => Ok((Vec::new(), false)),
                        Err(SearchError::Other(e)) => Err(e),
                    }
                }
            }
        };

        {
            let mut inner = self.inner.write().await;
            inner.active_search = Some(search);
        }

        res
    }

    /// Search across all joined rooms via the local seshat index. Uses
    /// `matrix_sdk::Client::search_messages` (`GlobalSearchIterator`), which
    /// queries each room's local index (the same one the in-room local fallback
    /// populates). Does not hit the server `/search` endpoint.
    ///
    /// `scope` narrows the working set to DM / group rooms before searching.
    /// Unlike [`search_messages_in_room`], there is no server probe and no
    /// backfill step here — the global iterator only sees events already in
    /// the client's index (per-room backfill happens when each room is opened
    /// or in-room-searched). Pagination / "load more" is a separate concern
    /// (issue #303); this fetches a single batch.
    pub async fn search_messages_global(
        &self,
        query: &str,
        max_results: usize,
        scope: GlobalSearchScope,
    ) -> Result<Vec<MessageSearchResult>> {
        let client = self.client().await;

        let parsed = crate::utils::search_query::parse_search_query(query);
        let resolved_sender = if let Some(sender_str) = &parsed.sender_filter {
            if matrix_sdk::ruma::UserId::parse(sender_str).is_ok() {
                Some(sender_str.as_str())
            } else {
                None
            }
        } else {
            None
        };
        let tantivy_query = parsed.to_tantivy_query(resolved_sender);
        let effective_query = if tantivy_query.is_empty() {
            query.to_owned()
        } else {
            tantivy_query
        };

        let effective_scope = parsed.scope.unwrap_or(scope);

        let builder = client.search_messages(effective_query);
        let builder = match effective_scope {
            GlobalSearchScope::All => builder,
            GlobalSearchScope::DmsOnly => builder.only_dm_rooms().await?,
            GlobalSearchScope::GroupsOnly => builder.no_dms().await?,
        };
        let mut search_stream = Box::pin(builder.build_events());

        let Some(events_res) = search_stream.next().await else {
            return Ok(Vec::new());
        };
        let mut events = events_res?;
        events.truncate(max_results);

        let mut results = Vec::with_capacity(events.len());
        for (room_id, event) in events {
            // Resolve a display name for the originating room (best-effort).
            let room_name = client.get_room(&room_id).and_then(|room| {
                room.name()
                    .or_else(|| room.cached_display_name().map(|n| n.to_string()))
            });
            if let Some(hit) = map_timeline_event(&room_id, room_name, event)? {
                results.push(hit);
            }
        }

        Ok(results)
    }
}
fn format_timestamp(ts: matrix_sdk::ruma::MilliSecondsSinceUnixEpoch) -> String {
    let ts_millis = u64::from(ts.0);
    let datetime = chrono::DateTime::from_timestamp_millis(ts_millis as i64).unwrap_or_default();
    datetime
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn extract_event_summary(
    timeline_event: &matrix_sdk::deserialized_responses::TimelineEvent,
) -> Option<(
    matrix_sdk::ruma::OwnedUserId,
    matrix_sdk::ruma::MilliSecondsSinceUnixEpoch,
    String,
)> {
    match &timeline_event.kind {
        matrix_sdk::deserialized_responses::TimelineEventKind::Decrypted(decrypted) => {
            let ev = decrypted.event.deserialize().ok()?;
            let sender = ev.sender().to_owned();
            let ts = ev.origin_server_ts();
            let body = match &ev {
                matrix_sdk::ruma::events::AnyTimelineEvent::MessageLike(msg) => match msg {
                    matrix_sdk::ruma::events::AnyMessageLikeEvent::RoomMessage(
                        matrix_sdk::ruma::events::MessageLikeEvent::Original(
                            matrix_sdk::ruma::events::OriginalMessageLikeEvent { content, .. },
                        ),
                    ) => content.body().to_string(),
                    _ => "Unsupported message event type".to_string(),
                },
                _ => "Unsupported state event type".to_string(),
            };
            Some((sender, ts, body))
        }
        matrix_sdk::deserialized_responses::TimelineEventKind::UnableToDecrypt {
            event, ..
        } => {
            let ev = event.deserialize().ok()?;
            let sender = ev.sender().to_owned();
            let ts = ev.origin_server_ts();
            let body = match &ev {
                matrix_sdk::ruma::events::AnySyncTimelineEvent::MessageLike(msg) => match msg {
                    matrix_sdk::ruma::events::AnySyncMessageLikeEvent::RoomMessage(
                        matrix_sdk::ruma::events::SyncMessageLikeEvent::Original(
                            matrix_sdk::ruma::events::OriginalSyncMessageLikeEvent {
                                content, ..
                            },
                        ),
                    ) => content.body().to_string(),
                    _ => "Unsupported message event type".to_string(),
                },
                _ => "Unsupported state event type".to_string(),
            };
            Some((sender, ts, body))
        }
        matrix_sdk::deserialized_responses::TimelineEventKind::PlainText { event, .. } => {
            let ev = event.deserialize().ok()?;
            let sender = ev.sender().to_owned();
            let ts = ev.origin_server_ts();
            let body = match &ev {
                matrix_sdk::ruma::events::AnySyncTimelineEvent::MessageLike(msg) => match msg {
                    matrix_sdk::ruma::events::AnySyncMessageLikeEvent::RoomMessage(
                        matrix_sdk::ruma::events::SyncMessageLikeEvent::Original(
                            matrix_sdk::ruma::events::OriginalSyncMessageLikeEvent {
                                content, ..
                            },
                        ),
                    ) => content.body().to_string(),
                    _ => "Unsupported message event type".to_string(),
                },
                _ => "Unsupported state event type".to_string(),
            };
            Some((sender, ts, body))
        }
    }
}
