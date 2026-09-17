use super::*;

impl MatrixEngine {
    pub async fn timeline(&self, room_id: &str) -> Result<Arc<Timeline>> {
        let room_id = RoomId::parse(room_id)?;

        let rls = self
            .room_list_service()
            .await
            .context("RoomListService not initialized")?;
        rls.set_room_subscriptions(&[&room_id]).await;

        {
            let inner = self.inner.read().await;
            if let Some(timeline) = inner.timelines.get(&room_id) {
                return Ok(timeline.clone());
            }
        }

        let room = rls
            .room(&room_id)
            .map_err(|e| anyhow::anyhow!("Failed to get room: {}", e))?;
        let timeline = Arc::new(
            room.timeline_builder()
                .with_focus(TimelineFocus::Live {
                    hide_threaded_events: false,
                })
                .build()
                .await?,
        );

        let mut inner = self.inner.write().await;
        inner.timelines.insert(room_id.to_owned(), timeline.clone());

        Ok(timeline)
    }

    pub async fn threaded_timeline(
        &self,
        room_id: &str,
        root_event_id: &matrix_sdk::ruma::EventId,
    ) -> Result<Arc<Timeline>> {
        let room_id = RoomId::parse(room_id)?;
        let root_event_id = root_event_id.to_owned();

        let rls = self
            .room_list_service()
            .await
            .context("RoomListService not initialized")?;
        rls.set_room_subscriptions(&[&room_id]).await;

        {
            let inner = self.inner.read().await;
            if let Some(timeline) = inner
                .threaded_timelines
                .get(&(room_id.clone(), root_event_id.clone()))
            {
                return Ok(timeline.clone());
            }
        }

        let room = rls
            .room(&room_id)
            .map_err(|e| anyhow::anyhow!("Failed to get room: {}", e))?;
        let timeline = Arc::new(
            room.timeline_builder()
                .with_focus(TimelineFocus::Thread {
                    root_event_id: root_event_id.clone(),
                })
                .build()
                .await?,
        );

        let mut inner = self.inner.write().await;
        inner
            .threaded_timelines
            .insert((room_id.to_owned(), root_event_id), timeline.clone());

        Ok(timeline)
    }
    /// Query unread metrics for a thread using `ThreadEventCache`.
    pub async fn thread_unread_counts(
        &self,
        room_id: &str,
        thread_root_id: &matrix_sdk::ruma::EventId,
    ) -> Result<ThreadUnread> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let _ = client.event_cache().subscribe();
        let (thread_cache, _drop_handle) = client
            .event_cache()
            .thread(&room_id_parsed, thread_root_id)
            .await?;

        let num_unread_messages = thread_cache.num_unread_messages().await.unwrap_or(0);
        let num_unread_notifications = thread_cache.num_unread_notifications().await.unwrap_or(0);
        let num_unread_mentions = thread_cache.num_unread_mentions().await.unwrap_or(0);

        Ok(ThreadUnread {
            num_unread_messages,
            num_unread_notifications,
            num_unread_mentions,
        })
    }

    /// Subscribe to real-time `ThreadInfo` updates for a thread using `ThreadEventCache`.
    pub async fn subscribe_to_thread_info(
        &self,
        room_id: &str,
        thread_root_id: &matrix_sdk::ruma::EventId,
    ) -> Result<
        eyeball::Subscriber<matrix_sdk_base::event_cache::thread::ThreadInfo, eyeball::AsyncLock>,
    > {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let _ = client.event_cache().subscribe();
        let (thread_cache, _drop_handle) = client
            .event_cache()
            .thread(&room_id_parsed, thread_root_id)
            .await?;
        let subscriber = thread_cache.subscribe_to_thread_info().await?;
        Ok(subscriber)
    }

    /// Mark a threaded timeline as read by sending a threaded read receipt.
    pub async fn mark_threaded_timeline_as_read(
        &self,
        room_id: &str,
        root_event_id: &matrix_sdk::ruma::EventId,
    ) -> Result<bool> {
        let timeline = self.threaded_timeline(room_id, root_event_id).await?;
        let res = timeline
            .mark_as_read(
                matrix_sdk::ruma::api::client::receipt::create_receipt::v3::ReceiptType::Read,
            )
            .await?;
        Ok(res)
    }

    /// Build (or fetch from cache) a timeline focused on a specific event, used
    /// to open permalinks to messages that are not present in the live window.
    ///
    /// Uses [`TimelineFocus::Event`] with `num_context_events = 50` so the
    /// target is centred among surrounding context. Thread handling is
    /// `Automatic` so an event inside a thread still resolves without forcing
    /// the whole room into threaded mode.
    ///
    /// Repeated opens for the same (room, event) are served from the cache so
    /// navigating back is cheap.
    pub async fn event_timeline(
        &self,
        room_id: &str,
        target: matrix_sdk::ruma::OwnedEventId,
    ) -> Result<Arc<Timeline>> {
        let room_id_parsed = RoomId::parse(room_id)?;

        {
            let inner = self.inner.read().await;
            if let Some(timeline) = inner
                .event_timelines
                .get(&(room_id_parsed.clone(), target.clone()))
            {
                return Ok(timeline.clone());
            }
        }

        let rls = self
            .room_list_service()
            .await
            .context("RoomListService not initialized")?;
        rls.set_room_subscriptions(&[&room_id_parsed]).await;

        let room = rls
            .room(&room_id_parsed)
            .map_err(|e| anyhow::anyhow!("Failed to get room: {}", e))?;
        let timeline = Arc::new(
            room.timeline_builder()
                .with_focus(TimelineFocus::Event {
                    target: target.clone(),
                    num_context_events: 50,
                    thread_mode: TimelineEventFocusThreadMode::Automatic {
                        hide_threaded_events: true,
                    },
                })
                .build()
                .await?,
        );

        let mut inner = self.inner.write().await;
        inner
            .event_timelines
            .insert((room_id_parsed, target), timeline.clone());

        Ok(timeline)
    }

    /// Drop a cached event-focused timeline, e.g. when returning to live. The
    /// underlying matrix-sdk timeline is simply dropped (no server teardown is
    /// needed); a future open rebuilds it.
    pub async fn drop_event_timeline(
        &self,
        room_id: &str,
        target: &matrix_sdk::ruma::EventId,
    ) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let mut inner = self.inner.write().await;
        inner.event_timelines.remove(&(room_id, target.to_owned()));
        Ok(())
    }

    pub async fn paginate_backwards(&self, room_id: &str, limit: u16) -> Result<()> {
        let timeline = self.timeline(room_id).await?;
        timeline.paginate_backwards(limit).await?;
        Ok(())
    }

    pub async fn is_room_encrypted(&self, room_id: &str) -> Result<bool> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        Ok(room.encryption_settings().is_some())
    }

    pub async fn enable_encryption(&self, room_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.enable_encryption().await?;
        Ok(())
    }

    pub async fn set_room_name(&self, room_id: &str, name: String) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.set_name(name).await?;
        Ok(())
    }

    pub async fn set_room_topic(&self, room_id: &str, topic: String) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.set_room_topic(&topic).await?;
        Ok(())
    }

    pub async fn set_canonical_alias(&self, room_id: &str, alias: Option<String>) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        use matrix_sdk::ruma::RoomAliasId;
        use matrix_sdk::ruma::events::room::canonical_alias::RoomCanonicalAliasEventContent;

        let mut content = room
            .get_state_event_static::<RoomCanonicalAliasEventContent>()
            .await?
            .and_then(|e| e.deserialize().ok())
            .and_then(|e| {
                e.as_sync()
                    .and_then(|s| s.as_original().map(|o| o.content.clone()))
                    .or_else(|| e.as_stripped().map(|s| s.content.clone()))
            })
            .unwrap_or_else(RoomCanonicalAliasEventContent::new);

        content.alias = alias
            .filter(|s| !s.is_empty())
            .map(|s| RoomAliasId::parse(s).map(|a| a.to_owned()))
            .transpose()?;

        Ok(())
    }

    pub async fn set_room_history_visibility(
        &self,
        room_id: &str,
        history_visibility: matrix_sdk::ruma::events::room::history_visibility::HistoryVisibility,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        use matrix_sdk::ruma::events::room::history_visibility::RoomHistoryVisibilityEventContent;
        let content = RoomHistoryVisibilityEventContent::new(history_visibility);
        room.send_state_event(content).await?;
        Ok(())
    }

    pub async fn update_room_aliases(
        &self,
        room_id: &str,
        canonical_alias: Option<String>,
        alt_aliases: Vec<String>,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        use matrix_sdk::ruma::RoomAliasId;
        use matrix_sdk::ruma::events::room::canonical_alias::RoomCanonicalAliasEventContent;

        let mut content = room
            .get_state_event_static::<RoomCanonicalAliasEventContent>()
            .await?
            .and_then(|e| e.deserialize().ok())
            .and_then(|e| {
                e.as_sync()
                    .and_then(|s| s.as_original().map(|o| o.content.clone()))
                    .or_else(|| e.as_stripped().map(|s| s.content.clone()))
            })
            .unwrap_or_else(RoomCanonicalAliasEventContent::new);

        content.alias = canonical_alias
            .filter(|s| !s.is_empty())
            .map(|s| RoomAliasId::parse(s).map(|a| a.to_owned()))
            .transpose()?;

        content.alt_aliases = alt_aliases
            .into_iter()
            .map(|s| RoomAliasId::parse(s).map(|a| a.to_owned()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    pub async fn get_room_visibility(
        &self,
        room_id: &str,
    ) -> Result<matrix_sdk::ruma::api::client::room::Visibility> {
        let room_id_parsed = RoomId::parse(room_id).map_err(|e| anyhow::anyhow!(e))?;
        let client = self.client().await;
        let request =
            matrix_sdk::ruma::api::client::directory::get_room_visibility::v3::Request::new(
                room_id_parsed,
            );
        let response = client.send(request).await?;
        Ok(response.visibility)
    }

    pub async fn set_room_visibility(
        &self,
        room_id: &str,
        visibility: matrix_sdk::ruma::api::client::room::Visibility,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id).map_err(|e| anyhow::anyhow!(e))?;
        let client = self.client().await;
        let request =
            matrix_sdk::ruma::api::client::directory::set_room_visibility::v3::Request::new(
                room_id_parsed,
                visibility,
            );
        client.send(request).await?;
        Ok(())
    }

    pub async fn get_room_join_rule(
        &self,
        room_id: &str,
    ) -> Result<matrix_sdk::ruma::events::room::join_rules::JoinRule> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        Ok(room
            .join_rule()
            .unwrap_or(matrix_sdk::ruma::events::room::join_rules::JoinRule::Invite))
    }

    pub async fn set_room_join_rule(
        &self,
        room_id: &str,
        join_rule: matrix_sdk::ruma::events::room::join_rules::JoinRule,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        use matrix_sdk::ruma::events::room::join_rules::RoomJoinRulesEventContent;
        let content = RoomJoinRulesEventContent::new(join_rule);
        room.send_state_event(content).await?;
        Ok(())
    }

    pub async fn upload_room_avatar(&self, room_id: &str, data: Vec<u8>, mime: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        let content_type = mime.parse::<mime::Mime>()?;
        room.upload_avatar(&content_type, data, None).await?;
        Ok(())
    }

    pub async fn leave_room(&self, room_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.leave().await?;
        Ok(())
    }

    pub async fn get_room_permalink(&self, room_id: &str) -> Result<String> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        let permalink = room.matrix_to_permalink().await?;
        Ok(permalink.to_string())
    }

    pub async fn get_room_event_permalink(
        &self,
        room_id: &str,
        event_id: &matrix_sdk::ruma::EventId,
    ) -> Result<String> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        let permalink = room.matrix_to_event_permalink(event_id.to_owned()).await?;
        Ok(permalink.to_string())
    }

    pub async fn forget_room(&self, room_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.forget().await?;
        Ok(())
    }

    pub async fn get_room_power_levels(
        &self,
        room_id: &str,
    ) -> Result<(i64, HashMap<matrix_sdk::ruma::OwnedUserId, i64>)> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        let power_levels = room.power_levels().await?;

        let users = room.users_with_power_levels().await;
        // Also add users who have the default power level but are members
        // To avoid listing thousands of users in large rooms, maybe we only list members if the room is small?
        // Actually, let's just use what's in the power levels event first.
        // If the user wants to promote someone else, they can search for them.
        Ok((power_levels.users_default.into(), users))
    }

    pub async fn update_user_power_level(
        &self,
        room_id: &str,
        user_id: &str,
        level: i64,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        let int_level = matrix_sdk::ruma::Int::new(level)
            .ok_or_else(|| anyhow::anyhow!("Invalid power level"))?;
        room.update_power_levels(vec![(&user_id_parsed, int_level)])
            .await?;
        Ok(())
    }

    pub async fn update_room_power_level_settings(
        &self,
        room_id: &str,
        powers: RoomPowerLevelChanges,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;

        let mut changes = RoomPowerLevelChanges::new();
        changes.ban = powers.ban;
        changes.invite = powers.invite;
        changes.kick = powers.kick;
        changes.redact = powers.redact;
        changes.events_default = powers.events_default;
        changes.room_name = powers.room_name;
        changes.room_topic = powers.room_topic;
        changes.room_avatar = powers.room_avatar;

        room.apply_power_level_changes(changes).await?;
        Ok(())
    }

    pub async fn invite_user(&self, room_id: &str, user_id: &str) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.invite_user_by_id(&user_id_parsed).await?;
        Ok(())
    }

    pub async fn kick_user(
        &self,
        room_id: &str,
        user_id: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.kick_user(&user_id_parsed, reason.as_deref()).await?;
        Ok(())
    }

    pub async fn ban_user(
        &self,
        room_id: &str,
        user_id: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let user_id_parsed = matrix_sdk::ruma::UserId::parse(user_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        room.ban_user(&user_id_parsed, reason.as_deref()).await?;
        Ok(())
    }

    pub async fn join_room(&self, room_id: &RoomId) -> Result<()> {
        let client = self.client().await;
        if let Some(room) = client.get_room(room_id) {
            room.join().await?;
        } else {
            // If the room is unknown, try joining by ID directly
            client.join_room_by_id(room_id).await?;
        }
        Ok(())
    }

    /// Resolve a room alias to its canonical room ID via the homeserver.
    /// Used when opening a permalink that targets a room by alias
    /// (`#room:server`) rather than an ID.
    pub async fn resolve_room_alias(
        &self,
        alias: &matrix_sdk::ruma::RoomAliasId,
    ) -> Result<OwnedRoomId> {
        let client = self.client().await;
        let response = client.resolve_room_alias(alias).await?;
        Ok(response.room_id)
    }

    pub async fn get_room_members(&self, room_id: &str) -> Result<Vec<RoomMemberInfo>> {
        let room_id_parsed = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id_parsed).context("Room not found")?;
        let members = room.members(matrix_sdk::RoomMemberships::ACTIVE).await?;
        let member_infos = members
            .into_iter()
            .map(|m| RoomMemberInfo {
                user_id: m.user_id().to_string(),
                display_name: m.display_name().map(|s| s.to_string()),
                avatar_url: m.avatar_url().map(|u| u.to_string()),
            })
            .collect();
        Ok(member_infos)
    }
}
