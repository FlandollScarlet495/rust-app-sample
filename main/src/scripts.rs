use mlua::{Lua, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 実行ファイルと同じ場所、またはカレントディレクトリから `scripts` ディレクトリを取得する
fn get_scripts_dir() -> PathBuf {
    // 1. 実行ファイル (main.exe) と同じ階層の scripts を探す
    if let Ok(mut exe_path) = env::current_exe() {
        exe_path.pop(); // main.exe のファイル名を除去
        let exe_scripts_dir = exe_path.join("scripts");
        if exe_scripts_dir.exists() {
            return exe_scripts_dir;
        }
    }

    // 2. カレントディレクトリの scripts を探す（フォールバック）
    PathBuf::from("scripts")
}

/// `scripts/` ディレクトリ配下の全 `.lua` ファイルを再帰的に走査して実行する
pub fn run_all_scripts() -> Result<()> {
    // Lua インスタンスの作成
    let lua = Lua::new();

    // スクリプト側から呼び出せる Rust 側の関数を登録
    let print_hello = lua.create_function(|_, name: String| {
        println!("[Rust Native Call] こんにちは、{}ちゃん！", name);
        Ok(())
    })?;
    lua.globals().set("rust_hello", print_hello)?;

    let scripts_dir = get_scripts_dir();
    if !scripts_dir.exists() {
        println!("'{}' ディレクトリが見つかりません。", scripts_dir.display());
        return Ok(());
    }

    // scripts/ 配下を再帰的に走査
    for entry in WalkDir::new(&scripts_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // 拡張子が .lua のファイルのみ実行
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("lua") {
            println!("\n--- Executing Script: {:?} ---", path);

            let script_content = fs::read_to_string(path)
                .map_err(|e| mlua::Error::ExternalError(std::sync::Arc::new(e)))?;

            // スクリプトの読み込みと実行
            if let Err(err) = lua
                .load(&script_content)
                .set_name(path.to_string_lossy())
                .exec()
            {
                eprintln!("Failed to execute script {:?}: {}", path, err);
            }
        }
    }

    Ok(())
}
