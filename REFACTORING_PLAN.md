# FLACミュージックプレイヤー リファクタリング実行計画

## 概要
このドキュメントは、FLACミュージックプレイヤーのコードベースをより保守しやすく、拡張しやすい構造にリファクタリングするための実行計画です。

## 現状の問題点

### 1. ファイルサイズの問題
- `main_old.rs`: 1,413行 - **削除対象**
- `playlist/manager.rs`: 1,040行 - **分割必要**
- `app/ui_playlist.rs`: 990行 - **分割必要**
- `ui/playback_controls.rs`: 884行 - **分割必要**

### 2. MyApp構造体の肥大化
- 23個のパブリックフィールド
- 責任の分散（UI状態、選択状態、アプリ状態が混在）
- 初期化メソッド`new()`が45行

### 3. 重複・不要ファイル
- `main_old.rs` / `main_refactored.rs` - 未使用
- `ui/music_tree.rs` vs `ui/music_tree_simple.rs` - 機能重複

### 4. エラーハンドリングの不統一
- `unwrap_or_else`での隠蔽が多用
- 適切な`Result`型の活用不足

## リファクタリング実行計画

### フェーズ1: クリーンアップ（低リスク）

#### 1.1 不要ファイルの削除 ⏱️ 30分
- [ ] `src/main_old.rs` 削除
- [ ] `src/main_refactored.rs` 削除  
- [ ] 使用されていないimportの除去
- [ ] コンパイル確認

#### 1.2 重複UIコンポーネントの統合 ⏱️ 1時間
- [ ] `ui/music_tree.rs` と `ui/music_tree_simple.rs` の比較分析
- [ ] より適切な実装を選択し、もう片方を削除
- [ ] 関連するimportの更新

### フェーズ2: 構造体リファクタリング（中リスク）

#### 2.1 MyApp構造体の分割 ⏱️ 3時間

**新構造体の設計:**
```rust
// UI状態管理
pub struct UIState {
    pub show_dialog: bool,
    pub current_tab: Tab,
    pub right_pane_tab: RightTab,
    pub splitter_position: f32,
    pub right_top_bottom_position: f32,
    pub right_bottom_left_right_position: f32,
    pub should_focus_controls: bool,
}

// 選択・検索状態管理  
pub struct SelectionState {
    pub selected_track: Option<TrackInfo>,
    pub selected_tracks: HashSet<PathBuf>,
    pub last_selected_path: Option<PathBuf>,
    pub search_query: String,
    pub focus_search: bool,
    pub search_has_focus: bool,
}

// プレイヤー状態管理
pub struct PlayerState {
    pub audio_player: AudioPlayer,
    pub seek_drag_state: Option<PlaybackState>,
    pub repeat_mode: RepeatMode,
    pub shuffle_enabled: bool,
}

// プレイリスト編集状態
pub struct PlaylistEditState {
    pub editing_playlist_id: Option<String>,
    pub editing_playlist_name: String,
}

// リファクタリング後のMyApp
pub struct MyApp {
    pub ui_state: UIState,
    pub selection_state: SelectionState,
    pub player_state: PlayerState,
    pub playlist_edit_state: PlaylistEditState,
    pub settings: Settings,
    pub music_library: MusicLibrary,
    pub playlist_manager: PlaylistManager,
    pub cover_art_cache: HashMap<PathBuf, TextureHandle>,
}
```

**実装手順:**
- [ ] 新構造体を`app/state.rs`に定義
- [ ] `MyApp::new()`の分割
- [ ] 既存メソッドの更新（段階的移行）
- [ ] テスト・動作確認

#### 2.2 初期化処理の分割 ⏱️ 1時間
- [ ] `MyApp::new()`を複数のヘルパー関数に分割
- [ ] 各状態の初期化を専用関数に移行
- [ ] エラーハンドリングの改善

### フェーズ3: 大型ファイルの分割（高リスク）

#### 3.1 PlaylistManager の分割 ⏱️ 4時間
```
playlist/
├── mod.rs           # 公開API
├── manager.rs       # コア管理機能（300行以内）
├── operations.rs    # CRUD操作
├── persistence.rs   # 既存（保存・読み込み）
├── shuffle.rs       # シャッフル・リピート機能
└── validation.rs    # バリデーション
```

**実装手順:**
- [ ] 現在の`manager.rs`の機能分析
- [ ] 機能ごとにファイル分割
- [ ] モジュール構造の再設計
- [ ] 段階的移行とテスト

#### 3.2 UI コンポーネントの分割 ⏱️ 3時間

**プレイリストUI分割:**
```
app/ui_playlist/
├── mod.rs              # 公開API
├── playlist_list.rs    # プレイリスト一覧
├── track_list.rs       # トラック一覧
├── edit_dialog.rs      # 編集ダイアログ
└── drag_drop.rs        # ドラッグ&ドロップ
```

**再生コントロールUI分割:**
```
ui/playback_controls/
├── mod.rs              # 公開API  
├── transport.rs        # 再生/停止/次へ等
├── seek_bar.rs         # シークバー
├── volume_control.rs   # 音量調整
└── mode_controls.rs    # リピート/シャッフル
```

- [ ] 現在の巨大UIファイルを機能別に分割
- [ ] 共通UIコンポーネントの抽出
- [ ] レスポンシビリティの明確化

### フェーズ4: エラーハンドリング統一（中リスク）

#### 4.1 カスタムエラー型の導入 ⏱️ 2時間
```rust
#[derive(Debug, thiserror::Error)]
pub enum MusicPlayerError {
    #[error("Audio playback error: {0}")]
    AudioPlayback(String),
    
    #[error("File I/O error: {0}")]
    FileIO(#[from] std::io::Error),
    
    #[error("Playlist error: {0}")]
    Playlist(String),
    
    #[error("Settings error: {0}")]
    Settings(String),
}
```

#### 4.2 Result型の統一 ⏱️ 2時間
- [ ] 主要な関数にResult型を適用
- [ ] `unwrap_or_else`の適切な置換
- [ ] エラー表示UIの改善

### フェーズ5: パフォーマンス最適化（低リスク）

#### 5.1 メモリ使用量最適化 ⏱️ 1時間
- [ ] 不要なclone()の除去
- [ ] 参照渡しの活用
- [ ] キャッシュ戦略の見直し

#### 5.2 非同期処理の導入検討 ⏱️ 3時間
- [ ] 音楽ライブラリスキャンの非同期化
- [ ] UIブロッキングの解消
- [ ] プログレス表示の改善

## 実行順序とタイムライン

| フェーズ | 期間 | リスクレベル | 優先度 |
|---------|------|------------|--------|
| フェーズ1 | 1.5時間 | 低 | 高 |
| フェーズ2 | 4時間 | 中 | 高 |
| フェーズ3 | 7時間 | 高 | 中 |
| フェーズ4 | 4時間 | 中 | 中 |
| フェーズ5 | 4時間 | 低 | 低 |

**総推定時間: 20.5時間**

## リスク管理

### 高リスク作業の注意点
- **フェーズ3**: 大型ファイル分割時は段階的移行必須
- 各ステップでコンパイル確認
- 機能テストの実施
- Gitブランチでの作業推奨

### 回帰テスト項目
- [ ] 音楽再生機能
- [ ] プレイリスト作成・編集
- [ ] 検索機能
- [ ] 設定保存・復元
- [ ] UI表示・操作性

## 成功指標

### 技術指標
- ファイルサイズ: 500行以下を目標
- 構造体フィールド: 10個以下を目標
- コンパイル時間: 現状比20%短縮
- テストカバレッジ: 70%以上

### 保守性指標
- 新機能追加時間: 現状比30%短縮
- バグ修正時間: 現状比40%短縮
- コードレビュー時間: 現状比50%短縮

## 次のステップ
1. このドキュメントのレビューと承認
2. Gitブランチ `refactoring/phase1` の作成
3. フェーズ1からの段階的実行開始

---
*最終更新: 2025-08-31*
*リファクタリング進捗は本ドキュメントで管理*