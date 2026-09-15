use debug_util::debug_print;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::Path;
use std::sync::OnceLock;

// DLL インスタンスのキャッシュ
static CORE_LIB: OnceLock<Option<Library>> = OnceLock::new();

// 読み込んだ言語ファイルデータ (ロケールコード -> ファイルの中身) のキャッシュ
static LANG_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

fn get_core_lib() -> Option<&'static Library> {
    CORE_LIB
        .get_or_init(|| {
            // OSに応じたライブラリ名（Windows: core.dll, Linux: libcore.so, macOS: libcore.dylib）を生成
            let lib_name = format!(
                "{}core{}",
                std::env::consts::DLL_PREFIX,
                std::env::consts::DLL_SUFFIX
            );
            let dll_path = Path::new("libraries").join(&lib_name);
            debug_print(&format!(
                "[DEBUG] Loading core library from: {:?}",
                dll_path
            ));
            unsafe { Library::new(&dll_path).ok() }
        })
        .as_ref()
}

/// Languages フォルダ内の *.lang ファイルを全自動スキャンして HashMap にキャッシュする
fn get_lang_cache() -> &'static HashMap<String, String> {
    LANG_CACHE.get_or_init(|| {
        let mut map = HashMap::new();
        let lang_dir = Path::new("Languages");

        if let Ok(entries) = fs::read_dir(lang_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // .lang 拡張子のファイルのみを対象とする
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lang") {
                    if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            // ファイル名（例: "ja-JP", "en_US"）を小文字・ハイフン区切りに正規化して登録
                            let key = normalize_lang_code(file_stem);
                            debug_print(&format!(
                                "[DEBUG] Loaded lang file: {:?} -> normalized key: '{}'",
                                path, key
                            ));
                            map.insert(key, content);
                        }
                    }
                }
            }
        } else {
            debug_print(&format!(
                "[DEBUG] WARNING: Failed to read Languages dir: {:?}",
                lang_dir
            ));
        }
        map
    })
}

/// ロケール表記の正規化（小文字化 ＆ アンダースコアをハイフンに統一）
fn normalize_lang_code(code: &str) -> String {
    code.to_lowercase().replace('_', "-")
}

pub fn get_text(lang_code: &str, key: &str) -> String {
    let cache = get_lang_cache();
    let target_code = normalize_lang_code(lang_code);

    debug_print(&format!(
        "[DEBUG] Core Query -> lang: '{}', key: '{}'",
        target_code, key
    ));

    // 1. 完全一致（例: "ja-jp"）
    // 2. プライマリ言語コード一致（例: "ja-jp" が無ければ "ja" で始まるものを検索）
    // 3. フォールバック（"ja-jp"、またはキャッシュ内の先頭データのいずれか）
    let lang_data = cache
        .get(&target_code)
        .or_else(|| {
            let primary = target_code.split('-').next().unwrap_or("");
            cache
                .iter()
                .find(|(k, _)| k.starts_with(primary))
                .map(|(_, v)| v)
        })
        .or_else(|| cache.get("ja-jp"))
        .or_else(|| cache.values().next());

    let lang_data = match lang_data {
        Some(data) => data,
        None => {
            debug_print("[DEBUG] Language files not found in cache!");
            return "Language files not found".to_string();
        }
    };

    // キャッシュされた DLL インスタンスを取得
    let lib = match get_core_lib() {
        Some(l) => l,
        None => {
            let lib_name = format!(
                "{}core{}",
                std::env::consts::DLL_PREFIX,
                std::env::consts::DLL_SUFFIX
            );
            let err_msg = format!("Failed to load libraries/{}", lib_name);
            debug_print(&format!("[DEBUG] {}", err_msg));
            return err_msg;
        }
    };

    unsafe {
        let func: Symbol<unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char> =
            match lib.get(b"parse_translation") {
                Ok(f) => f,
                Err(_) => {
                    debug_print("[DEBUG] Failed to find symbol parse_translation");
                    return "Failed to find symbol parse_translation".to_string();
                }
            };

        let c_data = CString::new(lang_data.as_str()).unwrap();
        let c_key = CString::new(key).unwrap();
        let result_ptr = func(c_data.as_ptr(), c_key.as_ptr());

        if result_ptr.is_null() {
            debug_print("[DEBUG] Translation error (result pointer is null)");
            return "Translation error".to_string();
        }

        let c_str = CStr::from_ptr(result_ptr);
        let s = c_str.to_string_lossy().into_owned();

        if let Ok(free_func) = lib.get::<unsafe extern "C" fn(*mut c_char)>(b"free_string") {
            free_func(result_ptr);
        }

        debug_print(&format!("[DEBUG] Core Result: '{}'", s));
        s
    }
}
