use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::OnceLock;

// HashMap<PluginName, HashMap<LangCode, Content>>
static PLUGIN_LANG_CACHE: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

// 起動時の引数に --debug または -d があるか判定
fn is_debug_mode() -> bool {
    std::env::args().any(|arg| arg == "--debug" || arg == "-d")
}

// デバッグ用出力マクロ
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if is_debug_mode() {
            println!($($arg)*);
        }
    };
}

fn get_plugin_lang_cache() -> &'static HashMap<String, HashMap<String, String>> {
    PLUGIN_LANG_CACHE.get_or_init(|| {
        let mut root_map = HashMap::new();

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        // dist/libraries/ から 1つ上の階層 (dist/) にある PluginsLanguages を探す
        let parent_dir = exe_dir.parent().map(|p| p.join("PluginsLanguages"));
        let base_dir = if let Some(ref p) = parent_dir {
            if p.exists() {
                p.clone()
            } else {
                let local_dir = exe_dir.join("PluginsLanguages");
                if local_dir.exists() {
                    local_dir
                } else {
                    PathBuf::from("PluginsLanguages")
                }
            }
        } else {
            PathBuf::from("PluginsLanguages")
        };

        debug_print!("[DEBUG] Plugin lang base_dir resolved to: {:?}", base_dir);

        if let Ok(plugin_entries) = fs::read_dir(&base_dir) {
            for plugin_entry in plugin_entries.flatten() {
                let plugin_path = plugin_entry.path();
                if plugin_path.is_dir() {
                    if let Some(plugin_name) = plugin_path.file_name().and_then(|s| s.to_str()) {
                        let mut lang_map = HashMap::new();

                        if let Ok(lang_entries) = fs::read_dir(&plugin_path) {
                            for lang_entry in lang_entries.flatten() {
                                let lang_file_path = lang_entry.path();
                                if lang_file_path.is_file()
                                    && lang_file_path.extension().and_then(|s| s.to_str())
                                        == Some("lang")
                                {
                                    if let Some(file_stem) =
                                        lang_file_path.file_stem().and_then(|s| s.to_str())
                                    {
                                        if let Ok(content) = fs::read_to_string(&lang_file_path) {
                                            let key = file_stem.to_lowercase().replace('_', "-");
                                            debug_print!("[DEBUG] Loaded lang file: {:?} -> normalized key: '{}'", lang_file_path, key);
                                            lang_map.insert(key, content);
                                        }
                                    }
                                }
                            }
                        }
                        debug_print!("[DEBUG] Registered plugin in cache: '{}' with langs: {:?}", plugin_name.to_lowercase(), lang_map.keys().collect::<Vec<_>>());
                        root_map.insert(plugin_name.to_lowercase(), lang_map);
                    }
                }
            }
        } else {
            debug_print!("[DEBUG] WARNING: Failed to read plugin dir: {:?}", base_dir);
        }
        root_map
    })
}

fn parse_key_from_content(content: &str, key: &str) -> String {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let clean_k = k.trim().trim_start_matches('\u{feff}');
            if clean_k == key {
                return v.trim().to_string();
            }
        }
    }
    format!("[Key not found: {}]", key)
}

#[no_mangle]
pub extern "C" fn get_plugin_translation(
    plugin_name_ptr: *const c_char,
    lang_code_ptr: *const c_char,
    key_ptr: *const c_char,
) -> *mut c_char {
    if plugin_name_ptr.is_null() || lang_code_ptr.is_null() || key_ptr.is_null() {
        debug_print!("[DEBUG] get_plugin_translation received null pointer!");
        return std::ptr::null_mut();
    }

    let plugin_name = unsafe {
        match CStr::from_ptr(plugin_name_ptr).to_str() {
            Ok(s) => s.to_lowercase(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let lang_code = unsafe {
        match CStr::from_ptr(lang_code_ptr).to_str() {
            Ok(s) => s.to_lowercase().replace('_', "-"),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let key = unsafe {
        match CStr::from_ptr(key_ptr).to_str() {
            Ok(s) => s.trim(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    debug_print!(
        "[DEBUG] Query -> plugin: '{}', lang: '{}', key: '{}'",
        plugin_name,
        lang_code,
        key
    );

    let cache = get_plugin_lang_cache();

    let result_text = if let Some(lang_map) = cache.get(&plugin_name) {
        let content = lang_map
            .get(&lang_code)
            .or_else(|| {
                let primary = lang_code.split('-').next().unwrap_or("");
                lang_map
                    .iter()
                    .find(|(k, _)| k.starts_with(primary))
                    .map(|(_, v)| v)
            })
            .or_else(|| lang_map.get("ja-jp"))
            .or_else(|| lang_map.get("ja"))
            .or_else(|| lang_map.values().next());

        match content {
            Some(c) => parse_key_from_content(c, key),
            None => {
                debug_print!(
                    "[DEBUG] Language file content not matched for plugin: {}",
                    plugin_name
                );
                format!("[Plugin Language File Not Found: {}]", plugin_name)
            }
        }
    } else {
        debug_print!(
            "[DEBUG] Plugin NOT found in cache! Cache keys: {:?}",
            cache.keys().collect::<Vec<_>>()
        );
        format!("[Plugin Not Found: {}]", plugin_name)
    };

    debug_print!("[DEBUG] Result translation text: '{}'", result_text);

    let clean_text = result_text.replace('\0', "");
    match CString::new(clean_text) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn parse_translation(
    content_ptr: *const c_char,
    key_ptr: *const c_char,
) -> *mut c_char {
    if content_ptr.is_null() || key_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let content = unsafe {
        match CStr::from_ptr(content_ptr).to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let key = unsafe {
        match CStr::from_ptr(key_ptr).to_str() {
            Ok(s) => s.trim(),
            Err(_) => return std::ptr::null_mut(),
        }
    };

    let translation = parse_key_from_content(content, key);
    let clean_translation = translation.replace('\0', "");

    match CString::new(clean_translation) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
