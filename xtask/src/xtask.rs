use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

struct TargetConfig {
    alias: &'static str,      // ショートカット引数名 (例: win-x86_64)
    os_dir: &'static str,     // dist 以下のフォルダ名
    triple: &'static str,     // cargo/cargo-zigbuild に渡すターゲットトリプル
    exe_ext: &'static str,    // 実行ファイルの拡張子 ("exe" または "")
    lib_prefix: &'static str, // 動的ライブラリの接頭辞 ("lib" または "")
    lib_suffix: &'static str, // 動的ライブラリの拡張子 ("dll", "dylib", "so")
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // target/*/release ディレクトリを走査して一括削除
    println!("Cleaning release build artifacts (target/*/release)...");
    clean_target_release_dirs(Path::new("target"));

    // cargo clean --release も併せて実行
    let _ = Command::new("cargo").args(["clean", "--release"]).status();

    // 毎回ビルド開始前に dist ディレクトリを一度完全に削除してクリアする
    let out_dir = Path::new("dist");
    if out_dir.exists() {
        println!("Cleaning up existing 'dist' directory...");
        if let Err(e) = fs::remove_dir_all(out_dir) {
            eprintln!("Warning: Failed to remove 'dist' directory: {}", e);
        }
    }

    let macos_sdk_path = r"C:\sdks\MacOSX11.3.sdk";
    let all_targets = vec![
        // --- Windows ---
        TargetConfig {
            alias: "windows-x86_64",
            os_dir: "windows/x86_64",
            triple: "x86_64-pc-windows-msvc",
            exe_ext: "exe",
            lib_prefix: "",
            lib_suffix: "dll",
        },
        TargetConfig {
            alias: "windows-x86",
            os_dir: "windows/x86",
            triple: "i686-pc-windows-msvc",
            exe_ext: "exe",
            lib_prefix: "",
            lib_suffix: "dll",
        },
        TargetConfig {
            alias: "windows-arm64",
            os_dir: "windows/arm64",
            triple: "aarch64-pc-windows-msvc",
            exe_ext: "exe",
            lib_prefix: "",
            lib_suffix: "dll",
        },
        // --- macOS (Darwin) ---
        TargetConfig {
            alias: "darwin-x86_64",
            os_dir: "darwin/x86_64",
            triple: "x86_64-apple-darwin",
            exe_ext: "",
            lib_prefix: "lib",
            lib_suffix: "dylib",
        },
        TargetConfig {
            alias: "darwin-arm64",
            os_dir: "darwin/arm64",
            triple: "aarch64-apple-darwin",
            exe_ext: "",
            lib_prefix: "lib",
            lib_suffix: "dylib",
        },
        // --- Linux ---
        TargetConfig {
            alias: "linux-x86_64",
            os_dir: "linux/x86_64",
            triple: "x86_64-unknown-linux-gnu",
            exe_ext: "",
            lib_prefix: "lib",
            lib_suffix: "so",
        },
        TargetConfig {
            alias: "linux-x86",
            os_dir: "linux/x86",
            triple: "i686-unknown-linux-gnu",
            exe_ext: "",
            lib_prefix: "lib",
            lib_suffix: "so",
        },
        TargetConfig {
            alias: "linux-arm64",
            os_dir: "linux/arm64",
            triple: "aarch64-unknown-linux-gnu",
            exe_ext: "",
            lib_prefix: "lib",
            lib_suffix: "so",
        },
        TargetConfig {
            alias: "linux-arm32",
            os_dir: "linux/arm32",
            triple: "armv7-unknown-linux-gnueabihf",
            exe_ext: "",
            lib_prefix: "lib",
            lib_suffix: "so",
        },
    ];

    // 引数解析ロジック（エイリアス限定・逆指定対応）
    let (exclude_args, include_args): (Vec<String>, Vec<String>) = args
        .into_iter()
        .partition(|arg| arg.starts_with('!') || arg.starts_with('^'));

    // 除外用パターン（先頭の ! や ^ を除外して小文字化）
    let excludes: Vec<String> = exclude_args.iter().map(|s| s[1..].to_lowercase()).collect();

    // 包含用パターン（小文字化）
    let includes: Vec<String> = include_args.iter().map(|s| s.to_lowercase()).collect();

    let targets: Vec<&TargetConfig> = all_targets
        .iter()
        .filter(|t| {
            let alias_lower = t.alias.to_lowercase();

            // 1. 除外条件にヒットした場合は無条件で除外
            if excludes.iter().any(|ex| alias_lower.contains(ex)) {
                return false;
            }

            // 2. 包含条件の判定（指定なしなら全許可、指定ありならいずれかに一致）
            if includes.is_empty() {
                true
            } else {
                includes.iter().any(|inc| alias_lower.contains(inc))
            }
        })
        .collect();

    if targets.is_empty() {
        eprintln!("Error: No matching targets found for provided filter criteria.");
        return;
    }

    for target in targets {
        println!("\n==========================================");
        println!(" Building for target: {} ({})", target.triple, target.alias);
        println!("==========================================");

        // ビルド前に出力先ディレクトリ構造を作成してリンク警告を防止
        let dist_os_dir = out_dir.join(target.os_dir);
        let lib_dir = dist_os_dir.join("libraries");
        let plugin_dir = dist_os_dir.join("plugins");
        let lang_dir = dist_os_dir.join("Languages");
        let plugin_lang_dir = dist_os_dir.join("PluginsLanguages");
        let scripts_dir = dist_os_dir.join("scripts");

        fs::create_dir_all(&lib_dir).ok();
        fs::create_dir_all(&plugin_dir).ok();
        fs::create_dir_all(&lang_dir).ok();
        fs::create_dir_all(&plugin_lang_dir).ok();
        fs::create_dir_all(&scripts_dir).ok();

        let mut cmd = Command::new("cargo");

        if target.triple.contains("-msvc") {
            cmd.args([
                "build",
                "--workspace",
                "--release",
                "--target",
                target.triple,
            ]);
        } else {
            cmd.args([
                "zigbuild",
                "--workspace",
                "--release",
                "--target",
                target.triple,
            ]);

            cmd.env("CC", "zig cc -w");
            cmd.env("CXX", "zig c++ -w");
        }

        if target.triple.contains("apple-darwin") {
            cmd.env("SDKROOT", macos_sdk_path);
        }

        let status = cmd.status();

        match status {
            Ok(s) if s.success() => {
                println!("Build succeeded for {}", target.triple);
            }
            _ => {
                eprintln!(
                    "Warning: Build failed for {}. Skipping deployment for this target.",
                    target.triple
                );
                continue;
            }
        }

        // --- 成果物のコピー処理 ---
        let triple_base = target.triple.split('.').next().unwrap_or(target.triple);
        let target_release_dir = Path::new("target").join(triple_base).join("release");

        let main_exe_name = if target.exe_ext.is_empty() {
            "main".to_string()
        } else {
            format!("main.{}", target.exe_ext)
        };
        let src_main_path = target_release_dir.join(&main_exe_name);
        if src_main_path.exists() {
            let _ = fs::copy(&src_main_path, dist_os_dir.join(&main_exe_name));
            println!("Copied binary: {:?}", main_exe_name);
        }

        copy_libs_from_dir(
            Path::new("libraries"),
            &target_release_dir,
            &lib_dir,
            target,
        );
        copy_libs_from_dir(
            Path::new("plugins"),
            &target_release_dir,
            &plugin_dir,
            target,
        );

        let src_lang = Path::new("Languages");
        if src_lang.exists() {
            copy_languages_dir(src_lang, &lang_dir);
        }

        let src_plugin_lang = Path::new("PluginsLanguages");
        if src_plugin_lang.exists() {
            copy_dir_all(src_plugin_lang, &plugin_lang_dir);
        }

        // プロジェクト直下の scripts/ を dist/*/scripts/ 配下に再帰コピー
        let src_scripts = Path::new("scripts");
        if src_scripts.exists() {
            copy_dir_all(src_scripts, &scripts_dir);
            println!("Copied scripts directory to {:?}", scripts_dir);
        }
    }

    println!("\nSUCCESS! Artifacts deployed to 'dist/'");
}

// target ディレクトリ以下の全 release フォルダを検索・削除する関数
fn clean_target_release_dirs(dir: &Path) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|n| n.to_str()) == Some("release") {
                    let _ = fs::remove_dir_all(&path);
                } else {
                    clean_target_release_dirs(&path);
                }
            }
        }
    }
}

fn copy_libs_from_dir(
    src_parent: &Path,
    target_dir: &Path,
    dest_dir: &Path,
    target: &TargetConfig,
) {
    if let Ok(entries) = fs::read_dir(src_parent) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let folder_name = entry.file_name();
                let lib_name = format!(
                    "{}{}.{}",
                    target.lib_prefix,
                    folder_name.to_string_lossy(),
                    target.lib_suffix
                );
                let lib_path = target_dir.join(&lib_name);

                if lib_path.exists() {
                    let _ = fs::copy(&lib_path, dest_dir.join(&lib_name));
                    println!("Copied library: {:?}", lib_name);
                }
            }
        }
    }
}

fn copy_languages_dir(src_dir: &Path, dest_dir: &Path) {
    if let Ok(entries) = fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let dest_path = dest_dir.join(file_name);
                    let _ = fs::copy(&path, &dest_path);
                }
            }
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) {
    if let Ok(entries) = fs::read_dir(src) {
        let _ = fs::create_dir_all(dst);
        for entry in entries.flatten() {
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if path.is_dir() {
                copy_dir_all(&path, &dest_path);
            } else if path.is_file() {
                let _ = fs::copy(&path, &dest_path);
            }
        }
    }
}
