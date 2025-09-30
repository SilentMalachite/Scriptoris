# CLAUDE.md

> あなたはプロのAIプロンプトエンジニア兼フルスタックRust開発者として行動します。以下の要件・制約・タスク分解に厳密に従い、**Rust + Ratatui** によるクロスプラットフォーム（Windows/macOS/Linux）対応の**ターミナル Markdown エディタ**を実装・検証してください。

---

## 目的

- Rust + Ratatui を用いて、Vim風操作でMarkdownファイルを効率的に編集できるターミナルアプリを開発する。
- **日本語文字（Unicode）**の表示・編集に対応し、**Windows / macOS / Linux** で動作すること。
- **ファイルの保存・読み込み**、基本的な**シンタックスハイライト**、および **設定保存**機能を備える。

## 成果物

- アプリ本体のソースコード（Rust ワークスペース構成）
- クロスプラットフォーム対応のバイナリファイル
- ユニットテストと基本的な動作確認手順
- 開発・ビルド・使用方法のドキュメント（README）

---

## 技術スタック & 方針

- **言語**: Rust 1.32+（MSRVは `rust-toolchain.toml` で明示）
- **TUIフレームワーク**: Ratatui 0.26 + Crossterm 0.27
- **テキスト処理**: Ropey 1.6（効率的なテキストデータ構造）
- **Markdown処理**: `comrak`（GFM拡張: 表/脚注/打消し/タスクリスト）- mdcoreクレート
- **シンタックスハイライト**: `syntect` 5.0
- **非同期処理**: `tokio` 1.32（ファイルI/O、設定保存）
- **文字処理**: `unicode-width`, `unicode-segmentation`（日本語対応）
- **設定管理**: `serde`/`serde_json`（設定保存）+ `directories`（設定ファイル格納）
- **テスト**: `cargo test`（ユニット）
- **エラーハンドリング**: `anyhow` 1.0

---

## 現在の実装状態

### 実装済み機能
1. **基本的なターミナルUI**
   - Ratatui使用のTUIアプリケーション
   - タイトルバー、エディタ領域、ステータスバー
   - ヘルプモーダル表示

2. **Vim風操作モード**
   - Normal / Insert / Command / Help モード
   - 基本的なカーソル移動（h/j/k/l）
   - テキスト挿入・削除

3. **テキスト編集**
   - Ropeyベースの効率的なテキスト処理
   - Unicode文字対応（日本語表示可能）
   - 行番号表示、カーソル位置表示

4. **ファイル操作**
   - ファイル読み込み・保存機能の基盤
   - UTF-8エンコーディング対応

5. **設定機能**
   - JSON形式での設定保存
   - directories crateでクロスプラットフォーム対応

### Markdown処理（mdcoreクレート）
- comrakベースのGFM対応HTML生成
- 表、脚注、タスクリスト、打消し線サポート
- 数式・Mermaidブロック検出（将来の拡張用）
- HTMLサニタイゼーション機能

---

## ディレクトリ構成（現在）

```
Scriptoris/
├─ Cargo.toml                # workspace ルート
├─ rust-toolchain.toml
├─ README.md
├─ CLAUDE.md                 # 本ファイル
├─ crates/
│  ├─ scriptoris/            # メインTUIアプリケーション
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     ├─ main.rs          # エントリポイント、ターミナル初期化
│  │     ├─ app.rs           # アプリケーション構造、モード管理
│  │     ├─ editor.rs        # テキスト編集ロジック、Ropey統合
│  │     ├─ ui.rs            # UI描画、レイアウト管理
│  │     ├─ keybindings.rs   # キーバインド処理
│  │     └─ config.rs        # 設定管理
│  └─ mdcore/                # Markdown処理ライブラリ
│     ├─ Cargo.toml
│     └─ src/
│         ├─ lib.rs
│         ├─ markdown.rs     # comrak設定/HTML生成
│         ├─ sanitize.rs     # HTMLサニタイズ
│         └─ tests.rs        # テストケース
└─ assets/                   # 静的ファイル（将来用）
   └─ imports/               # 外部ファイル保存先
```

---

## 主要機能（現在の受け入れ基準）

1. **Vim風エディタ操作**
   - Normal: h/j/k/l移動、i挿入モード、:コマンドモード、?ヘルプ
   - Insert: 通常の文字入力、Escで Normal モードへ
   - Command: :w保存、:q終了、:wq保存終了
   - Help: キーバインド一覧表示

2. **テキスト編集**
   - 日本語（Unicode）文字の正しい表示・入力
   - 行番号表示、カーソル位置表示
   - ファイル変更検知（modified状態）
   - 基本的なクリップボード操作

3. **ファイル操作**
   - ファイル読み込み（コマンドライン引数 or :e コマンド）
   - 保存機能（:w, :wq コマンド）
   - UTF-8エンコーディング対応
   - ファイル変更確認

4. **UI/UX**
   - クロスプラットフォーム対応のターミナル描画
   - 最低限のシンタックスハイライト（将来実装）
   - ステータスバー（行数、文字数、モード表示）

5. **設定**
   - JSON形式での設定ファイル
   - キーバインドカスタマイズ（将来実装）
   - テーマ設定（将来実装）

---

## 実装ガイド

### 1) アプリケーション構造

```rust
// crates/scriptoris/src/app.rs
pub struct App {
    pub editor: Editor,      // テキスト編集状態
    pub mode: Mode,         // Normal/Insert/Command/Help
    pub config: Config,     // 設定情報
    pub file_path: Option<PathBuf>, // 現在のファイル
    pub status_message: String,     // ステータス表示
    pub show_help: bool,    // ヘルプ表示フラグ
    pub command_buffer: String,     // コマンド入力バッファ
    pub should_quit: bool,  // アプリ終了フラグ
}

pub enum Mode {
    Normal,   // カーソル移動、コマンド
    Insert,   // テキスト挿入
    Command,  // :コマンド入力
    Help,     // ヘルプ表示
}
```

### 2) テキスト編集（Ropey）

```rust
// crates/scriptoris/src/editor.rs
pub struct Editor {
    rope: Rope,              // 効率的なテキスト表現
    cursor_line: usize,      // カーソル行
    cursor_col: usize,       // カーソル列
    viewport_offset: usize,  // スクロール位置
    viewport_height: usize,  // 表示可能行数
    modified: bool,          // 変更フラグ
    clipboard: String,       // 簡易クリップボード
}
```

### 3) Markdown処理

```rust
// crates/mdcore/src/markdown.rs
use comrak::{markdown_to_html, ComrakOptions};

pub fn to_html(src: &str) -> String {
    let mut opt = ComrakOptions::default();
    opt.extension.table = true;
    opt.extension.strikethrough = true;
    opt.extension.tasklist = true;
    opt.extension.footnotes = true;
    opt.parse.smart = true;
    // セキュリティ対策: unsafe HTML無効化
    opt.render.unsafe_ = false;
    opt.render.escape = true;
    markdown_to_html(src, &opt)
}
```

### 4) UI描画

```rust
// crates/scriptoris/src/ui.rs
pub fn draw(frame: &mut Frame, app: &App) {
    // タイトルバー、エディタ、ステータスバーの描画
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),      // タイトル
            Constraint::Min(1),         // エディタ
            Constraint::Length(1),      // ステータス
        ])
        .split(frame.size());
}
```

---

## 開発タスク（優先順）

### 1. 基本機能の完成（高優先）
- [ ] ファイル保存・読み込みの完全実装
- [ ] Vim操作の拡張（削除、ヤンク、検索など）
- [ ] エラーハンドリングの強化
- [ ] 設定ファイルの読み書き

### 2. エディタ機能の向上（中優先）
- [ ] 基本的なシンタックスハイライト
- [ ] 検索・置換機能
- [ ] Undo/Redo機能
- [ ] 複数行操作

### 3. Markdown対応（中優先）
- [ ] Markdown文法のハイライト
- [ ] プレビュー機能（別ペインまたは外部）
- [ ] 表・リンク編集支援

### 4. UX改善（低優先）
- [ ] テーマ・カラースキーム
- [ ] 設定のカスタマイズUI
- [ ] 最近使ったファイル（MRU）
- [ ] ファイルブラウザ

---

## ビルド & 実行

```bash
# 開発実行
cargo run

# 特定ファイルを開く
cargo run -- path/to/file.md

# リリースビルド
cargo build --release

# テスト実行
cargo test
```

---

## テスト観点

- **テキスト処理**: 日本語文字、Unicode、改行コードの処理
- **ファイル操作**: UTF-8読み書き、エラーハンドリング、パーミッション
- **UI操作**: キーバインド、モード遷移、エラー状態の表示
- **Markdown**: GFM構文の正しいHTML変換、サニタイゼーション
- **クロスプラットフォーム**: Windows/macOS/Linuxでの動作確認

---

## 最近の修正（2024年）

1. **LSPプラグイン**: UTF-16オフセット計算の修正、メモリリーク対策
2. **Unicode処理**: グラフェムクラスタ対応、絵文字・結合文字の正確な処理
3. **エラーハンドリング**: タイムアウト処理、指数バックオフリトライ
4. **テストカバレッジ**: LSP、enhanced UI、日本語テキスト処理のテスト追加

## 現在の制限事項

1. **プレビュー機能なし**: ターミナルアプリのため、リアルタイムHTML表示は不可
2. **高度なVim機能一部未実装**: より多くのレジスタ、複雑なマクロなど
3. **プラグイン機能限定的**: LSP以外の拡張性は制限あり
4. **画像表示不可**: ターミナルの制約
5. **数式・図表**: ASCII形式以外の表示は困難

---

## 将来の拡張可能性

- 外部プレビューアとの連携
- LSP（Language Server Protocol）統合によるMarkdown支援
- Git統合機能
- セッション管理（タブ、分割ウィンドウ）
- コラボレーション機能

---

## 期待するClaudeの振る舞い

1. 現在の実装状態を理解した上で、**段階的な改善**を実施
2. **セキュリティ** を重視し、適切なサニタイゼーションを実装
3. **パフォーマンス** を考慮した効率的なコード
4. **テスト可能** な設計とモジュール分割
5. クロスプラットフォーム対応の**互換性**確保

---

以上。