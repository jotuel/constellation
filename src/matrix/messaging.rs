use super::*;

impl MatrixEngine {
    pub async fn send_message(
        &self,
        room_id: &str,
        body: String,
        html_body: Option<String>,
    ) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;

        let content = if let Some(html) = html_body {
            RoomMessageEventContent::text_html(body, html)
        } else {
            RoomMessageEventContent::text_plain(body)
        };

        room.send(content).await?;
        Ok(())
    }

    pub async fn send_location(&self, room_id: &str, body: String, geo_uri: String) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;

        use matrix_sdk::ruma::events::room::message::{LocationMessageEventContent, MessageType};

        let content = RoomMessageEventContent::new(MessageType::Location(
            LocationMessageEventContent::new(body, geo_uri),
        ));

        room.send(content).await?;
        Ok(())
    }

    pub async fn send_reply(
        &self,
        room_id: &str,
        reply_to_event_id: &matrix_sdk::ruma::EventId,
        reply_to_sender: &matrix_sdk::ruma::UserId,
        body: String,
        html_body: Option<String>,
    ) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;

        let content = if let Some(html) = html_body {
            RoomMessageEventContent::text_html(body, html)
        } else {
            RoomMessageEventContent::text_plain(body)
        };

        let reply = content.make_for_thread(
            matrix_sdk::ruma::events::room::message::ReplyMetadata::new(
                reply_to_event_id,
                reply_to_sender,
                None,
            ),
            matrix_sdk::ruma::events::room::message::ReplyWithinThread::No,
            matrix_sdk::ruma::events::room::message::AddMentions::Yes,
        );

        room.send(reply).await?;
        Ok(())
    }

    pub async fn send_threaded_message(
        &self,
        room_id: &str,
        root_event_id: &matrix_sdk::ruma::EventId,
        sender: Option<&String>,
        body: String,
        html_body: Option<String>,
    ) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;

        let content = if let Some(html) = html_body {
            RoomMessageEventContent::text_html(body, html)
        } else {
            RoomMessageEventContent::text_plain(body)
        };

        let sender_id = if let Some(s) = sender {
            UserId::parse(s)?
        } else {
            client.user_id().context("No user id")?.to_owned()
        };

        let threaded_message = content.make_for_thread(
            matrix_sdk::ruma::events::room::message::ReplyMetadata::new(
                root_event_id,
                &sender_id,
                None,
            ),
            matrix_sdk::ruma::events::room::message::ReplyWithinThread::Yes,
            matrix_sdk::ruma::events::room::message::AddMentions::Yes,
        );

        room.send(threaded_message).await?;
        Ok(())
    }

    pub async fn send_attachment(&self, room_id: &str, path: &std::path::PathBuf) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;

        let metadata = tokio::fs::metadata(path).await?;
        let mut max_size = 50 * 1024 * 1024; // 50 MB default
        if let Ok(size) = client.load_or_fetch_max_upload_size().await {
            max_size = u64::from(size);
        }
        if metadata.len() > max_size {
            anyhow::bail!("File size exceeds maximum allowed upload size");
        }

        let data = tokio::fs::read(path).await?;
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mime_type = mime_guess::from_path(path).first_or_octet_stream();
        let config = matrix_sdk::attachment::AttachmentConfig::new();

        room.send_attachment(&filename, &mime_type, data, config)
            .await?;
        Ok(())
    }

    pub async fn edit_message(
        &self,
        room_id: &str,
        item_id: &TimelineEventItemId,
        body: String,
        html_body: Option<String>,
    ) -> Result<()> {
        let timeline = self.timeline(room_id).await?;
        let content = if let Some(html) = html_body {
            RoomMessageEventContent::text_html(body, html)
        } else {
            RoomMessageEventContent::text_plain(body)
        };
        timeline
            .edit(item_id, EditedContent::RoomMessage(content.into()))
            .await?;
        Ok(())
    }

    pub async fn redact_message(
        &self,
        room_id: &str,
        item_id: &TimelineEventItemId,
        reason: Option<String>,
    ) -> Result<()> {
        let timeline = self.timeline(room_id).await?;
        timeline.redact(item_id, reason.as_deref()).await?;
        Ok(())
    }

    pub async fn typing_notice(&self, room_id: &str, typing: bool) -> Result<()> {
        let room_id = RoomId::parse(room_id)?;
        let client = self.client().await;
        let room = client.get_room(&room_id).context("Room not found")?;
        room.typing_notice(typing).await?;
        Ok(())
    }

    pub async fn toggle_reaction(
        &self,
        room_id: &str,
        item_id: &TimelineEventItemId,
        reaction_key: &str,
    ) -> Result<()> {
        let timeline = self.timeline(room_id).await?;
        timeline.toggle_reaction(item_id, reaction_key).await?;
        Ok(())
    }

    pub async fn fetch_media(&self, source: MediaSource) -> Result<Vec<u8>> {
        let client = self.client().await;
        let request = matrix_sdk::media::MediaRequestParameters {
            source,
            format: MediaFormat::File,
        };
        let content = client.media().get_media_content(&request, true).await?;
        Ok(content)
    }

    pub async fn ignored_users(&self) -> Result<Vec<matrix_sdk::ruma::OwnedUserId>> {
        let client = self.client().await;
        let ignored = client
            .account()
            .account_data::<IgnoredUserListEventContent>()
            .await?;
        let users = if let Some(content) = ignored {
            let content = content.deserialize()?;
            content.ignored_users.into_keys().collect()
        } else {
            Vec::new()
        };
        Ok(users)
    }

    pub async fn ignore_user(&self, user_id: &UserId) -> Result<()> {
        let client = self.client().await;
        client.account().ignore_user(user_id).await?;
        Ok(())
    }

    pub async fn unignore_user(&self, user_id: &UserId) -> Result<()> {
        let client = self.client().await;
        client.account().unignore_user(user_id).await?;
        Ok(())
    }

    pub async fn is_user_ignored(&self, user_id: &UserId) -> Result<bool> {
        let client = self.client().await;
        let ignored = client
            .account()
            .account_data::<IgnoredUserListEventContent>()
            .await?;
        if let Some(content) = ignored {
            let content = content.deserialize()?;
            Ok(content.ignored_users.contains_key(user_id))
        } else {
            Ok(false)
        }
    }

    /// Fetches a rich URL preview generated by the Matrix homeserver via
    /// `/_matrix/client/v1/media/preview_url` or `/_matrix/media/v3/preview_url`.
    pub async fn get_media_preview(
        &self,
        url: &str,
    ) -> Result<Option<crate::utils::og::OgPreview>> {
        let client = self.client().await;
        let Some(data) = client.media().get_media_preview(url, None).await? else {
            return Ok(None);
        };

        let Some(meta) = crate::utils::og::parse_homeserver_preview_meta(url, data.get()) else {
            return Ok(None);
        };

        let image = if let Some(img) = &meta.image_url {
            if img.starts_with("mxc://") {
                let mxc_uri = matrix_sdk::ruma::OwnedMxcUri::from(img.as_str());
                let source = matrix_sdk::ruma::events::room::MediaSource::Plain(mxc_uri);
                match self.fetch_media(source).await {
                    Ok(bytes) => Some(cosmic::iced::widget::image::Handle::from_bytes(bytes)),
                    Err(err) => {
                        tracing::debug!("Failed to fetch MXC preview thumbnail {img}: {err}");
                        None
                    }
                }
            } else if img.starts_with("http://") || img.starts_with("https://") {
                crate::utils::og::fetch_image_handle_url(img).await
            } else if let Ok(parsed_url) = Url::parse(url)
                && let Ok(joined) = parsed_url.join(img)
            {
                crate::utils::og::fetch_image_handle_url(joined.as_str()).await
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(crate::utils::og::OgPreview {
            url: meta.url,
            title: meta.title,
            description: meta.description,
            site_name: meta.site_name,
            domain: meta.domain,
            image_url: meta.image_url,
            image,
        }))
    }
}
