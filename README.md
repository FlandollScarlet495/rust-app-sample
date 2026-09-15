# Rust App Sample

![Version](https://img.shields.io/badge/version-v0.0.1--alpha.2-blue)
![License](https://img.shields.io/badge/license-MIT-green)

このリポジトリは、Rust のワークスペース構成で、コアライブラリ・ユーティリティ・プラグイン・ローカライズ対応を組み合わせたサンプルアプリケーションです。

主な特徴:

- Rust workspace で複数クレートを管理
- 動的ライブラリのロードと実行
- 共通の言語ファイルによる i18n
- プラグインごとの言語ファイル対応
- `xtask` による複数ターゲットへのビルド・自動クリーンアップ・成果物配置

## 構成

- `main/` : アプリ本体
- `libraries/core/` : コア機能と翻訳処理
- `libraries/debug_util/` : デバッグ出力ユーティリティ
- `plugins/` : プラグイン群
- `Languages/` : 共通言語ファイル
- `PluginsLanguages/` : プラグイン固有の言語ファイル
- `xtask/` : `dist/` 配布用のビルド補助

## 必要条件

- Rust toolchain
- `cargo`
- `cargo-zigbuild` と `zig`（マルチプラットフォーム向けビルド時）

## 動作イメージ

アプリ起動時に以下のような動作をします。

1. `main` が起動
2. 共通辞書から `greeting` を取得
3. プラグインを自動スキャンして読み込み
4. プラグイン固有のメッセージを出力

## 生成物の配布 (`xtask`)

配布用アーティファクトを `dist/` 配下にまとめたい場合、以下のコマンドを実行します。

```bash
# 全ターゲットをビルド
cargo run -p xtask

```

※ ビルド開始時に、前回の `dist/` ディレクトリおよび `target/*/release` 配下のキャッシュ成果物は一括して完全削除・クリアされます。

### エイリアス指定とフィルタリング

引数としてエイリアス名を渡すことで、特定のターゲットのみのビルドや除外指定が可能です。

- **通常指定 (包含):**
指定したエイリアス文字列を含むターゲットのみをビルドします。

```bash
cargo run -p xtask -- win
cargo run -p xtask -- win-x86_64 darwin-arm64

```

- **逆指定 (除外):**
文字列の先頭に `!` または `^` を付けることで、該当するエイリアスを除外します。

```bash
# Windows 以外のターゲットをビルド
cargo run -p xtask -- !win

# Linux のうち arm 以外をビルド
cargo run -p xtask -- linux !arm

```

### 出力構成イメージ

`dist/` に以下のような構成で成果物が生成されます。

```text
dist/
  windows/
    x86_64/
      main.exe
      libraries/
      plugins/
      Languages/
      PluginsLanguages/

```

## 主要ファイル

- `main/src/main.rs` : アプリの入口
- `main/src/i18n.rs` : 言語ラッパー
- `main/src/plugin.rs` : プラグインの読み込みと実行
- `libraries/core/src/core.rs` : 翻訳とプラグイン言語辞書の処理
- `plugins/plugin_sample/src/plugin_sample.rs` : サンプルプラグイン
- `Languages/ja-jp.lang` : 共通メッセージ
- `PluginsLanguages/plugin_sample/ja-jp.lang` : プラグイン用メッセージ

## 注意事項

- 実行時に必要な DLL / .so / .dylib は `dist/` 配下に配置される前提で、プラグイン側は実行ファイルの近くにある `libraries` や `plugins` を探します。
- ローカライズは `lang_code` を正規化して、完全一致→主要言語一致→フォールバックの順で解決します。
- `--debug` または `-d` を付けると、各処理の内部ログが出力されます。

## リリース

最新のプレリリース、ソースコード、およびバイナリ成果物は [Releases](https://github.com/FlandollScarlet495/rust-app-sample/releases) ページから確認・ダウンロードできます。

## ライセンス

MIT License

Copyright (c) 2026 霧島 葵 ( きりしま あおい ) ( Aoi Kirishima )

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
