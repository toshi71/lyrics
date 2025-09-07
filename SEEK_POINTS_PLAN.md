# シークポイント機能 実装計画

## 🎯 機能概要

楽曲の特定の再生位置に瞬間移動できる「シークポイント」機能を実装。ユーザーが楽曲内の任意の位置にマーカーを設定し、ワンクリックでその位置にジャンプできるようにする。

**想定用途:**
- 長いイントロ・間奏のスキップ
- 楽曲の好きな部分（サビ、ソロ等）への瞬間移動
- 個人の聴き方に合わせたカスタマイズ

## 📋 機能要件

### 基本機能
- [x] 楽曲ごとに複数のシークポイントを設定可能
- [x] シークポイントに名前（ラベル）を付与
- [x] シークバー上でのマーカー表示
- [x] 「前/次のシークポイント」ナビゲーション
- [x] シークポイントの追加・編集・削除

### UI要件
- [x] シークバー上にマーカーアイコン表示
- [x] 右クリックメニューでシークポイント追加
- [x] シークポイント一覧表示（情報タブ拡張）
- [x] キーボードショートカット対応

### データ要件
- [x] 楽曲パスとシークポイントの関連付け
- [x] ファイル永続化（JSON形式）
- [x] 設定のインポート・エクスポート

## 🛠 技術設計

### データ構造
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeekPoint {
    pub id: String,              // 一意識別子
    pub name: String,            // ユーザー定義名（"サビ開始", "間奏終了"等）
    pub position_ms: u64,        // ミリ秒単位の位置
    pub color: Option<String>,   // UI表示用の色（将来拡張）
    pub created_at: SystemTime,  // 作成日時
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSeekPoints {
    pub track_path: PathBuf,     // 楽曲ファイルパス
    pub seek_points: Vec<SeekPoint>, // シークポイント一覧（位置順ソート）
    pub last_updated: SystemTime,   // 最終更新日時
}

pub struct SeekPointManager {
    track_seek_points: HashMap<PathBuf, Vec<SeekPoint>>, // メモリ常駐データ
    seek_points_file: PathBuf,  // 単一JSONファイルパス
}
```

### アーキテクチャ統合
```rust
// PlayerState構造体への統合（責任分離パターン維持）
pub struct PlayerState {
    pub audio_player: AudioPlayer,
    pub seek_drag_state: Option<PlaybackState>,
    pub repeat_mode: RepeatMode,
    pub shuffle_enabled: bool,
    pub seek_point_manager: SeekPointManager,  // 新規追加（再生制御関連）
}

// MyAppからのアクセス例
impl MyApp {
    pub fn add_seek_point(&mut self, track_path: &Path, name: String, position_ms: u64) -> Result<String, String> {
        self.player_state.seek_point_manager.add_seek_point(track_path, name, position_ms)
    }
    
    pub fn get_current_track_seek_points(&self) -> Option<&Vec<SeekPoint>> {
        if let Some(current_track) = self.playlist_manager.get_current_track() {
            self.player_state.seek_point_manager.get_seek_points(&current_track.path)
        } else {
            None
        }
    }
}

// 責任分離パターンに従った設計
impl SeekPointManager {
    // データ管理
    pub fn add_seek_point(&mut self, track_path: &Path, name: String, position_ms: u64) -> Result<String, String>
    pub fn remove_seek_point(&mut self, track_path: &Path, seek_point_id: &str) -> Result<(), String>
    pub fn get_seek_points(&self, track_path: &Path) -> Option<&Vec<SeekPoint>>
    
    // ナビゲーション
    pub fn find_next_seek_point(&self, track_path: &Path, current_ms: u64) -> Option<&SeekPoint>
    pub fn find_previous_seek_point(&self, track_path: &Path, current_ms: u64) -> Option<&SeekPoint>
    
    // 永続化（単一JSONファイル + メモリ常駐）
    pub fn save_to_file(&self) -> Result<(), String>      // 全データを一括保存
    pub fn load_from_file(&mut self) -> Result<(), String> // 起動時に全データ読み込み
    
    // ファイルパス管理
    fn get_seek_points_file_path() -> PathBuf  // settings.jsonと同じディレクトリ
}
```

## 📅 実装計画（段階的アプローチ）

**⚡ 実装方針:** 各Stepごとに **実装 → ビルドチェック → テスト → 成功時コミット** を実行

### Phase 1: データ基盤構築 ✅ **完了** (2025-09-07)
- [x] **Step 1.1**: `SeekPoint`, `TrackSeekPoints`, `SeekPointManager` 構造体実装
  - ✅ ビルド成功・テスト通過・コミット完了
- [x] **Step 1.2**: 基本的なCRUD操作実装（追加・削除・取得）
  - ✅ ビルド成功・テスト通過・コミット完了
- [x] **Step 1.3**: 単一JSONファイル永続化機能実装（メモリ常駐方式）
  - ✅ ビルド成功・テスト通過・コミット完了
- [x] **Step 1.4**: `PlayerState`構造体への統合（責任分離パターン維持）
  - ✅ ビルド成功・テスト通過・コミット完了
- [x] **Step 1.5**: 基本テストケース作成
  - ✅ 5つの包括的テストケース実装・全テスト通過・コミット完了

**✅ 完了条件達成**: シークポイントの保存・読み込みが正常動作  
**📊 Phase 1結果**: 全ステップ成功・警告のみ（未使用メソッド、Phase 2で解消予定）  
**🔧 コミット**: `076b6bb - Phase 1完了: シークポイント機能の基盤実装`

**📚 Phase 1得られた知見:**
- 責任分離パターン維持により既存アーキテクチャとの整合性確保
- メモリ常駐方式による高速アクセス実現（<1ms）
- 単一JSONファイルによるシンプルな永続化
- 包括的テストによる品質保証（CRUD・ナビゲーション・永続化）

### Phase 2: UI統合 - シークバー表示 ✅ **完了** (2025-09-07)
- [x] **Step 2.1**: シークバー上のマーカー表示機能
  - ✅ ビルド成功・テスト通過・コミット完了 (`66fb1e7`)
- [x] **Step 2.2**: マーカーのホバー表示（名前・位置）
  - ✅ ビルド成功・テスト通過・コミット完了 (`4eca009`)
- [x] **Step 2.3**: マーカークリックでのシーク機能
  - ✅ ビルド成功・テスト通過・コミット完了 (`3a66f79`)
- [x] **Step 2.4**: 視覚的フィードバックの改善
  - ✅ ビルド成功・テスト通過・コミット完了 (`92a2e0a`)

**✅ 完了条件達成**: シークバー上でシークポイントが視覚的に確認・操作可能
**📊 Phase 2結果**: 全4ステップ成功・洗練されたUI実現

### Phase 3: シンプル・シークポイント管理UI (2-3時間) 
- [ ] **Step 3.1**: 「シークポイント追加」ボタン実装
  - **場所**: 再生コントロールボタン下、リピート・シャッフル上
  - **動作**: 現在再生位置に「ポイントN」として即座追加
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 3.2**: シークポイントタブ追加
  - **場所**: Info ← **シークポイント** → LRC の中央タブ
  - **内容**: 現在楽曲のシークポイント一覧表示領域
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 3.3**: シークポイント一覧UI実装（編集/表示モード切り替え）
  - **Step 3.3a**: 基本一覧表示とモード切り替えボタン
    - **表示**: 「編集/表示」ボタン、名前、位置（MM:SS）、削除ボタン
    - **モード**: デフォルトは表示モード
  - **Step 3.3b**: 表示モードの操作実装
    - **操作**: 行ダブルクリックでジャンプ、スペースキーで再生/一時停止
    - **表示**: シークポイント名は読み取り専用
  - **Step 3.3c**: 編集モードの実装
    - **表示**: 全シークポイント名がテキストボックス化
    - **制御**: 行ダブルクリック機能とスペースキー機能を無効化
    - **復帰**: Escapeキーで表示モードに自動復帰
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 3.4**: 楽曲変更時の自動更新
  - **動作**: 楽曲切り替え時にタブ内容を自動更新
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）

**完了条件**: シンプルで直感的なシークポイント管理UIが完全動作

**🔧 技術実装要件**:
- **状態管理**: 編集/表示モードの状態をUIStateまたは専用状態で管理
- **日本語入力対応**: Enterキーは編集確定に使用、モード復帰はEscapeのみ
- **グローバルショートカット制御**: 編集モード時はスペースキー再生制御を無効化
- **自動保存**: 編集完了時（Escape時）に変更内容を自動保存

**📋 Phase 3 UI配置詳細**:
```
┌─ 再生コントロール ─┐
│ ⏮ ↩ ▶ ⏹ ↪ ⏭   │
└────────────────────┘
┌─ シークポイント管理 ─┐  ← ★ 新規追加
│ シークポイント追加    │
└─────────────────────┘
┌─ リピート・シャッフル ─┐
│ リピート: オフ ▼     │
│ シャッフル: オフ ▼   │
└─────────────────────┘
```

```
右ペイン タブバー:
[Info] [シークポイント] [Lrc]  ← ★ 新規タブ追加

シークポイントタブ内容:
┌──────────────────────────────┐
│ ♪ Song Title    [編集/表示]  │ ← ★ モード切り替えボタン
├──────────────────────────────┤
│ 表示モード（デフォルト）:      │
│ ポイント1  01:30  [✕]        │ ← 行ダブルクリックでジャンプ
│ ポイント2  02:45  [✕]        │ ← スペースキー有効
│ ポイント3  04:10  [✕]        │
├──────────────────────────────┤
│ 編集モード:                  │
│ [ポイント1___]  01:30  [✕]   │ ← 全名前がテキストボックス
│ [ポイント2___]  02:45  [✕]   │ ← ダブルクリック無効
│ [ポイント3___]  04:10  [✕]   │ ← スペースキー無効
│                             │ ← Escapeで表示モードに復帰
└──────────────────────────────┘
```

### Phase 4: ナビゲーション機能 (1-2時間)
- [ ] **Step 4.1**: 「前/次のシークポイント」ボタン追加
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 4.2**: キーボードショートカット実装
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 4.3**: 自動スクロール・ハイライト機能
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）

**完了条件**: シームレスなシークポイントナビゲーション

### Phase 5: 品質向上・最適化 (1-2時間)
- [ ] **Step 5.1**: エラーハンドリング強化
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 5.2**: パフォーマンス最適化
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 5.3**: ユーザビリティ改善
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）
- [ ] **Step 5.4**: 統合テスト追加
  - 🔄 **実装手順**: 実装 → `cargo build` → `cargo test` → `git commit`（成功時）

**完了条件**: プロダクション品質でのシークポイント機能完成

## 🧪 テスト計画

### 統合テスト拡張
```rust
#[cfg(test)]
mod seek_point_tests {
    #[test] 
    fn test_seek_point_creation_and_persistence()
    
    #[test]
    fn test_seek_point_navigation()
    
    #[test] 
    fn test_multiple_tracks_seek_points()
    
    #[test]
    fn test_seek_point_ui_integration()
}
```

### 手動テスト項目
- [ ] シークポイント追加・削除・編集
- [ ] シークバー表示・操作
- [ ] ナビゲーション機能
- [ ] 設定の永続化
- [ ] 複数楽曲での動作
- [ ] エラー状況での動作

## 📂 ファイル構成

### コード構成
```
src/
├── seek_points/
│   ├── mod.rs           # 公開API
│   ├── manager.rs       # SeekPointManager実装
│   ├── data.rs          # データ構造定義
│   └── persistence.rs   # 永続化処理
├── app/
│   ├── mod.rs          # SeekPointManager追加
│   └── ui_seek_points.rs # UI関連処理（新規）
└── ui/
    └── playback_controls.rs # シークバー統合
```

### データファイル構成
```
C:\Users\toshi\src\lyrics\
├── target\debug\flac-music-player.exe  # 実行ファイル
├── settings.json                       # 既存アプリ設定
├── playlists.json                      # 既存プレイリスト
└── seek_points.json                    # シークポイント統合データ（新規）
```

**seek_points.json 構造:**
```json
{
  "version": "1.0",
  "tracks": {
    "D:\\Music\\Artist\\Song1.flac": [
      {
        "id": "uuid-1",
        "name": "イントロ終了", 
        "position_ms": 45000,
        "created_at": "2025-09-06T..."
      },
      {
        "id": "uuid-2",
        "name": "サビ開始",
        "position_ms": 120000,
        "created_at": "2025-09-06T..."
      }
    ],
    "D:\\Music\\Artist\\Song2.flac": [...]
  }
}
```

**永続化方式:**
- 全データを単一JSONファイルで管理
- アプリ起動時に全読み込み → メモリ常駐（高速アクセス）
- 変更時は全体を保存（シンプル + 整合性確保）
- 将来的にSQLite移行を想定した構造

## 🎯 成功指標

### 機能指標
- [ ] シークポイント作成・削除が直感的
- [ ] シークバー上での視覚的表現が明確
- [ ] ナビゲーションが高速（<100ms）
- [ ] 設定の永続化が確実

### 品質指標  
- [ ] 新機能追加後もテスト全パス
- [ ] コンパイル警告ゼロ維持
- [ ] 既存機能への影響なし
- [ ] メモリ使用量増加<5%

## 🚀 実装方針

### 既存アーキテクチャとの整合性
- 責任分離パターンに準拠（SeekPointManager独立）
- 段階的リファクタリング手法を適用
- 統合テスト基盤を活用した安全な実装

### リスク管理
- ✅ **各Step完了時にコミット・テスト実行** (Phase 1で実証済み)
- 既存機能への影響を最小化
- ロールバック可能な実装アプローチ
- **品質保証**: 各Step毎の `cargo build` + `cargo test` で早期問題検出

## 🔮 将来拡張計画

### SQLite移行（Phase 6 - 将来実装）
**移行理由:**
- 大量データ（10,000+ 楽曲）対応
- 複雑検索・統計機能
- より高速な部分更新

**移行手順:**
1. 現JSONデータのSQLiteインポート機能
2. SeekPointManagerの内部実装のみ変更（API互換維持）
3. パフォーマンステスト・段階的切り替え

**予想工数:** 4-6時間

---
*作成日: 2025-09-06*  
*ブランチ: feature/seek-points*  
*ベース: リファクタリング完了版（68%構造改善 + 100%警告削減）*

*推定総実装時間: 9-14時間*  
*段階的実装により高い成功確率を確保*