//! Scriptoris のコアライブラリ。
//!
//! このクレートはターミナルベースの Markdown エディタに必要な
//! アプリケーション状態、入出力制御、セッション管理などを提供します。
//! 各モジュールの概要:
//! - `app`: アプリケーション全体の状態管理と UI との橋渡し。
//! - `command_processor`: `:` コマンドのパーサと実行。
//! - `config`: 設定ファイルの読み書きと型定義。
//! - `editor`: Rope ベースのテキスト編集エンジン。
//! - `session_manager`: セッションの保存・復元ユーティリティ。
//! - `status_manager` / `ui_state`: ステータスバーやモード遷移の状態管理。

pub mod app;
pub mod command_processor;
pub mod config;
pub mod editor;
pub mod enhanced_ui;
pub mod file_manager;
pub mod highlight;
pub mod session_manager;
pub mod status_manager;
pub mod ui_state;

pub use app::{App, BufferManager, Mode, Plugin, PluginManager, WindowManager};
pub use config::Config;
pub use editor::Editor;
pub use session_manager::SessionManager;
