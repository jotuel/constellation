use crate::settings;
use crate::{AuthFlow, Constellation, Message};
use cosmic::{Action, Task};

impl Constellation {
    pub fn handle_update(&mut self, message: Message) -> Task<Action<Message>> {
        let task = match message {
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
                content_height,
                rows,
            } => self.handle_timeline_measured(
                is_thread,
                generation,
                viewport_width,
                content_height,
                rows,
            ),
            Message::RoomSelected(room_id) => self.handle_room_selected(room_id),
            Message::RoomTabActivated(entity) => self.handle_room_tab_activated(entity),
            Message::RoomTabClosed(entity) => self.handle_room_tab_closed(entity),
            Message::CloseRoom(room_id) => self.handle_close_room(room_id),
            Message::CopyActiveRoomLink => self.handle_copy_active_room_link(),
            Message::CloseActiveRoom => self.handle_close_active_room(),
            Message::ComposerChanged(text) => self.handle_composer_changed(text),
            Message::ComposerAction(action) => self.handle_composer_action(action),
            Message::TogglePreview => {
                self.composer_is_preview = !self.composer_is_preview;
                if self.composer_is_preview {
                    self.fetch_missing_og_previews().unwrap_or_else(Task::none)
                } else {
                    Task::none()
                }
            }
            Message::SendMessage => self.handle_send_message(),
            Message::ShareLocation => self.handle_share_location(),
            Message::LocationRetrieved(res) => self.handle_location_retrieved(res),
            Message::MessageSent(res) => self.handle_message_sent(res),
            Message::MessageEdited(res) => self.handle_message_edited(res),
            Message::MessageRedacted(res) => self.handle_message_redacted(res),
            Message::StartEdit(item_id) => self.handle_start_edit(item_id),
            Message::CancelEdit => self.handle_cancel_edit(),
            Message::RedactMessage(item_id) => self.handle_redact_message(item_id),
            Message::CopyMessageLink(item_id) => self.handle_copy_message_link(item_id),
            Message::CopyRoomLink(room_id) => self.handle_copy_room_link(room_id),
            Message::CopyToClipboard(res) => self.handle_copy_to_clipboard(res),
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
            Message::AddAttachment => self.handle_add_attachment(),
            Message::AttachmentsSelected(paths) => self.handle_attachments_selected(paths),
            Message::DndFileTransfer(key) => self.handle_dnd_file_transfer(key),
            Message::DndFileTransferFinished(res) => self.handle_dnd_file_transfer_finished(res),
            Message::DndDataReceived(mime, data) => self.handle_dnd_data_received(mime, data),
            Message::RemoveAttachment(index) => self.handle_remove_attachment(index),
            Message::AttachmentSent(path, res) => self.handle_attachment_sent(path, res),
            Message::OpenReactionPicker(item_id) => self.handle_open_reaction_picker(item_id),
            Message::EmojiSearchQueryChanged(query) => self.handle_emoji_search_query_changed(query),
            Message::SelectEmojiGroup(group) => self.handle_select_emoji_group(group),
            Message::ToggleEmojiPicker => self.handle_toggle_emoji_picker(),
            Message::EmojiPickerSelected(emoji) => self.handle_emoji_picker_selected(emoji),
            Message::InsertEmoji(emoji) => self.handle_insert_emoji(emoji),
            Message::ToggleReaction(item_id, key) => self.handle_toggle_reaction(item_id, key),
            Message::ReactionToggled(res) => self.handle_reaction_toggled(res),
            Message::FetchMedia(source) => self.handle_fetch_media(source),
            Message::MediaFetched(mxc_url, res) => self.handle_media_fetched(mxc_url, res),
            Message::MediaFetchedBatch(batch) => self.handle_media_fetched_batch(batch),
            Message::FetchOgPreview(url) => self.handle_fetch_og_preview(url),
            Message::OgPreviewFetched(url, res) => self.handle_og_preview_fetched(url, res),
            Message::SaveMedia { source, filename } => self.handle_save_media(source, filename),
            Message::MediaSaved(res) => self.handle_media_saved(res),
            #[cfg(feature = "video-player")]
            Message::PlayVideo {
                source,
                mxc_url,
                filename,
                autoplay,
            } => self.handle_play_video(source, mxc_url, filename, autoplay),
            #[cfg(feature = "video-player")]
            Message::VideoReady(mxc_url, res) => self.handle_video_ready(mxc_url, res),
            #[cfg(feature = "video-player")]
            Message::ToggleVideoPause(mxc_url) => {
                if let Some(entry) = self.video_cache.get_mut(&mxc_url) {
                    entry.video.set_paused(!entry.video.paused());
                }
                Task::none()
            }
            #[cfg(feature = "video-player")]
            Message::VideoPlaybackError(error) => {
                self.set_error(crate::fl!("error-video-playback", error = error).to_string());
                Task::none()
            }
            Message::DismissError => self.handle_dismiss_error(),
            Message::ToggleCreateRoom => self.handle_toggle_create_room(),
            Message::ToggleCreateSpace => self.handle_toggle_create_space(),
            Message::ToggleInviteToSpace => self.handle_toggle_invite_to_space(),
            Message::InviteToSpaceIdChanged(id) => self.handle_invite_to_space_id_changed(id),
            Message::InviteToSpace => self.handle_invite_to_space(),
            Message::SpaceUserInvited(res) => self.handle_space_user_invited(res),
            Message::ToggleInviteToRoom => self.handle_toggle_invite_to_room(),
            Message::InviteToRoomIdChanged(id) => self.handle_invite_to_room_id_changed(id),
            Message::InviteToRoom => self.handle_invite_to_room(),
            Message::RoomUserInvited(res) => self.handle_room_user_invited(res),
            Message::NewRoomNameChanged(name) => self.handle_new_room_name_changed(name),
            Message::CreateRoom(name) => self.handle_create_room(name),
            Message::RoomCreated(res) => self.handle_room_created(res),
            Message::CreateSpace(name) => self.handle_create_space(name),
            Message::SpaceCreated(res) => self.handle_space_created(res),
            Message::LoginHomeserverChanged(homeserver) => {
                self.login_homeserver = homeserver;
                Task::none()
            }
            Message::LoginUsernameChanged(username) => {
                self.login_username = username;
                Task::none()
            }
            Message::LoginPasswordChanged(password) => {
                self.login_password = password;
                Task::none()
            }
            Message::SubmitLogin => self.handle_submit_login(),
            Message::LoginFinished(res) => self.handle_login_finished(res),
            Message::ToggleLoginMode => self.handle_toggle_login_mode(),
            Message::SubmitRegister => self.handle_submit_register(),
            Message::RegisterFinished(res) => self.handle_register_finished(res),
            Message::SelectSpace(space_id) => self.handle_select_space(space_id),
            Message::CloseSpaceSwitcher => {
                if self.is_room_list_open {
                    self.handle_close_space_switcher()
                } else {
                    Task::none()
                }
            }
            Message::SpaceChildrenFetched(space_id, res) => {
                self.handle_space_children_fetched(space_id, res)
            }
            Message::SpaceFilterUpdated => {
                self.update_filtered_rooms();
                Task::none()
            }
            Message::NoOp => Task::none(),
            Message::SubmitOidcLogin => self.handle_submit_oidc_login(),
            Message::CancelOidcLogin => {
                self.auth_flow = AuthFlow::Idle;
                Task::none()
            }
            Message::OidcLoginStarted(res) => self.handle_oidc_login_started(res),
            Message::OidcCallback(url) => self.handle_oidc_callback(url),
            Message::OpenMatrixLink(raw) => self.open_matrix_link(raw),
            Message::ToggleOpenLink => self.handle_toggle_open_link(),
            Message::OpenLinkTextChanged(text) => self.handle_open_link_text_changed(text),
            Message::SubmitOpenLink(text) => self.handle_submit_open_link(text),
            Message::RoomAliasResolved(res) => self.handle_room_alias_resolved(res),
            Message::StartQrLogin => self.handle_start_qr_login(),
            Message::CancelQrLogin => self.handle_cancel_qr_login(),
            Message::QrLoginProgress(progress) => self.handle_qr_login_progress(progress),
            Message::QrCheckCodeChanged(code) => self.handle_qr_check_code_changed(code),
            Message::SubmitQrCheckCode => self.handle_submit_qr_check_code(),
            Message::JoinRoom(room_id) => self.handle_join_room(room_id),
            Message::RoomJoined(res) => self.handle_room_joined(res),
            Message::Logout => self.handle_logout(),
            Message::LogoutFinished => self.handle_logout_finished(),
            Message::OpenSettings(panel) => self.handle_open_settings(panel),
            Message::CloseSettings => self.handle_close_settings(),
            Message::UserSettings(msg) => self.user_settings.update(msg, &self.matrix),
            Message::RoomSettings(msg) => match msg {
                settings::room::Message::OpenPanel(panel) => self.handle_open_settings(panel),
                msg => self.room_settings.update(msg, &self.matrix),
            },
            Message::SpaceSettings(msg) => self.space_settings.update(msg, &self.matrix),
            Message::AppSettings(msg) => match msg {
                settings::app::Message::ClearCache => {
                    self.media_cache.clear();
                    self.og_cache.clear();
                    Task::none()
                }
                _ => self.app_settings.update(msg),
            },
            Message::Shortcuts(msg) => self.shortcuts.update(msg),
            Message::ShortcutsSaved => self.handle_shortcuts_saved(),
            Message::ShortcutTriggered(action) => self.handle_shortcut_triggered(action),
            Message::SelectionMove(delta) => self.handle_selection_move(delta),
            Message::SelectionCommit => self.handle_selection_commit(),
            Message::SelectionCancel => self.handle_selection_cancel(),
            Message::PaneResized(event) => self.handle_pane_resized(event),
            Message::AppSettingChanged => self.handle_app_setting_changed(),
            Message::ToggleSearch => self.handle_toggle_search(),
            Message::SearchQueryChanged(query) => self.handle_search_query_changed(query),
            Message::PublicSearchResults(generation, res) => {
                self.handle_public_search_results(generation, res)
            }
            Message::MessageSearchResults(generation, res) => {
                self.handle_message_search_results(generation, res)
            }
            Message::LoadMoreMessageSearch => self.handle_load_more_message_search(),
            Message::MessageSearchMoreResults(res) => self.handle_message_search_more_results(res),
            Message::GlobalMessageSearchResults(generation, res) => {
                self.handle_global_message_search_results(generation, res)
            }
            Message::SetGlobalSearchScope(scope) => self.handle_set_global_search_scope(scope),
            Message::NewRoomIsVideoChanged(is_video) => {
                self.new_room_is_video = is_video;
                Task::none()
            }
            Message::JumpToMessage(event_id) => self.handle_jump_to_message(event_id),
            Message::JumpToMessageOrLoadContext(event_id) => {
                self.handle_jump_to_message_or_load_context(event_id)
            }
            Message::SetPendingEventFocus(event_id) => {
                // Set the event focus so the next `TimelineInitFinished`
                // scrolls to it (or builds an event-focused timeline). Fired by
                // `OpenRoomEvent` as a follow-up to `RoomSelected`: the
                // `RoomSelected` handler clears `pending_event_focus`, so the
                // focus must be set *after* the room switch in the same batch
                // (see `Message::OpenRoomEvent` and
                // `test_room_selected_clears_event_focus`).
                self.pending_event_focus = Some(event_id);
                Task::none()
            }
            Message::OpenRoomEvent { room_id, event_id } => {
                self.handle_open_room_event(room_id, event_id)
            }
            Message::LoadEventContext(event_id) => self.handle_load_event_context(event_id),
            Message::EventContextLoaded(event_id, res) => {
                self.handle_event_context_loaded(event_id, res)
            }
            Message::ReturnToLive => self.handle_return_to_live(),
            Message::JoinCall => self.handle_join_call(),
            Message::LeaveCall => self.handle_leave_call(),
            Message::CallJoined(res) => self.handle_call_joined(res),
            Message::CallLeft(res) => self.handle_call_left(res),
            Message::OpenUrl(url) => Task::perform(
                async move {
                    let _ = open::that(url);
                },
                |_| Action::from(Message::NoOp),
            ),
            Message::OpenImage(handle) => {
                self.fullscreen_image = Some(handle);
                Task::none()
            }
            Message::CloseImage => self.handle_close_image(),
            Message::RestoreTick => self.handle_restore_tick(),
            Message::ToggleMembersPanel => self.handle_toggle_members_panel(),
            Message::MembersFetched(res) => self.handle_members_fetched(res),
            Message::TogglePinnedPanel => self.handle_toggle_pinned_panel(),
            Message::PinnedEventsFetched(res) => self.handle_pinned_events_fetched(res),
            Message::UnpinMessage(event_id) => self.handle_unpin_message(event_id),
        };
        if self.space_nav_dirty {
            self.rebuild_space_nav_model();
        }
        task
    }
}
