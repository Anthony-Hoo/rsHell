use gtk::pango;

pub(crate) fn for_text(base: &pango::FontDescription, text: &str) -> pango::FontDescription {
    let mut font = base.clone();
    if let Some(family) = text_symbol_family(text) {
        font.set_family(family);
    } else if text.chars().any(is_emoji) {
        font.set_family(emoji_family());
    } else if text.chars().any(is_cjk) {
        font.set_family(cjk_family());
    }
    font
}

#[cfg(target_os = "windows")]
fn text_symbol_family(text: &str) -> Option<&'static str> {
    text.chars()
        .any(|value| value == '\u{279c}')
        .then_some("Segoe UI Symbol")
}

#[cfg(not(target_os = "windows"))]
fn text_symbol_family(_: &str) -> Option<&'static str> {
    None
}

fn is_cjk(value: char) -> bool {
    matches!(
        value,
        '\u{2e80}'..='\u{2fff}'
            | '\u{3000}'..='\u{303f}'
            | '\u{31c0}'..='\u{31ef}'
            | '\u{3400}'..='\u{4dbf}'
            | '\u{4e00}'..='\u{9fff}'
            | '\u{f900}'..='\u{faff}'
    )
}

fn is_emoji(value: char) -> bool {
    // The prompt chevron is a text dingbat covered by the terminal font.
    value != '\u{276f}' && matches!(value, '\u{2600}'..='\u{27bf}' | '\u{1f000}'..='\u{1faff}')
}

#[cfg(target_os = "windows")]
const fn cjk_family() -> &'static str {
    "Microsoft YaHei UI"
}

#[cfg(target_os = "macos")]
const fn cjk_family() -> &'static str {
    "PingFang SC"
}

#[cfg(all(unix, not(target_os = "macos")))]
const fn cjk_family() -> &'static str {
    "Monospace"
}

#[cfg(target_os = "windows")]
const fn emoji_family() -> &'static str {
    "Segoe UI Emoji"
}

#[cfg(target_os = "macos")]
const fn emoji_family() -> &'static str {
    "Apple Color Emoji"
}

#[cfg(all(unix, not(target_os = "macos")))]
const fn emoji_family() -> &'static str {
    "Monospace"
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use gtk::pango;

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_prompt_chevron_preserves_the_base_font_glyph() {
        let map = pangocairo::FontMap::new();
        let context = pango::Context::new();
        context.set_font_map(Some(&map));
        let settings = rshell_core::TerminalSettingsV1::default();
        let base = crate::terminal_metrics::logical_font_description(
            &settings.font_family,
            settings.font_size,
        );
        let text = "\u{276f}";
        let routed = super::for_text(&base, text);
        let counts = [&base, &routed].map(|description| {
            let layout = pango::Layout::new(&context);
            layout.set_font_description(Some(description));
            layout.set_text(text);
            layout.unknown_glyphs_count()
        });
        assert_eq!(
            counts[1], counts[0],
            "routing U+276F must not replace a covered prompt chevron with tofu"
        );
        assert_eq!(routed, base, "U+276F is a text dingbat, not an emoji");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_prompt_arrow_uses_a_covered_text_symbol_font() {
        let map = pangocairo::FontMap::new();
        let context = pango::Context::new();
        context.set_font_map(Some(&map));
        let settings = rshell_core::TerminalSettingsV1::default();
        let base = crate::terminal_metrics::logical_font_description(
            &settings.font_family,
            settings.font_size,
        );
        let text = "\u{279c}";
        let routed = super::for_text(&base, text);
        assert_eq!(routed.family().as_deref(), Some("Segoe UI Symbol"));
        assert_eq!(routed.size(), base.size());
        assert!(routed.is_size_absolute());
        let layout = pango::Layout::new(&context);
        layout.set_font_description(Some(&routed));
        layout.set_text(text);
        assert_eq!(
            layout.unknown_glyphs_count(),
            0,
            "the real U+279C prompt arrow must use a covered Windows text-symbol font"
        );
    }

    #[test]
    fn windows_terminal_fallbacks_cover_cjk_and_emoji_without_changing_ascii() {
        let settings = rshell_core::TerminalSettingsV1::default();
        let base = crate::terminal_metrics::logical_font_description(
            &settings.font_family,
            settings.font_size,
        );

        assert_eq!(super::for_text(&base, "ASCII"), base);
        assert_eq!(super::for_text(&base, "\u{276f}"), base);
        #[cfg(target_os = "windows")]
        {
            let map = pangocairo::FontMap::new();
            let context = pango::Context::new();
            context.set_font_map(Some(&map));
            for (text, family) in [
                ("界", "Microsoft YaHei UI"),
                ("🙂", "Segoe UI Emoji"),
                ("❤", "Segoe UI Emoji"),
            ] {
                let font = super::for_text(&base, text);
                assert_eq!(font.family().as_deref(), Some(family));
                assert_eq!(font.size(), base.size());
                assert!(font.is_size_absolute());
                let layout = pango::Layout::new(&context);
                layout.set_font_description(Some(&font));
                layout.set_text(text);
                assert_eq!(layout.unknown_glyphs_count(), 0, "missing glyph for {text}");
            }
        }
    }
}
