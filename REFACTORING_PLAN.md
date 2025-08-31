# FLACミュージックプレイヤー リファクタリング実行計画

## 概要
このドキュメントは、FLACミュージックプレイヤーのコードベースをより保守しやすく、拡張しやすい構造にリファクタリングするための実行計画です。

## ✅ リファクタリング実行結果 (2025-08-31更新)

### 完了済みフェーズ

**フェーズ1: クリーンアップ** ✅ **完了** (予定1.5時間 → 実際0.5時間)
- ✅ `src/main_old.rs` (1,413行) 削除
- ✅ `src/main_refactored.rs` (447行) 削除  
- ✅ `src/ui/music_tree.rs` (152行) 削除
- ✅ 未使用importの清理
- ✅ **総削除: 2,012行のコード**

**フェーズ2の準備作業** ✅ **部分完了**
- ✅ `app/state.rs` 新構造体定義完了
- ✅ コンパイル警告 24 → 21 に削減
- ✅ モジュールexportの最適化

### 学習した重要な知見

1. **段階的アプローチの重要性**: MyApp構造体を一度に全て変更すると116個のコンパイルエラーが発生
2. **小さなコミットの価値**: 各変更でコンパイル成功を維持することで安全性確保
3. **準備作業の効果**: state.rsの事前作成により将来の移行がスムーズに

## 現状の問題点 (更新済み)

### 1. 残存する大型ファイル（優先度順）
- `playlist/manager.rs`: 1,040行 - **最優先分割対象**
- `app/ui_playlist.rs`: 990行 - **高優先度**  
- `ui/playback_controls.rs`: 884行 - **中優先度**
- `player/queue.rs`: 467行 - **低優先度**

### 2. MyApp構造体の肥大化 ⚠️ **高リスク**
- 23個のパブリックフィールド
- 責任の分散（UI状態、選択状態、アプリ状態が混在）
- 初期化メソッド`new()`が45行
- **一括変更すると116個のコンパイルエラー発生を確認済み**

### 3. ~~重複・不要ファイル~~ ✅ **解決済み**
- ~~`main_old.rs` / `main_refactored.rs` - 未使用~~ → **削除完了**
- ~~`ui/music_tree.rs` vs `ui/music_tree_simple.rs` - 機能重複~~ → **統合完了**

### 4. エラーハンドリングの不統一
- `unwrap_or_else`での隠蔽が多用
- 適切な`Result`型の活用不足

### 5. 新たに判明した課題
- **21個のコンパイル警告** (未使用コード、変数等)
- 大型struct変更時の影響範囲の広さ
- テストカバレッジ不足によるリファクタリングリスク

## 🔄 更新されたリファクタリング実行計画

### ~~フェーズ1: クリーンアップ~~ ✅ **完了済み**

~~#### 1.1 不要ファイルの削除~~
- ✅ `src/main_old.rs` 削除 (1,413行)
- ✅ `src/main_refactored.rs` 削除 (447行)
- ✅ 使用されていないimportの除去
- ✅ コンパイル確認

~~#### 1.2 重複UIコンポーネントの統合~~
- ✅ `ui/music_tree.rs` と `ui/music_tree_simple.rs` の比較分析
- ✅ より適切な実装を選択し、もう片方を削除 (152行削除)
- ✅ 関連するimportの更新

### フェーズ2: 漸進的構造体リファクタリング（⚠️ 慎重アプローチ必須）

> **重要な学習**: 一括変更で116個のコンパイルエラーが発生することを確認。
> 影響範囲を最小化するため、1-2フィールドずつ段階的に移行する戦略に変更。

#### 2.1 準備作業 ⏱️ 1時間 ✅ **完了済み**
- ✅ `app/state.rs` 新構造体定義完了
- ✅ 将来の移行に向けたインフラ整備

#### 2.2 段階的フィールド移行 ⏱️ 6-8時間 (分割実行)

**移行優先順位 (リスクレベル順):**

**Step 1: UI関連フィールド移行** ⏱️ 2時間 (低リスク)
- [ ] `show_dialog` → `ui_state.show_dialog`
- [ ] `current_tab` → `ui_state.current_tab`  
- [ ] `right_pane_tab` → `ui_state.right_pane_tab`
- [ ] 各ステップでコンパイル確認

**Step 2: 検索関連フィールド移行** ⏱️ 2時間 (低リスク)
- [ ] `search_query` → `selection_state.search_query`
- [ ] `focus_search` → `selection_state.focus_search`
- [ ] `search_has_focus` → `selection_state.search_has_focus`

**Step 3: 選択関連フィールド移行** ⏱️ 2時間 (中リスク)
- [ ] `selected_track` → `selection_state.selected_track`
- [ ] `selected_tracks` → `selection_state.selected_tracks`
- [ ] `last_selected_path` → `selection_state.last_selected_path`

**Step 4: プレイヤー関連フィールド移行** ⏱️ 2時間 (高リスク)
- [ ] `audio_player` → `player_state.audio_player`
- [ ] `repeat_mode` → `player_state.repeat_mode`
- [ ] `shuffle_enabled` → `player_state.shuffle_enabled`
- [ ] `seek_drag_state` → `player_state.seek_drag_state`

**従来の設計 (参考用):**
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

### フェーズ3: 警告削減とコード品質向上（低リスク、高効果）

#### 3.1 コンパイル警告の段階的解消 ⏱️ 3-4時間
**現在の状況: 21個の警告**

**Phase A: 未使用変数・関数の処理** ⏱️ 1.5時間 (低リスク)
- [ ] 未使用変数の`_`プレフィックス付与
- [ ] 未使用`mut`の削除
- [ ] 未使用関数の削除または`#[allow(dead_code)]`付与

**Phase B: 未使用構造体・モジュールの整理** ⏱️ 1.5時間 (中リスク)
- [ ] `PlaybackQueue`全体の削除検討
- [ ] 未使用メソッドの削除
- [ ] 未使用インポートの最終清理

**Phase C: コード品質改善** ⏱️ 1時間 (低リスク)
- [ ] `Playback` variant の活用または削除
- [ ] より適切な型・ライフタイム使用

### フェーズ4: 大型ファイルの分割（高リスク → 将来実装）

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

## 🎯 更新された実行順序とタイムライン

| フェーズ | 状況 | 期間 | リスクレベル | 優先度 | 備考 |
|---------|------|------|------------|--------|------|
| ~~フェーズ1~~ | ✅ 完了 | ~~1.5時間~~ → **0.5時間** | 低 | 高 | 2,012行削除達成 |
| フェーズ2 | 🔄 進行中 | **6-8時間** | ⚠️ 高 | 高 | 段階的アプローチ採用 |
| フェーズ3 | 📋 待機中 | **3-4時間** | 低 | **最高** | 警告削減、即効性高 |
| フェーズ4 | 📋 将来検討 | 10+時間 | 高 | 中 | 安定後に実施 |

**修正された総推定時間: 10-13時間** (従来の20.5時間から半減)

## 🚨 更新されたリスク管理

### 学習した重要な教訓

1. **一括変更の危険性**: MyApp構造体の一括変更で116エラー発生 → **段階的移行が必須**
2. **小さなコミットの価値**: 各変更でコンパイル成功維持が安全性確保の鍵
3. **警告削減の効果**: 低リスクで高い品質改善効果を確認
4. **準備作業の重要性**: state.rsの事前作成で将来作業が効率化

### 推奨実行順序 (実体験ベース)

🥇 **最優先: フェーズ3 (警告削減)**  
- 低リスク、高効果
- 21 → 0 警告を目標
- コード品質の基盤作り

🥈 **次優先: フェーズ2 (段階的構造体分割)**  
- Step 1-4に分割実行
- 各Stepでコンパイル確認必須
- 1-2フィールドずつ慎重に移行

🥉 **将来検討: フェーズ4 (大型ファイル分割)**  
- 安定性確保後に実施
- 十分なテストカバレッジ必要

### 回帰テスト項目 (実証済み項目)
- ✅ コンパイル成功 (最重要)
- [ ] 音楽再生機能  
- [ ] プレイリスト作成・編集
- [ ] 検索機能
- [ ] 設定保存・復元
- [ ] UI表示・操作性

## 🎯 更新された成功指標

### 技術指標 (実測ベース)
- ✅ コードベース削減: **2,012行削除済み**
- ✅ コンパイル警告: **24 → 21** (さらに0を目標)
- [ ] ファイルサイズ: 500行以下を目標
- [ ] 構造体フィールド: 10個以下を目標  
- [ ] コンパイル時間: 現状比20%短縮

### 保守性指標
- ✅ 重複コード除去完了
- [ ] 新機能追加時間短縮
- [ ] バグ修正効率向上
- [ ] コードレビュー効率向上

## 📋 推奨される次のステップ

1. **即座に実行可能**: フェーズ3 (警告削減) から開始
2. フェーズ3完了後にフェーズ2の段階的実行
3. 各フェーズでの学習内容をこのドキュメントに反映
4. 安定性確保後に大型ファイル分割検討

---
*最終更新: 2025-08-31 (実行結果反映版)*  
*実際のリファクタリング経験を基に大幅更新*  
*段階的アプローチと実証済みの安全な手法を優先*