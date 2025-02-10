// src/i18n/mod.rs
use fluent::{FluentBundle, FluentResource};
use std::collections::HashMap;
use std::sync::Arc;
use unic_langid::LanguageIdentifier;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::error::Error;
use yew::use_state;
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct I18n {
    bundles: Arc<RwLock<HashMap<String, FluentBundle<FluentResource>>>>,
    current_language: Arc<RwLock<String>>,
}

// Define available languages
pub static AVAILABLE_LANGUAGES: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    vec![
        ("en", "English"),
        ("es", "Español"),
        ("fr", "Français"),
        ("de", "Deutsch"),
    ]
});

impl I18n {
    pub fn new() -> Self {
        let mut bundles = HashMap::new();
        
        // Initialize with English as default
        let en_content = include_str!("../locales/en.ftl");
        let en_resource = match FluentResource::try_new(en_content.to_string()) {
            Ok(res) => res,
            Err((res, _)) => res, // Ignore parsing errors and use what we could parse
        };
        
        let mut bundle = FluentBundle::new(vec![
            "en".parse().expect("Failed to parse language identifier")
        ]);
        bundle.add_resource(en_resource).expect("Failed to add English resource");
        bundles.insert("en".to_string(), bundle);

        I18n {
            bundles: Arc::new(RwLock::new(bundles)),
            current_language: Arc::new(RwLock::new("en".to_string())),
        }
    }

    pub fn load_language(&self, lang_code: &str) -> Result<(), Box<dyn Error>> {
        // Load the FTL file content
        let content = if cfg!(debug_assertions) {
            // In debug mode, read from file system
            let path = Path::new("locales").join(format!("{}.ftl", lang_code));
            fs::read_to_string(path)?
        } else {
            // In release mode, use compiled-in resources
            match lang_code {
                "en" => include_str!("../locales/en.ftl"),
                "es" => include_str!("../locales/es.ftl"),
                "fr" => include_str!("../locales/fr.ftl"),
                "de" => include_str!("../locales/de.ftl"),
                _ => return Err("Language not supported".into()),
            }
            .to_string()
        };

        let resource = match FluentResource::try_new(content) {
            Ok(res) => res,
            Err((res, _)) => res,
        };

        let lang_id: LanguageIdentifier = lang_code.parse()?;
        let mut bundle = FluentBundle::new(vec![lang_id]);
        if let Err(errors) = bundle.add_resource(resource) {
            log::warn!("Errors while adding resource: {:?}", errors);
        }

        self.bundles.write().insert(lang_code.to_string(), bundle);
        Ok(())
    }

    pub fn set_language(&self, lang_code: &str) -> Result<(), Box<dyn Error>> {
        if !self.bundles.read().contains_key(lang_code) {
            self.load_language(lang_code)?;
        }
        *self.current_language.write() = lang_code.to_string();
        Ok(())
    }

    pub fn get_message(&self, key: &str) -> String {
        let bundles = self.bundles.read();
        let current_lang = self.current_language.read();
        
        if let Some(bundle) = bundles.get(&*current_lang) {
            if let Some(message) = bundle.get_message(key) {
                if let Some(pattern) = message.value() {
                    let mut errors = vec![];
                    return bundle
                        .format_pattern(pattern, None, &mut errors)
                        .into_owned();
                }
            }
        }
        
        // Fallback to key if translation not found
        key.to_string()
    }
}

thread_local! {
    pub static I18N: I18n = I18n::new();
}

use yew::hook;

#[hook]
pub fn use_translator() -> Box<dyn Fn(&str) -> String> {
    Box::new(move |key: &str| -> String {
        I18N.with(|i18n| i18n.get_message(key))
    })
}

#[hook]
pub fn use_language() -> (String, Box<dyn Fn(&str)>) {
    let language = use_state(|| I18N.with(|i18n| i18n.current_language.read().clone()));
    
    let set_language = {
        let language = language.clone();
        Box::new(move |lang: &str| {
            I18N.with(|i18n| {
                if let Ok(()) = i18n.set_language(lang) {
                    language.set(lang.to_string());
                }
            });
        }) as Box<dyn Fn(&str)>
    };
    
    ((*language).clone(), set_language)
}