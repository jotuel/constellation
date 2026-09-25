use crate::utils::widget::tooltip_button;
use cosmic::Element;
use cosmic::widget::{button, settings};

/// Renders a settings section error banner with a dismiss button if an error message is present.
pub fn view_error<'a, M: Clone + 'static>(
    error: Option<&'a str>,
    on_dismiss: M,
) -> Option<Element<'a, M>> {
    error.map(|err| {
        settings::section()
            .add(settings::item(
                err,
                button::text(crate::fl!("dismiss")).on_press(on_dismiss),
            ))
            .into()
    })
}

/// Standardized save button that reflects in-progress saving state, triggers save actions
/// when modifications exist, and displays a guidance tooltip when inactive.
pub fn save_button<'a, M: Clone + 'static>(
    is_saving: bool,
    has_changes: bool,
    on_save: M,
) -> Element<'a, M> {
    let mut save_btn = button::text(if is_saving {
        crate::fl!("saving")
    } else {
        crate::fl!("save-changes")
    });

    if has_changes && !is_saving {
        save_btn = save_btn.on_press(on_save);
    }

    if !is_saving && !has_changes {
        tooltip_button(save_btn, crate::fl!("make-changes-to-save"))
    } else {
        save_btn.into()
    }
}
