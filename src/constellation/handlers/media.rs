use crate::{Constellation, MediaSource, Message};
use cosmic::{Action, Application, Task};

impl Constellation {
    pub fn handle_fetch_media(
        &mut self,
        source: MediaSource,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            let mxc_url = match &source {
                MediaSource::Plain(uri) => uri.to_string(),
                MediaSource::Encrypted(file) => file.url.to_string(),
            };
            Task::perform(
                async move { matrix.fetch_media(source).await.map_err(|e| e.to_string()) },
                move |res| Action::from(Message::MediaFetched(mxc_url, res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_media_fetched(
        &mut self,
        mxc_url: String,
        res: Result<Vec<u8>, String>,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        match res {
            Ok(data) => {
                self.media_cache.insert(
                    mxc_url,
                    cosmic::iced::widget::image::Handle::from_bytes(data),
                );
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-fetch-media", error = e.to_string()).to_string(),
                );
            }
        }
        Task::none()
    }

    pub fn handle_media_fetched_batch(
        &mut self,
        batch: Vec<(String, Result<Vec<u8>, String>)>,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        for (mxc_url, res) in batch {
            match res {
                Ok(data) => {
                    self.media_cache.insert(
                        mxc_url,
                        cosmic::iced::widget::image::Handle::from_bytes(data),
                    );
                }
                Err(e) => {
                    self.set_error(
                        crate::fl!("error-failed-fetch-media", error = e.to_string()).to_string(),
                    );
                }
            }
        }
        Task::none()
    }

    /// Save a received media attachment (file/video/audio) to a user-chosen path.
    /// Runs the file chooser and the fetch+write off the main update path so the
    /// UI stays responsive; reports the outcome via [`Message::MediaSaved`].
    pub fn handle_save_media(
        &mut self,
        source: MediaSource,
        filename: String,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        let Some(matrix) = self.matrix.clone() else {
            return Task::none();
        };
        Task::perform(
            async move {
                let dialog = rfd::AsyncFileDialog::new()
                    .set_file_name(&filename)
                    .save_file()
                    .await;
                let Some(handle) = dialog else {
                    // User cancelled the chooser — not an error.
                    return Ok(None);
                };
                let path = handle.path().to_path_buf();
                let data = matrix
                    .fetch_media(source)
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::fs::write(&path, &data)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(Some(path))
            },
            |res| Action::from(Message::MediaSaved(res)),
        )
    }

    pub fn handle_media_saved(
        &mut self,
        res: Result<Option<std::path::PathBuf>, String>,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        match res {
            Ok(Some(path)) => {
                let body = crate::fl!("media-saved", path = path.display().to_string());
                let _ = notify_rust::Notification::new()
                    .appname("Constellation")
                    .summary("Constellation")
                    .body(&body)
                    .icon("document-save")
                    .show();
            }
            Ok(None) => {}
            Err(e) => {
                self.set_error(crate::fl!("error-failed-save-media", error = e).to_string());
            }
        }
        Task::none()
    }

    pub(super) fn handle_dnd_data_received(
        &mut self,
        mime: String,
        data: Vec<u8>,
    ) -> Task<Action<Message>> {
        if mime == "text/uri-list"
            && let Ok(text) = String::from_utf8(data)
        {
            let mut paths = Vec::new();
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(url) = url::Url::parse(line) {
                    if let Ok(path) = url.to_file_path() {
                        paths.push(path);
                    }
                } else {
                    let path = std::path::PathBuf::from(line);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
            if !paths.is_empty() {
                return self.handle_update(Message::AttachmentsSelected(paths));
            }
        }
        Task::none()
    }
}
