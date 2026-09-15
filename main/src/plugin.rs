use debug_util::debug_print;
use libloading::{Library, Symbol};
use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::os::raw::c_char;
use std::path::PathBuf;

/// 実行ファイルのあるディレクトリを基準にしたパスを取得するヘルパー
fn get_base_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// core ライブラリ経由で指定プラグインの言語メッセージを取得
fn get_plugin_translation(plugin_name: &str, lang_code: &str, key: &str) -> String {
    let base_dir = get_base_dir();

    // OSに応じた core ライブラリ名（Windows: core.dll, Linux: libcore.so, macOS: libcore.dylib）を生成
    let core_lib_name = format!("{}core{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX);
    let core_path = base_dir.join("libraries").join(&core_lib_name);

    let lib = match unsafe { Library::new(&core_path) } {
        Ok(l) => l,
        Err(_) => return format!("[Failed to load {} at {:?}]", core_lib_name, core_path),
    };

    unsafe {
        let func: Symbol<
            unsafe extern "C" fn(*const c_char, *const c_char, *const c_char) -> *mut c_char,
        > = match lib.get(b"get_plugin_translation") {
            Ok(f) => f,
            Err(_) => return format!("[Symbol not found: get_plugin_translation]"),
        };

        let c_plugin = CString::new(plugin_name).unwrap_or_default();
        let c_lang = CString::new(lang_code).unwrap_or_default();
        let c_key = CString::new(key).unwrap_or_default();

        let ptr = func(c_plugin.as_ptr(), c_lang.as_ptr(), c_key.as_ptr());
        if ptr.is_null() {
            return format!("[Translation Error: {}]", key);
        }

        let result = CStr::from_ptr(ptr).to_string_lossy().into_owned();

        if let Ok(free_func) = lib.get::<unsafe extern "C" fn(*mut c_char)>(b"free_string") {
            free_func(ptr);
        }

        result
    }
}

/// plugins/ フォルダ内のすべての動的ライブラリを自動スキャンして一括実行
pub fn load_and_run_all_plugins(lang_code: &str) {
    let base_dir = get_base_dir();
    let plugins_dir = base_dir.join("plugins");

    let entries = match fs::read_dir(&plugins_dir) {
        Ok(e) => e,
        Err(_) => {
            debug_print(&format!(
                "[DEBUG] Failed to read plugins dir: {:?}",
                plugins_dir
            ));
            return;
        }
    };

    // 現在のOSの動的ライブラリ拡張子（例: ".dll", ".so", ".dylib"）
    let target_ext = env::consts::DLL_SUFFIX.trim_start_matches('.');

    for entry in entries.flatten() {
        let path = entry.path();

        // 該当OSの動的ライブラリファイルのみを対象にする
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some(target_ext) {
            // ファイル名からプラグイン名を取得
            // Linux/Macで "libplugin_sample.so" となっている場合は "lib" を除外して "plugin_sample" に変換
            let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(stem) => stem,
                None => continue,
            };

            let plugin_name = if !env::consts::DLL_PREFIX.is_empty()
                && file_stem.starts_with(env::consts::DLL_PREFIX)
            {
                &file_stem[env::consts::DLL_PREFIX.len()..]
            } else {
                file_stem
            };

            // 1. core ライブラリから該当プラグインの翻訳テキストを取得
            let msg = get_plugin_translation(plugin_name, lang_code, "plugin_msg");
            let c_msg = CString::new(msg).unwrap_or_default();

            // 2. プラグイン動的ライブラリをロード
            if let Ok(lib) = unsafe { Library::new(&path) } {
                unsafe {
                    // エクスポート関数 `plugin_hello` があれば実行
                    if let Ok(func) =
                        lib.get::<Symbol<unsafe extern "C" fn(*const c_char)>>(b"plugin_hello")
                    {
                        func(c_msg.as_ptr());
                    }
                }
            }
        }
    }
}
