use i18n_embed::{
    DefaultLocalizer, LanguageRequester, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "res/i18n"]
struct Localizations;

pub static LOAD_LOCALIZATION: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader = fluent_language_loader!();
    let localizer = DefaultLocalizer::new(&loader, &Localizations);

    let requested_languages = i18n_embed::DesktopLanguageRequester::new().requested_languages();

    if let Err(e) = localizer.select(&requested_languages) {
        tracing::error!("Error while loading localizations: {}", e);
    }

    loader
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LOAD_LOCALIZATION, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LOAD_LOCALIZATION, $message_id, $($args),*)
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_about_i18n_strings() {
        assert_eq!(crate::fl!("about"), "About");
        assert_eq!(crate::fl!("app-about"), "About");
        let summary = crate::fl!("app-about-summary", version = "0.2.0");
        assert!(summary.contains("Version"));
        assert!(summary.contains("0.2.0"));
        assert_eq!(
            crate::fl!("app-about-description"),
            "Application information, version, and external links"
        );
    }
}
