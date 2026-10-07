//! Minimal i18n: embedded JSON tables + system-locale detection.
//!
//! 9 languages; English is the source and the fallback. The active language is
//! kept in a global (like the theme mode) so `Display` impls can localize too.
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Lang {
    En,
    Fr,
    De,
    Es,
    Ru,
    Ja,
    Ko,
    ZhCn,
}

impl Lang {
    pub const ALL: [Lang; 8] = [
        Lang::En,
        Lang::Fr,
        Lang::De,
        Lang::Es,
        Lang::Ru,
        Lang::Ja,
        Lang::Ko,
        Lang::ZhCn,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::Es => "es",
            Lang::Ru => "ru",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
            Lang::ZhCn => "zh-CN",
        }
    }

    /// Native name, shown in the language picker.
    pub fn endonym(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Fr => "Français",
            Lang::De => "Deutsch",
            Lang::Es => "Español",
            Lang::Ru => "Русский",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
            Lang::ZhCn => "简体中文",
        }
    }

    /// Best-effort match of the OS locale; falls back to English.
    pub fn detect() -> Lang {
        sys_locale::get_locale()
            .as_deref()
            .map(Lang::from_locale)
            .unwrap_or(Lang::En)
    }

    pub fn from_locale(locale: &str) -> Lang {
        let lower = locale.to_ascii_lowercase();
        let primary = lower.split(['-', '_']).next().unwrap_or("");
        match primary {
            "fr" => Lang::Fr,
            "de" => Lang::De,
            "es" => Lang::Es,
            "ru" => Lang::Ru,
            "ja" => Lang::Ja,
            "ko" => Lang::Ko,
            "zh" => Lang::ZhCn,
            _ => Lang::En,
        }
    }

    fn index(self) -> u8 {
        match self {
            Lang::En => 0,
            Lang::Fr => 1,
            Lang::De => 2,
            Lang::Es => 3,
            Lang::Ru => 4,
            Lang::Ja => 5,
            Lang::Ko => 6,
            Lang::ZhCn => 7,
        }
    }

    fn from_index(i: u8) -> Lang {
        match i {
            1 => Lang::Fr,
            2 => Lang::De,
            3 => Lang::Es,
            4 => Lang::Ru,
            5 => Lang::Ja,
            6 => Lang::Ko,
            7 => Lang::ZhCn,
            _ => Lang::En,
        }
    }
}

/// User preference: `None` = follow the system locale.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct LangPref(pub Option<Lang>);

impl LangPref {
    pub fn resolve(self) -> Lang {
        self.0.unwrap_or_else(Lang::detect)
    }

    pub fn choices() -> Vec<LangPref> {
        let mut v = vec![LangPref(None)];
        v.extend(Lang::ALL.iter().map(|l| LangPref(Some(*l))));
        v
    }
}

impl std::fmt::Display for LangPref {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str(t("lang_auto")),
            Some(l) => f.write_str(l.endonym()),
        }
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(lang: Lang) {
    LANG.store(lang.index(), Ordering::Relaxed);
}

pub fn lang() -> Lang {
    Lang::from_index(LANG.load(Ordering::Relaxed))
}

fn tables() -> &'static HashMap<Lang, HashMap<&'static str, &'static str>> {
    static TABLES: OnceLock<HashMap<Lang, HashMap<&'static str, &'static str>>> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(Lang::En, parse(include_str!("i18n/en.json")));
        m.insert(Lang::Fr, parse(include_str!("i18n/fr.json")));
        m.insert(Lang::De, parse(include_str!("i18n/de.json")));
        m.insert(Lang::Es, parse(include_str!("i18n/es.json")));
        m.insert(Lang::Ru, parse(include_str!("i18n/ru.json")));
        m.insert(Lang::Ja, parse(include_str!("i18n/ja.json")));
        m.insert(Lang::Ko, parse(include_str!("i18n/ko.json")));
        m.insert(Lang::ZhCn, parse(include_str!("i18n/zh-CN.json")));
        m
    })
}

fn parse(src: &str) -> HashMap<&'static str, &'static str> {
    let owned: HashMap<String, String> =
        serde_json::from_str(src).expect("invalid embedded language table");
    // Leak once so lookups return `&'static str` with zero allocation per call.
    owned
        .into_iter()
        .map(|(k, v)| {
            (
                &*Box::leak(k.into_boxed_str()),
                &*Box::leak(v.into_boxed_str()),
            )
        })
        .collect()
}

/// Translate `key` into the active language (no allocation).
pub fn t(key: &'static str) -> &'static str {
    t_lang(lang(), key)
}

/// Translate `key` into a specific language (falls back to English, then the key).
pub fn t_lang(lang: Lang, key: &'static str) -> &'static str {
    let tables = tables();
    tables
        .get(&lang)
        .and_then(|m| m.get(key))
        .copied()
        .or_else(|| tables.get(&Lang::En).and_then(|m| m.get(key)).copied())
        .unwrap_or(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_languages_have_the_same_keys() {
        let tables = tables();
        let en = tables.get(&Lang::En).unwrap();
        for lang in Lang::ALL {
            let table = tables.get(&lang).unwrap();
            for key in en.keys() {
                assert!(table.contains_key(key), "{} is missing key `{key}`", lang.code());
            }
            for key in table.keys() {
                assert!(en.contains_key(key), "{} has extra key `{key}`", lang.code());
            }
        }
    }

    #[test]
    fn locale_detection() {
        assert_eq!(Lang::from_locale("zh-CN"), Lang::ZhCn);
        assert_eq!(Lang::from_locale("zh-Hans"), Lang::ZhCn);
        assert_eq!(Lang::from_locale("en_US"), Lang::En);
        assert_eq!(Lang::from_locale("fr-FR"), Lang::Fr);
        assert_eq!(Lang::from_locale("de"), Lang::De);
        assert_eq!(Lang::from_locale("es-ES"), Lang::Es);
        assert_eq!(Lang::from_locale("ru-RU"), Lang::Ru);
        assert_eq!(Lang::from_locale("ja-JP"), Lang::Ja);
        assert_eq!(Lang::from_locale("ko-KR"), Lang::Ko);
        assert_eq!(Lang::from_locale("xx-YY"), Lang::En); // unknown -> English
    }

    #[test]
    fn missing_key_falls_back_to_key() {
        assert_eq!(t_lang(Lang::Fr, "does_not_exist"), "does_not_exist");
    }
}
