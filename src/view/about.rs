use cosmic::widget::about::About;
use cosmic::widget::icon;
use std::sync::LazyLock;

/// Constructs the `About` metadata model for Constellation.
pub fn about_info() -> About {
    About::default()
        .name("Constellation")
        .icon(icon::from_svg_bytes(crate::CONSTELLATION_ICON))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Joonas Tuomi")
        .license("Apache-2.0")
        .license_url("https://www.apache.org/licenses/LICENSE-2.0")
        .developers([("Joonas Tuomi", "me@joonastuomi.fi")])
        .links([
            ("Repository", "https://github.com/jotuel/constellation"),
            (
                "Support / Issues",
                "https://github.com/jotuel/constellation/issues",
            ),
            ("Donate", "https://github.com/sponsors/jotuel"),
        ])
}

/// Static cache of the About metadata model for context drawer rendering.
pub static ABOUT: LazyLock<About> = LazyLock::new(about_info);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_info_metadata() {
        let about = about_info();
        let debug_str = format!("{about:?}");

        assert!(debug_str.contains("name: Some(\"Constellation\")"));
        assert!(debug_str.contains("icon: Some("));
        assert!(debug_str.contains("data: Svg("));
        assert!(debug_str.contains(&format!("version: Some(\"{}\")", env!("CARGO_PKG_VERSION"))));
        assert!(debug_str.contains("author: Some(\"Joonas Tuomi\")"));
        assert!(debug_str.contains("license: Some(\"Apache-2.0\")"));
        assert!(
            debug_str
                .contains("license_url: Some(\"https://www.apache.org/licenses/LICENSE-2.0\")")
        );
        assert!(debug_str.contains("(\"Joonas Tuomi\", \"mailto:me@joonastuomi.fi\")"));
        assert!(
            debug_str.contains("(\"Repository\", \"https://github.com/jotuel/constellation\")")
        );
        assert!(debug_str.contains(
            "(\"Support / Issues\", \"https://github.com/jotuel/constellation/issues\")"
        ));
        assert!(debug_str.contains("(\"Donate\", \"https://github.com/sponsors/jotuel\")"));
    }

    #[test]
    fn test_about_widget_renders() {
        let about = about_info();
        let _element = cosmic::widget::about::about(&about, |_url| ());
    }
}
