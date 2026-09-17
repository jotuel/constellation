use super::*;
use matrix_sdk::ruma::events::StateEventType;
use matrix_sdk::ruma::events::image_pack::rooms::{ImagePackRoomsEventContent, RoomImagePackMeta};
use matrix_sdk::ruma::events::room::image_pack::{PackUsage, RoomImagePackEventContent};
use matrix_sdk::ruma::{OwnedRoomId, RoomId};
use matrix_sdk_base::deserialized_responses::SyncOrStrippedState;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImagePackItem {
    pub shortcode: String,
    pub body: String,
    pub url: String,
    pub is_emoji: bool,
    pub is_sticker: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImagePack {
    pub room_id: Option<OwnedRoomId>,
    pub state_key: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub images: Vec<ImagePackItem>,
    pub is_globally_enabled: bool,
}

pub type AccountImagePacksData = (
    Vec<ImagePack>,
    BTreeMap<OwnedRoomId, BTreeMap<String, RoomImagePackMeta>>,
);

impl MatrixEngine {
    /// Fetch all image packs published as room state events in `room_id`.
    /// Handles standard `m.room.image_pack` and falls back to `im.ponies.room_emotes`.
    pub async fn get_room_image_packs(&self, room_id: &RoomId) -> Result<Vec<ImagePack>> {
        let client = self.client().await;
        let room = client.get_room(room_id).context("Room not found")?;

        let mut packs = Vec::new();

        // 1. Query standard m.room.image_pack events
        if let Ok(state_events) = room
            .get_state_events_static::<RoomImagePackEventContent>()
            .await
        {
            for raw_ev in state_events {
                if let Ok(ev) = raw_ev.deserialize()
                    && let SyncOrStrippedState::Sync(
                        matrix_sdk::ruma::events::SyncStateEvent::Original(original),
                    ) = ev
                {
                    packs.push(Self::parse_room_image_pack_content(
                        Some(room_id.to_owned()),
                        original.state_key.as_str().to_string(),
                        &original.content,
                    ));
                }
            }
        }

        // 2. If no standard packs found, try legacy im.ponies.room_emotes
        if packs.is_empty()
            && let Ok(legacy_events) = room
                .get_state_events(StateEventType::from("im.ponies.room_emotes"))
                .await
        {
            for raw_ev in legacy_events {
                if let Ok(state) = raw_ev.cast::<RoomImagePackEventContent>().deserialize()
                    && let SyncOrStrippedState::Sync(
                        matrix_sdk::ruma::events::SyncStateEvent::Original(ev),
                    ) = state
                {
                    packs.push(Self::parse_room_image_pack_content(
                        Some(room_id.to_owned()),
                        ev.state_key.as_str().to_string(),
                        &ev.content,
                    ));
                }
            }
        }

        Ok(packs)
    }

    /// Parse a `RoomImagePackEventContent` into `ImagePack`.
    /// Respects MSC2545 usage inheritance:
    /// "If the image has no usage specified, it inherits the pack's usage.
    /// If neither specify a usage, the image is assumed to be an emoticon."
    pub fn parse_room_image_pack_content(
        room_id: Option<OwnedRoomId>,
        state_key: String,
        content: &RoomImagePackEventContent,
    ) -> ImagePack {
        let pack_meta = &content.pack;
        let pack_usage = &pack_meta.usage;

        let (is_emoji, is_sticker) = if !pack_usage.is_empty() {
            (
                pack_usage.contains(&PackUsage::Emoticon),
                pack_usage.contains(&PackUsage::Sticker),
            )
        } else {
            // Default: emoticon
            (true, false)
        };

        let mut images = Vec::new();
        for (shortcode, img) in &content.images {
            let (width, height) = if let Some(info) = &img.info {
                (
                    info.width.and_then(|w| u32::try_from(u64::from(w)).ok()),
                    info.height.and_then(|h| u32::try_from(u64::from(h)).ok()),
                )
            } else {
                (None, None)
            };
            let body = img.body.clone().unwrap_or_else(|| shortcode.clone());

            images.push(ImagePackItem {
                shortcode: shortcode.clone(),
                body,
                url: img.url.to_string(),
                is_emoji,
                is_sticker,
                width,
                height,
            });
        }

        // Sort images by shortcode for consistent presentation
        images.sort_by(|a, b| a.shortcode.cmp(&b.shortcode));

        ImagePack {
            room_id,
            state_key,
            display_name: pack_meta.display_name.clone(),
            avatar_url: pack_meta.avatar_url.as_ref().map(|u| u.to_string()),
            images,
            is_globally_enabled: false,
        }
    }

    /// Query global account data for subscribed room packs (`m.image_pack.rooms`)
    /// and personal packs (`m.image_pack`).
    pub async fn get_account_image_packs(
        &self,
    ) -> Result<(
        Vec<ImagePack>,
        BTreeMap<OwnedRoomId, BTreeMap<String, RoomImagePackMeta>>,
    )> {
        let client = self.client().await;

        // 1. Fetch room subscriptions from account data
        let mut rooms_map = BTreeMap::new();
        if let Ok(Some(raw)) = client
            .account()
            .account_data::<ImagePackRoomsEventContent>()
            .await
            && let Ok(content) = raw.deserialize()
        {
            rooms_map = content.rooms;
        } else if let Ok(Some(raw)) = client
            .account()
            .fetch_account_data(matrix_sdk::ruma::events::GlobalAccountDataEventType::from(
                "im.ponies.emote_rooms",
            ))
            .await
            && let Ok(Some(content)) = raw.get_field::<ImagePackRoomsEventContent>("content")
        {
            rooms_map = content.rooms;
        }

        // 2. Fetch personal pack from account data
        let mut personal_packs = Vec::new();
        if let Ok(Some(raw)) = client
            .account()
            .fetch_account_data(matrix_sdk::ruma::events::GlobalAccountDataEventType::from(
                "m.image_pack",
            ))
            .await
            && let Ok(Some(content)) = raw.get_field::<RoomImagePackEventContent>("content")
        {
            personal_packs.push(Self::parse_room_image_pack_content(
                None,
                "user".to_string(),
                &content,
            ));
        } else if let Ok(Some(raw)) = client
            .account()
            .fetch_account_data(matrix_sdk::ruma::events::GlobalAccountDataEventType::from(
                "im.ponies.user_emotes",
            ))
            .await
            && let Ok(Some(content)) = raw.get_field::<RoomImagePackEventContent>("content")
        {
            personal_packs.push(Self::parse_room_image_pack_content(
                None,
                "user".to_string(),
                &content,
            ));
        }

        Ok((personal_packs, rooms_map))
    }

    /// Save or update a room image pack state event.
    pub async fn set_room_image_pack(
        &self,
        room_id: &RoomId,
        state_key: &str,
        content: RoomImagePackEventContent,
    ) -> Result<()> {
        let client = self.client().await;
        let room = client.get_room(room_id).context("Room not found")?;

        room.send_state_event_for_key(state_key, content).await?;
        Ok(())
    }

    /// Delete a room image pack state event by setting empty images.
    pub async fn delete_room_image_pack(&self, room_id: &RoomId, state_key: &str) -> Result<()> {
        let client = self.client().await;
        let room = client.get_room(room_id).context("Room not found")?;

        let empty = RoomImagePackEventContent::new(BTreeMap::new());
        room.send_state_event_for_key(state_key, empty).await?;
        Ok(())
    }

    /// Toggle a room pack's subscription in global account data (`m.image_pack.rooms`).
    pub async fn set_account_pack_subscription(
        &self,
        room_id: &RoomId,
        state_key: &str,
        enabled: bool,
    ) -> Result<()> {
        let client = self.client().await;

        let mut rooms_map = BTreeMap::new();
        if let Ok(Some(raw)) = client
            .account()
            .account_data::<ImagePackRoomsEventContent>()
            .await
            && let Ok(content) = raw.deserialize()
        {
            rooms_map = content.rooms;
        }

        let entry = rooms_map.entry(room_id.to_owned()).or_default();
        if enabled {
            entry.insert(state_key.to_string(), RoomImagePackMeta::new());
        } else {
            entry.remove(state_key);
        }

        if entry.is_empty() {
            rooms_map.remove(room_id);
        }

        let new_content = ImagePackRoomsEventContent::new(rooms_map);
        client.account().set_account_data(new_content).await?;
        Ok(())
    }

    /// Upload media bytes to the homeserver and return the mxc URI.
    pub async fn upload_media(
        &self,
        data: Vec<u8>,
        mime: &str,
    ) -> Result<matrix_sdk::ruma::OwnedMxcUri> {
        let client = self.client().await;
        let content_type = mime.parse::<mime::Mime>()?;
        let res = client.media().upload(&content_type, data, None).await?;
        Ok(res.content_uri)
    }

    /// Create a new room image pack with given display name.
    pub async fn create_room_image_pack(
        &self,
        room_id: &RoomId,
        state_key: &str,
        display_name: String,
    ) -> Result<()> {
        let mut content = RoomImagePackEventContent::new(BTreeMap::new());
        content.pack.display_name = Some(display_name);
        self.set_room_image_pack(room_id, state_key, content).await
    }

    /// Add an image to an existing room pack.
    pub async fn add_image_to_room_pack(
        &self,
        room_id: &RoomId,
        state_key: &str,
        shortcode: String,
        mxc_url: matrix_sdk::ruma::OwnedMxcUri,
    ) -> Result<()> {
        let client = self.client().await;
        let room = client.get_room(room_id).context("Room not found")?;

        let mut content = if let Ok(state_events) = room
            .get_state_events_static::<RoomImagePackEventContent>()
            .await
        {
            let mut found = None;
            for raw_ev in state_events {
                if let Ok(ev) = raw_ev.deserialize()
                    && let SyncOrStrippedState::Sync(
                        matrix_sdk::ruma::events::SyncStateEvent::Original(original),
                    ) = ev
                    && original.state_key.as_str() == state_key
                {
                    found = Some(original.content);
                    break;
                }
            }
            found.unwrap_or_else(|| RoomImagePackEventContent::new(BTreeMap::new()))
        } else {
            RoomImagePackEventContent::new(BTreeMap::new())
        };

        use ruma_events::room::image_pack::ImagePackImage;
        let img = ImagePackImage::new(mxc_url);
        content.images.insert(shortcode, img);

        self.set_room_image_pack(room_id, state_key, content).await
    }
}
