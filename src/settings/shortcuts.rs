//! Keyboard-shortcuts settings panel: view current bindings, rebind
//! changeable actions by recording a new combo, and Save to commit the draft
//! into config + the running app.

use crate::constellation::keybind::{
    Bindings, KeyBind, SerializedKeyBind, ShortcutAction, format_keybind,
};
use cosmic::iced::Alignment;
use cosmic::widget::{self, button, settings};
use cosmic::{Action, Element, Task};
use std::collections::HashMap;

/// A shortcut recording in progress on the shortcuts page.
#[derive(Debug, Clone)]
pub struct Recording {
    pub action: ShortcutAction,
    pub captured: Option<KeyBind>,
    /// Action currently holding the captured combo, if any.
    pub conflict_with: Option<ShortcutAction>,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    /// Draft of effective bindings being edited: every action → its binding
    /// (`None` = intentionally unbound). Seeded from the active bindings.
    pub draft: HashMap<ShortcutAction, Option<KeyBind>>,
    /// Active recording dialog state.
    pub recording: Option<Recording>,
    /// Persisted overrides (draft entries that differ from the default).
    /// Mirrored into `Config::key_bindings` on save; kept here so unrelated
    /// `build_config()` calls never wipe saved shortcuts.
    pub overrides: HashMap<ShortcutAction, SerializedKeyBind>,
    /// Whether the draft differs from the persisted overrides.
    pub dirty: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartRecording(ShortcutAction),
    RecordingCaptured(KeyBind),
    CancelRecording,
    ConfirmRecording,
    ResetToDefault(ShortcutAction),
    ResetAll,
    Save,
}

/// The actions grouped under "General" on the page.
const GENERAL: [ShortcutAction; 7] = [
    ShortcutAction::ToggleAppSettings,
    ShortcutAction::ToggleUserSettings,
    ShortcutAction::ToggleRoomSettings,
    ShortcutAction::ToggleSpaceSettings,
    ShortcutAction::Quit,
    ShortcutAction::Search,
    ShortcutAction::CopyText,
];

/// The actions grouped under "Navigation" on the page.
const NAVIGATION: [ShortcutAction; 6] = [
    ShortcutAction::ToggleSpaceSwitcher,
    ShortcutAction::CloseThread,
    ShortcutAction::SelectRoomList,
    ShortcutAction::SelectSpaceSwitcher,
    ShortcutAction::ScrollUp,
    ShortcutAction::ScrollDown,
];

impl State {
    /// Seed the draft from the currently active bindings.
    pub fn from_bindings(bindings: &Bindings) -> Self {
        let draft = ShortcutAction::ALL
            .iter()
            .map(|&action| (action, bindings.keybind_for(action).cloned()))
            .collect();
        Self {
            draft,
            recording: None,
            overrides: HashMap::new(),
            dirty: false,
        }
    }

    fn binding_of(&self, action: ShortcutAction) -> Option<&KeyBind> {
        self.draft.get(&action).and_then(|kb| kb.as_ref())
    }

    /// Overrides implied by the current draft: every entry whose binding
    /// differs from the shipped default. An unbound-by-default action that was
    /// bound, or an empty binding, is stored as an explicitly-empty value so
    /// it round-trips as "unbound" instead of falling back to the default.
    pub fn compute_overrides(&self) -> HashMap<ShortcutAction, SerializedKeyBind> {
        let mut overrides = HashMap::new();
        for (&action, draft_kb) in &self.draft {
            let ser = draft_kb.as_ref().map(SerializedKeyBind::from);
            let matches_default = match (&ser, action.default_keybind()) {
                (None, None) => true,
                (None, Some(_)) => false,
                (Some(ser), None) => ser.is_unbound(),
                (Some(ser), Some(def)) => *ser == SerializedKeyBind::from(&def),
            };
            if !matches_default {
                overrides.insert(action, ser.unwrap_or_default());
            }
        }
        overrides
    }

    pub fn update(&mut self, message: Message) -> Task<Action<crate::Message>> {
        match message {
            Message::StartRecording(action) => {
                if !action.rebindable() {
                    return Task::none();
                }
                self.recording = Some(Recording {
                    action,
                    captured: None,
                    conflict_with: None,
                });
                Task::none()
            }
            Message::RecordingCaptured(kb) => {
                if let Some(rec) = &mut self.recording {
                    // Conflict against the whole draft except the action
                    // being rebound.
                    rec.conflict_with = self.draft.iter().find_map(|(&a, existing)| {
                        (a != rec.action && existing.as_ref() == Some(&kb)).then_some(a)
                    });
                    rec.captured = Some(kb);
                }
                Task::none()
            }
            Message::CancelRecording => {
                self.recording = None;
                Task::none()
            }
            Message::ConfirmRecording => {
                if let Some(rec) = self.recording.take()
                    && let Some(combo) = rec.captured
                {
                    self.draft.insert(rec.action, Some(combo));
                    // The conflicting action loses its binding.
                    if let Some(other) = rec.conflict_with {
                        self.draft.insert(other, None);
                    }
                    self.dirty = true;
                }
                Task::none()
            }
            Message::ResetToDefault(action) => {
                self.draft.insert(action, action.default_keybind());
                self.dirty = true;
                Task::none()
            }
            Message::ResetAll => {
                for action in ShortcutAction::ALL {
                    self.draft.insert(action, action.default_keybind());
                }
                self.dirty = true;
                Task::none()
            }
            Message::Save => Task::done(Action::from(crate::Message::ShortcutsSaved)),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if let Some(rec) = &self.recording {
            return self.view_recording(rec);
        }
        self.view_bindings()
    }

    fn view_bindings(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::spacing();
        let mut column = widget::column::with_capacity(4).spacing(spacing.space_m);

        let sections: [(String, &[ShortcutAction]); 2] = [
            (
                crate::fl!("shortcuts-section-general").to_string(),
                &GENERAL,
            ),
            (
                crate::fl!("shortcuts-section-navigation").to_string(),
                &NAVIGATION,
            ),
        ];
        for (title, actions) in sections {
            let mut list = settings::section().title(title);
            for &action in actions {
                list = list.add(self.view_action_row(action));
            }
            column = column.push(list);
        }

        column = column.push(
            widget::row::with_capacity(2)
                .spacing(spacing.space_xs)
                .push(button::suggested(crate::fl!("shortcuts-save")).on_press(Message::Save))
                .push(
                    button::destructive(crate::fl!("shortcuts-reset-all"))
                        .on_press(Message::ResetAll),
                ),
        );

        widget::scrollable(column.width(cosmic::iced::Length::Fill))
            .width(cosmic::iced::Length::Fill)
            .into()
    }

    fn view_action_row(&self, action: ShortcutAction) -> cosmic::Element<'static, Message> {
        let spacing = cosmic::theme::spacing();
        let combo_text = self
            .binding_of(action)
            .map(format_keybind)
            .unwrap_or_else(|| crate::fl!("shortcuts-unbound").to_string());

        let mut control: cosmic::Element<'static, Message> = if action.rebindable() {
            button::text(combo_text)
                .on_press(Message::StartRecording(action))
                .into()
        } else {
            widget::text::body(combo_text).into()
        };

        let has_override = self
            .binding_of(action)
            .map(|kb| Some(kb) != action.default_keybind().as_ref())
            .unwrap_or(false);
        if action.rebindable() && has_override {
            control = widget::row::with_capacity(2)
                .spacing(spacing.space_xs)
                .align_y(Alignment::Center)
                .push(control)
                .push(
                    button::icon(widget::icon::from_name("edit-undo-symbolic").symbolic(true))
                        .extra_small()
                        .on_press(Message::ResetToDefault(action)),
                )
                .into();
        }

        settings::item(action_label(action), control).into()
    }

    fn view_recording<'a>(&'a self, rec: &'a Recording) -> Element<'a, Message> {
        let spacing = cosmic::theme::spacing();

        let mut col = widget::column::with_capacity(5)
            .spacing(spacing.space_m)
            .push(widget::text::title3(crate::fl!(
                "shortcuts-record-title",
                action = action_label(rec.action)
            )))
            .push(widget::text::body(crate::fl!("shortcuts-record-hint")));

        col = col.push(
            widget::container(widget::text::title2(match &rec.captured {
                Some(combo) => format_keybind(combo),
                None => String::from("\u{2026}"),
            }))
            .padding(spacing.space_l)
            .center_x(cosmic::iced::Length::Fill),
        );

        if let Some(conflict) = rec.conflict_with {
            col = col.push(widget::text::body(crate::fl!(
                "shortcuts-record-conflict",
                action = action_label(conflict)
            )));
        }

        let mut buttons = widget::row::with_capacity(2)
            .spacing(spacing.space_xs)
            .push(button::text(crate::fl!("cancel")).on_press(Message::CancelRecording));
        if rec.captured.is_some() {
            buttons = buttons.push(
                button::suggested(crate::fl!("shortcuts-record-confirm"))
                    .on_press(Message::ConfirmRecording),
            );
        }

        col.push(buttons)
            .padding(spacing.space_l)
            .width(cosmic::iced::Length::Fill)
            .into()
    }
}

fn action_label(action: ShortcutAction) -> String {
    match action {
        ShortcutAction::ToggleAppSettings => crate::fl!("shortcut-toggle-app-settings"),
        ShortcutAction::ToggleUserSettings => crate::fl!("shortcut-toggle-user-settings"),
        ShortcutAction::ToggleRoomSettings => crate::fl!("shortcut-toggle-room-settings"),
        ShortcutAction::ToggleSpaceSettings => crate::fl!("shortcut-toggle-space-settings"),
        ShortcutAction::ToggleSpaceSwitcher => crate::fl!("shortcut-toggle-space-switcher"),
        ShortcutAction::CloseThread => crate::fl!("shortcut-close-thread"),
        ShortcutAction::Quit => crate::fl!("shortcut-quit"),
        ShortcutAction::Search => crate::fl!("shortcut-search"),
        ShortcutAction::CopyText => crate::fl!("shortcut-copy-text"),
        ShortcutAction::SelectRoomList => crate::fl!("shortcut-select-room-list"),
        ShortcutAction::SelectSpaceSwitcher => crate::fl!("shortcut-select-space-switcher"),
        ShortcutAction::ScrollUp => crate::fl!("shortcut-scroll-up"),
        ShortcutAction::ScrollDown => crate::fl!("shortcut-scroll-down"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic::widget::menu::key_bind::Modifier;

    fn seeded() -> State {
        State::from_bindings(&Bindings::defaults())
    }

    #[test]
    fn record_confirm_rebinds_and_unbinds_conflict() {
        let mut s = seeded();

        // Rebind SelectSpaceSwitcher ("S") onto Ctrl+J; no conflicts yet.
        let kb = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: cosmic::iced::keyboard::Key::Character("j".into()),
        };
        let _ = s.update(Message::StartRecording(ShortcutAction::SelectSpaceSwitcher));
        assert!(s.recording.is_some());
        let _ = s.update(Message::RecordingCaptured(kb.clone()));
        assert_eq!(s.recording.as_ref().unwrap().conflict_with, None);
        let _ = s.update(Message::ConfirmRecording);
        assert_eq!(s.binding_of(ShortcutAction::SelectSpaceSwitcher), Some(&kb));

        // Now record ScrollUp onto the same combo; SelectSpaceSwitcher must
        // lose its binding.
        let _ = s.update(Message::StartRecording(ShortcutAction::ScrollUp));
        let _ = s.update(Message::RecordingCaptured(kb.clone()));
        assert_eq!(
            s.recording.as_ref().unwrap().conflict_with,
            Some(ShortcutAction::SelectSpaceSwitcher)
        );
        let _ = s.update(Message::ConfirmRecording);
        assert!(s.binding_of(ShortcutAction::SelectSpaceSwitcher).is_none());
        assert_eq!(s.binding_of(ShortcutAction::ScrollUp), Some(&kb));
        assert!(s.dirty);
    }

    #[test]
    fn fixed_actions_cannot_start_recording() {
        let mut s = seeded();
        let _ = s.update(Message::StartRecording(ShortcutAction::Quit));
        assert!(s.recording.is_none());
    }

    #[test]
    fn overrides_skip_defaults_and_keep_unbound() {
        let mut s = seeded();
        // Unbind CloseThread: an explicit empty override must be produced.
        s.draft.insert(ShortcutAction::CloseThread, None);
        let overrides = s.compute_overrides();
        assert_eq!(overrides.len(), 1);
        assert!(overrides[&ShortcutAction::CloseThread].is_unbound());

        // Rebinding away from default yields exactly that override.
        let kb = ShortcutAction::ScrollUp.default_keybind().unwrap();
        s.draft
            .insert(ShortcutAction::CloseThread, Some(kb.clone()));
        let overrides = s.compute_overrides();
        assert_eq!(overrides.len(), 1);
        assert_eq!(
            overrides.get(&ShortcutAction::CloseThread),
            Some(&SerializedKeyBind::from(&kb))
        );

        // Pure defaults produce no overrides at all.
        let clean = seeded();
        assert!(clean.compute_overrides().is_empty());
    }

    #[test]
    fn reset_all_restores_defaults() {
        let mut s = seeded();
        s.draft.insert(ShortcutAction::ScrollUp, None);
        let _ = s.update(Message::ResetAll);
        assert_eq!(
            s.binding_of(ShortcutAction::ScrollUp),
            ShortcutAction::ScrollUp.default_keybind().as_ref()
        );
        assert!(s.dirty);
    }
}
