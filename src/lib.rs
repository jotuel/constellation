#![recursion_limit = "256"]

pub mod constellation;
pub mod matrix;
pub mod settings;
pub mod utils;
pub mod view;

#[cfg(feature = "video-player")]
pub use constellation::CachedVideo;
pub use constellation::{AuthFlow, Constellation, MenuAct, Message, QrLoginStep, SettingsPanel};
pub use cosmic::Core;
pub use matrix_sdk::ruma::OwnedRoomId;
pub use matrix_sdk::ruma::events::room::MediaSource;
pub use url::Url;
pub use utils::item::ConstellationItem;
pub use utils::preview::{PreviewEvent, parse_markdown, parse_plain_text};
pub use utils::{
    ApplyVectorDiffExt, contains_ignore_ascii_case, fuzzy_match_ignore_case, redact_url,
};

pub use utils::i18n;
pub use utils::ipc;
pub use utils::item;
pub use utils::preview;
pub use utils::rich_text;
pub use utils::unified_push;

use std::sync::LazyLock;

pub const CONSTELLATION_ICON: &[u8] = include_bytes!("../res/const.svg");

pub static TIMELINE_ID: LazyLock<cosmic::iced::widget::Id> =
    LazyLock::new(cosmic::iced::widget::Id::unique);
pub static THREADED_TIMELINE_ID: LazyLock<cosmic::iced::widget::Id> =
    LazyLock::new(cosmic::iced::widget::Id::unique);
pub static SEARCH_INPUT_ID: LazyLock<cosmic::iced::widget::Id> =
    LazyLock::new(cosmic::iced::widget::Id::unique);
pub static SEARCH_RESULTS_ID: LazyLock<cosmic::iced::widget::Id> =
    LazyLock::new(cosmic::iced::widget::Id::unique);

/// Determines whether a string is an accepted launch URI.
///
/// Accepts internal OIDC callbacks (`fi.joonastuomi.constellation:/`),
/// internal permalink wrappers (`fi.joonastuomi.constellation://`),
/// and raw Matrix permalinks (`matrix.to` / `matrix:`).
pub fn is_launch_uri(u: &str) -> bool {
    u.starts_with("fi.joonastuomi.constellation:/")
        || u.starts_with("fi.joonastuomi.constellation://")
        || utils::permalink::parse(u).is_ok()
}

/// Extracts a recognized launch URI from CLI arguments if present at `argv[1]`.
pub fn parse_launch_uri(args: &[String]) -> Option<String> {
    args.get(1).filter(|u| is_launch_uri(u)).cloned()
}
