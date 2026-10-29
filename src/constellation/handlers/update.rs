use crate::settings;
use crate::{Constellation, Message};
use cosmic::{Action, Task};

#[allow(clippy::result_large_err)]
impl Constellation {
    pub fn handle_update(&mut self, message: Message) -> Task<Action<Message>> {
        let task = self
            .handle_session_message(message)
            .or_else(|msg| self.handle_settings_message(msg))
            .or_else(|msg| self.handle_search_message(msg))
            .or_else(|msg| self.handle_room_and_space_message(msg))
            .or_else(|msg| self.handle_composer_and_media_message(msg))
            .unwrap_or_else(|msg| self.handle_timeline_and_core_message(msg));

        if self.space_nav_dirty {
            self.rebuild_space_nav_model();
        }
        task
    }

    fn handle_session_message(
        &mut self,
        message: Message,
    ) -> Result<Task<Action<Message>>, Message> {
        match message {
            Message::LoginHomeserverChanged(homeserver) => Ok({
                self.login_homeserver = homeserver;
                Task::none()
            }),
            Message::LoginUsernameChanged(username) => Ok({
                self.login_username = username;
                Task::none()
            }),
            Message::LoginPasswordChanged(password) => Ok({
                self.login_password = password;
                Task::none()
            }),
            Message::SubmitLogin => Ok(self.handle_submit_login()),
            Message::LoginFinished(res) => Ok(self.handle_login_finished(res)),
            Message::ToggleLoginMode => Ok(self.handle_toggle_login_mode()),
            Message::SubmitRegister => Ok(self.handle_submit_register()),
            Message::RegisterFinished(res) => Ok(self.handle_register_finished(res)),
            Message::SubmitOidcLogin => Ok(self.handle_submit_oidc_login()),
            Message::CancelOidcLogin => Ok(self.handle_cancel_oidc_login()),
            Message::OidcLoginStarted(res) => Ok(self.handle_oidc_login_started(res)),
            Message::OidcCallback(url) => Ok(self.handle_oidc_callback(url)),
            Message::StartQrLogin => Ok(self.handle_start_qr_login()),
            Message::CancelQrLogin => Ok(self.handle_cancel_qr_login()),
            Message::QrLoginProgress(progress) => Ok(self.handle_qr_login_progress(progress)),
            Message::QrCheckCodeChanged(code) => Ok(self.handle_qr_check_code_changed(code)),
            Message::SubmitQrCheckCode => Ok(self.handle_submit_qr_check_code()),
            Message::Logout => Ok(self.handle_logout()),
            Message::LogoutFinished => Ok(self.handle_logout_finished()),
            Message::SessionVerificationNeeded(target_device_id) => Ok({
                if self.session_verification_prompt.is_none() {
                    self.session_verification_prompt =
                        Some(crate::constellation::SessionVerificationPrompt { target_device_id });
                }
                Task::none()
            }),
            Message::DismissSessionVerificationPrompt => Ok({
                self.session_verification_prompt = None;
                Task::none()
            }),
            Message::IdentityViolationDetected(user_id) => Ok({
                if !self.identity_violations.contains(&user_id) {
                    self.identity_violations.push(user_id.clone());
                    let summary = crate::fl!("identity-violation-notification-summary");
                    let body = crate::fl!(
                        "identity-violation-notification-body",
                        user = user_id.as_str()
                    );
                    if let Ok(handle) = tokio::runtime::Handle::try_current() {
                        handle.spawn(async move {
                            let _ = crate::constellation::state::build_verification_notification(
                                &summary, &body,
                            )
                            .show_async()
                            .await;
                        });
                    } else {
                        let _ = crate::constellation::state::build_verification_notification(
                            &summary, &body,
                        )
                        .show();
                    }
                }
                Task::none()
            }),
            Message::DismissIdentityViolation(user_id) => Ok({
                self.identity_violations.retain(|id| id != &user_id);
                Task::none()
            }),
            msg => Err(msg),
        }
    }

    fn handle_settings_message(
        &mut self,
        message: Message,
    ) -> Result<Task<Action<Message>>, Message> {
        match message {
            Message::OpenSettings(panel) => Ok(self.handle_open_settings(panel)),
            Message::CloseSettings => Ok(self.handle_close_settings()),
            Message::SettingsBack => Ok({
                if self.settings_stack.len() > 1 {
                    self.settings_stack.pop();
                }
                Task::none()
            }),
            Message::UserSettings(crate::settings::user::Message::OpenPanel(panel)) => {
                Ok(self.handle_open_settings(panel))
            }
            Message::UserSettings(msg) => Ok({
                let was_done = matches!(msg, crate::settings::user::Message::DismissVerification)
                    || matches!(
                        self.user_settings.verification_ui_state,
                        crate::settings::user::VerificationUIState::Done
                    );
                let task = self.user_settings.update(msg, &self.matrix);
                if was_done
                    || matches!(
                        self.user_settings.verification_ui_state,
                        crate::settings::user::VerificationUIState::Done
                    )
                {
                    self.session_verification_prompt = None;
                }
                task
            }),
            Message::RoomSettings(msg) => Ok(match msg {
                settings::room::Message::OpenPanel(panel) => self.handle_open_settings(panel),
                msg => self.room_settings.update(msg, &self.matrix),
            }),
            Message::SpaceSettings(msg) => Ok(match msg {
                settings::space::Message::OpenPanel(panel) => self.handle_open_settings(panel),
                msg => self.space_settings.update(msg, &self.matrix),
            }),
            Message::AppSettings(msg) => Ok(match msg {
                settings::app::Message::ClearCache => {
                    self.media_cache.clear();
                    self.og_cache.clear();
                    Task::none()
                }
                settings::app::Message::OpenPanel(panel) => self.handle_open_settings(panel),
                settings::app::Message::OpenShortcuts => {
                    self.handle_open_settings(crate::SettingsPanel::Shortcuts)
                }
                msg => self.app_settings.update(msg),
            }),
            Message::Shortcuts(msg) => Ok(self.shortcuts.update(msg)),
            Message::ShortcutsSaved => Ok(self.handle_shortcuts_saved()),
            Message::ShortcutTriggered(action) => Ok(self.handle_shortcut_triggered(action)),
            Message::AppSettingChanged => Ok(self.handle_app_setting_changed()),
            msg => Err(msg),
        }
    }

    fn handle_search_message(
        &mut self,
        message: Message,
    ) -> Result<Task<Action<Message>>, Message> {
        match message {
            Message::ToggleSearch => Ok(self.handle_toggle_search()),
            Message::SearchQueryChanged(query) => Ok(self.handle_search_query_changed(query)),
            Message::SubmitSearch => Ok(self.handle_submit_search()),
            Message::SearchApplySuggestion(replacement) => {
                Ok(self.handle_search_apply_suggestion(replacement))
            }
            Message::SearchDismissSuggestions => Ok(self.handle_search_dismiss_suggestions()),
            Message::PublicSearchResults(generation, res) => {
                Ok(self.handle_public_search_results(generation, res))
            }
            Message::MessageSearchResults(generation, res) => {
                Ok(self.handle_message_search_results(generation, res))
            }
            Message::LoadMoreMessageSearch => Ok(self.handle_load_more_message_search()),
            Message::MessageSearchMoreResults(res) => {
                Ok(self.handle_message_search_more_results(res))
            }
            Message::GlobalMessageSearchResults(generation, res) => {
                Ok(self.handle_global_message_search_results(generation, res))
            }
            Message::SetGlobalSearchScope(scope) => Ok(self.handle_set_global_search_scope(scope)),
            msg => Err(msg),
        }
    }

    fn handle_room_and_space_message(
        &mut self,
        message: Message,
    ) -> Result<Task<Action<Message>>, Message> {
        match message {
            Message::RoomSelected(room_id) => Ok(self.handle_room_selected(room_id)),
            Message::TabActivated(entity) => Ok(self.handle_tab_activated(entity)),
            Message::TabClosed(entity) => Ok(self.handle_tab_closed(entity)),
            Message::CloseTab(tab) => Ok(self.handle_close_tab(&tab)),
            Message::CloseRoom(room_id) => Ok(self.handle_close_room(room_id)),
            Message::CopyActiveRoomLink => Ok(self.handle_copy_active_room_link()),
            Message::CloseActiveRoom => Ok(self.handle_close_active_room()),
            Message::ToggleCreateRoom => Ok(self.handle_toggle_create_room()),
            Message::ToggleCreateSpace => Ok(self.handle_toggle_create_space()),
            Message::ToggleInviteToSpace => Ok(self.handle_toggle_invite_to_space()),
            Message::InviteToSpaceIdChanged(id) => Ok(self.handle_invite_to_space_id_changed(id)),
            Message::InviteToSpace => Ok(self.handle_invite_to_space()),
            Message::SpaceUserInvited(res) => Ok(self.handle_space_user_invited(res)),
            Message::ToggleInviteToRoom => Ok(self.handle_toggle_invite_to_room()),
            Message::InviteToRoomIdChanged(id) => Ok(self.handle_invite_to_room_id_changed(id)),
            Message::InviteToRoom => Ok(self.handle_invite_to_room()),
            Message::RoomUserInvited(res) => Ok(self.handle_room_user_invited(res)),
            Message::NewRoomNameChanged(name) => Ok(self.handle_new_room_name_changed(name)),
            Message::CreateRoom(name) => Ok(self.handle_create_room(name)),
            Message::RoomCreated(res) => Ok(self.handle_room_created(res)),
            Message::CreateSpace(name) => Ok(self.handle_create_space(name)),
            Message::SpaceCreated(res) => Ok(self.handle_space_created(res)),
            Message::SelectSpace(space_id) => Ok(self.handle_select_space(space_id)),
            Message::CloseSpaceSwitcher => Ok(if self.is_room_list_open {
                self.handle_close_space_switcher()
            } else {
                Task::none()
            }),
            Message::SpaceChildrenFetched(space_id, res) => {
                Ok(self.handle_space_children_fetched(space_id, res))
            }
            Message::SpaceFilterUpdated => Ok({
                self.update_filtered_rooms();
                Task::none()
            }),
            Message::JoinRoom(room_id) => Ok(self.handle_join_room(room_id)),
            Message::RoomJoined(res) => Ok(self.handle_room_joined(res)),
            Message::JoinCall => Ok(self.handle_join_call()),
            Message::LeaveCall => Ok(self.handle_leave_call()),
            Message::CallJoined(res) => Ok(self.handle_call_joined(res)),
            Message::CallLeft(res) => Ok(self.handle_call_left(res)),
            Message::ToggleMembersPanel => Ok(self.handle_toggle_members_panel()),
            Message::MembersFetched(res) => Ok(self.handle_members_fetched(res)),
            Message::TogglePinnedPanel => Ok(self.handle_toggle_pinned_panel()),
            Message::PinnedEventsFetched(res) => Ok(self.handle_pinned_events_fetched(res)),
            Message::ToggleActiveThreadsPanel => Ok(self.handle_toggle_active_threads_panel()),
            Message::ActiveThreadsFetched(res) => Ok(self.handle_active_threads_fetched(res)),
            Message::UnpinMessage(event_id) => Ok(self.handle_unpin_message(event_id)),
            Message::NewRoomIsVideoChanged(is_video) => Ok({
                self.new_room_is_video = is_video;
                Task::none()
            }),
            msg => Err(msg),
        }
    }

    fn handle_composer_and_media_message(
        &mut self,
        message: Message,
    ) -> Result<Task<Action<Message>>, Message> {
        match message {
            Message::ComposerChanged(text) => Ok(self.handle_composer_changed(text)),
            Message::ComposerAction(action) => Ok(self.handle_composer_action(action)),
            Message::TogglePreview => Ok({
                self.composer_is_preview = !self.composer_is_preview;
                if self.composer_is_preview {
                    self.fetch_missing_og_previews().unwrap_or_else(Task::none)
                } else {
                    Task::none()
                }
            }),
            Message::SendMessage => Ok(self.handle_send_message()),
            Message::ShareLocation => Ok(self.handle_share_location()),
            Message::LocationRetrieved(res) => Ok(self.handle_location_retrieved(res)),
            Message::MessageSent(res) => Ok(self.handle_message_sent(res)),
            Message::MessageEdited(res) => Ok(self.handle_message_edited(res)),
            Message::MessageRedacted(res) => Ok(self.handle_message_redacted(res)),
            Message::StartEdit(item_id) => Ok(self.handle_start_edit(item_id)),
            Message::CancelEdit => Ok(self.handle_cancel_edit()),
            Message::RedactMessage(item_id) => Ok(self.handle_redact_message(item_id)),
            Message::CopyMessageLink(item_id) => Ok(self.handle_copy_message_link(item_id)),
            Message::CopyRoomLink(room_id) => Ok(self.handle_copy_room_link(room_id)),
            Message::CopyToClipboard(res) => Ok(self.handle_copy_to_clipboard(res)),
            Message::AddAttachment => Ok(self.handle_add_attachment()),
            Message::AttachmentsSelected(paths) => Ok(self.handle_attachments_selected(paths)),
            Message::DndFileTransfer(key) => Ok(self.handle_dnd_file_transfer(key)),
            Message::DndFileTransferFinished(res) => {
                Ok(self.handle_dnd_file_transfer_finished(res))
            }
            Message::DndDataReceived(mime, data) => Ok(self.handle_dnd_data_received(mime, data)),
            Message::RemoveAttachment(index) => Ok(self.handle_remove_attachment(index)),
            Message::AttachmentSent(path, res) => Ok(self.handle_attachment_sent(path, res)),
            Message::OpenReactionPicker(item_id) => Ok(self.handle_open_reaction_picker(item_id)),
            Message::EmojiSearchQueryChanged(query) => {
                Ok(self.handle_emoji_search_query_changed(query))
            }
            Message::SelectEmojiGroup(group) => Ok(self.handle_select_emoji_group(group)),
            Message::ToggleEmojiPicker => Ok(self.handle_toggle_emoji_picker()),
            Message::EmojiPickerSelected(emoji) => Ok(self.handle_emoji_picker_selected(emoji)),
            Message::InsertEmoji(emoji) => Ok(self.handle_insert_emoji(emoji)),
            Message::SelectEmojiPickerTab(tab) => Ok({
                self.emoji_picker_tab = tab;
                Task::none()
            }),
            Message::SendSticker {
                body,
                url,
                width,
                height,
            } => Ok(self.handle_send_sticker(body, url, width, height)),
            Message::StickerSent(res) => Ok({
                if let Err(e) = res {
                    self.set_error(format!("Failed to send sticker: {e}"));
                }
                Task::none()
            }),
            Message::LoadRoomImagePacks(room_id) => Ok(self.handle_load_room_image_packs(room_id)),
            Message::RoomImagePacksLoaded(room_id, res) => {
                Ok(self.handle_room_image_packs_loaded(room_id, res))
            }
            Message::LoadAccountImagePacks => Ok(self.handle_load_account_image_packs()),
            Message::AccountImagePacksLoaded(res) => {
                Ok(self.handle_account_image_packs_loaded(res))
            }
            Message::ToggleReaction(item_id, key) => Ok(self.handle_toggle_reaction(item_id, key)),
            Message::ReactionToggled(res) => Ok(self.handle_reaction_toggled(res)),
            Message::FetchMedia(source) => Ok(self.handle_fetch_media(source)),
            Message::MediaFetched(mxc_url, res) => Ok(self.handle_media_fetched(mxc_url, res)),
            Message::MediaFetchedBatch(batch) => Ok(self.handle_media_fetched_batch(batch)),
            Message::FetchOgPreview(url) => Ok(self.handle_fetch_og_preview(url)),
            Message::OgPreviewFetched(url, res) => Ok(self.handle_og_preview_fetched(url, res)),
            Message::SaveMedia { source, filename } => Ok(self.handle_save_media(source, filename)),
            Message::MediaSaved(res) => Ok(self.handle_media_saved(res)),
            #[cfg(feature = "video-player")]
            Message::PlayVideo {
                source,
                mxc_url,
                filename,
                autoplay,
            } => Ok(self.handle_play_video(source, mxc_url, filename, autoplay)),
            #[cfg(feature = "video-player")]
            Message::VideoReady(mxc_url, res) => Ok(self.handle_video_ready(mxc_url, res)),
            #[cfg(feature = "video-player")]
            Message::ToggleVideoPause(mxc_url) => Ok({
                if let Some(entry) = self.video_cache.get_mut(&mxc_url) {
                    entry.video.set_paused(!entry.video.paused());
                }
                Task::none()
            }),
            #[cfg(feature = "video-player")]
            Message::VideoPlaybackError(error) => Ok({
                self.set_error(crate::fl!("error-video-playback", error = error).to_string());
                Task::none()
            }),
            Message::OpenImage(handle) => Ok({
                self.fullscreen_image = Some(handle);
                Task::none()
            }),
            Message::CloseImage => Ok(self.handle_close_image()),
            msg => Err(msg),
        }
    }

    fn handle_timeline_and_core_message(&mut self, message: Message) -> Task<Action<Message>> {
        match message {
            Message::EngineReady(res) => self.handle_engine_ready(res),
            Message::UserReady(user_id, sync_res) => self.handle_user_ready(user_id, sync_res),
            Message::Matrix(event) => self.handle_matrix_event(event),
            Message::MatrixThreadDiff(root_id, diff) => {
                self.handle_timeline_diff(diff, true, Some(root_id))
            }
            Message::MatrixThreadReset(root_id) => self.handle_matrix_thread_reset(root_id),
            Message::MatrixThreadInitFinished(root_id) => {
                self.handle_matrix_thread_init_finished(root_id)
            }
            Message::OpenThread(root_id) => self.handle_open_thread(root_id),
            Message::StartReply(item_id) => self.handle_start_reply(item_id),
            Message::CancelReply => self.handle_cancel_reply(),
            Message::CloseThread => self.handle_close_thread(),
            Message::LoadMoreFinished(res) => self.handle_load_more_finished(res),
            Message::TimelineScrolled(viewport, is_thread) => {
                self.handle_timeline_scrolled(viewport, is_thread)
            }
            Message::TimelineMeasured {
                is_thread,
                generation,
                viewport_width,
                viewport_height,
                content_height,
                rows,
            } => self.handle_timeline_measured(
                is_thread,
                generation,
                viewport_width,
                viewport_height,
                content_height,
                rows,
            ),
            Message::DmRoomResolved(res) => match res {
                Ok(room_id) => {
                    let room_id_arc: std::sync::Arc<str> = room_id.as_str().into();
                    self.handle_update(Message::RoomSelected(room_id_arc))
                }
                Err(e) => {
                    self.set_error(crate::fl!("error-failed-start-dm", error = e));
                    Task::none()
                }
            },
            Message::DismissError => self.handle_dismiss_error(),
            Message::NoOp => Task::none(),
            Message::OpenMatrixLink(raw) => self.open_matrix_link(raw),
            Message::ToggleOpenLink => self.handle_toggle_open_link(),
            Message::OpenLinkTextChanged(text) => self.handle_open_link_text_changed(text),
            Message::SubmitOpenLink(text) => self.handle_submit_open_link(text),
            Message::RoomAliasResolved(res) => self.handle_room_alias_resolved(res),
            Message::SelectionMove(delta) => self.handle_selection_move(delta),
            Message::SelectionCommit => self.handle_selection_commit(),
            Message::SelectionCancel => self.handle_selection_cancel(),
            Message::PaneResized(event) => self.handle_pane_resized(event),
            Message::JumpToMessage(event_id) => self.handle_jump_to_message(event_id),
            Message::JumpToMessageOrLoadContext(event_id) => {
                self.handle_jump_to_message_or_load_context(event_id)
            }
            Message::SetPendingEventFocus(event_id) => {
                self.set_pending_event_focus(event_id);
                Task::none()
            }
            Message::OpenRoomEvent { room_id, event_id } => {
                self.handle_open_room_event(room_id, event_id)
            }
            Message::PermalinkEventRoomChecked {
                room_id,
                event_id,
                is_joined,
            } => self.handle_permalink_event_room_checked(room_id, event_id, is_joined),
            Message::LoadEventContext(event_id) => self.handle_load_event_context(event_id),
            Message::EventContextLoaded(room_id, event_id, res) => {
                self.handle_event_context_loaded(room_id, event_id, res)
            }
            Message::ReturnToLive => self.handle_return_to_live(),
            Message::OpenUrl(url) => Task::perform(
                async move {
                    let _ = open::that(url);
                },
                |_| Action::from(Message::NoOp),
            ),
            Message::RestoreTick => self.handle_restore_tick(),
            Message::ThreadInfoUpdated {
                room_id,
                root_id,
                unread,
            } => self.handle_thread_info_updated(room_id, root_id, unread),
            _ => Task::none(),
        }
    }
}
