use crate::core;
use debug_util::debug_print;

pub struct I18n {
    lang: String,
}

impl I18n {
    pub fn new(lang: &str) -> Self {
        debug_print(&format!(
            "[DEBUG] I18n initialized with language: '{}'",
            lang
        ));
        Self {
            lang: lang.to_string(),
        }
    }

    // 将来使う予定のメソッドには警告抑制を付与
    #[allow(dead_code)]
    pub fn set_lang(&mut self, lang: &str) {
        debug_print(&format!(
            "[DEBUG] I18n language changed from '{}' to '{}'",
            self.lang, lang
        ));
        self.lang = lang.to_string();
    }

    pub fn t(&self, key: &str) -> String {
        debug_print(&format!(
            "[DEBUG] I18n translating key '{}' for lang '{}'",
            key, self.lang
        ));
        core::get_text(&self.lang, key)
    }
}
