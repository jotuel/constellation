#[cfg(feature = "webview-preview")]
use crate::constellation::{Constellation, WebviewPreview};
#[cfg(feature = "webview-preview")]
use cosmic::{Action, Application, Task};

#[cfg(feature = "webview-preview")]
impl Constellation {
    pub(super) fn handle_spin_webview(
        &mut self,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if let Some(preview) = self.webview_cache.values().next() {
            preview.state.spin();
        }
        Task::none()
    }

    pub(super) fn handle_webview_input(
        &mut self,
        url: String,
        input: cosmic_webview::InputEvent,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if let Some(preview) = self.webview_cache.get(&url) {
            preview.state.send_input(input);
        }
        Task::none()
    }

    pub(super) fn handle_webview_resize(
        &mut self,
        url: String,
        width: u32,
        height: u32,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if let Some(preview) = self.webview_cache.get_mut(&url) {
            preview.width = width;
            preview.height = height;
            preview.state.resize(dpi::PhysicalSize::new(width, height));
        }
        Task::none()
    }

    pub(super) fn handle_toggle_webview_preview(
        &mut self,
        url_str: String,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if self.expanded_webview_previews.contains(&url_str) {
            self.expanded_webview_previews.remove(&url_str);
        } else {
            self.expanded_webview_previews.insert(url_str.clone());
            if !self.webview_cache.contains_key(&url_str)
                && let Ok(parsed_url) = url::Url::parse(&url_str)
            {
                let (state, rx) = cosmic_webview::ServoState::new(
                    parsed_url,
                    dpi::PhysicalSize::new(400, 250),
                    1.0,
                );
                if self.webview_rx.is_none() {
                    self.webview_rx = Some(rx);
                }
                self.webview_cache.insert(
                    url_str.clone(),
                    WebviewPreview {
                        state,
                        width: 400,
                        height: 250,
                    },
                );
            }
        }
        Task::none()
    }
}
