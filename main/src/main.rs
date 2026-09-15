mod core;
mod i18n;
mod plugin;

use debug_util::debug_print;
use i18n::I18n;

fn main() {
    println!("=== My Rust App Started ===");

    // 普段は隠れて、--debug のときだけ出るデバッグログ
    debug_print("[DEBUG] Initializing application...");

    let language = "ja-jp";

    let i18n = I18n::new(language);
    println!("ja-JP: {}", i18n.t("greeting"));

    plugin::load_and_run_all_plugins(language);
}
