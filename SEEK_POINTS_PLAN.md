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

### Phase 1: データ基盤構築 (2-3時間)
- [ ] **Step 1.1**: `SeekPoint`, `TrackSeekPoints`, `SeekPointManager` 構造体実装
- [ ] **Step 1.2**: 基本的なCRUD操作実装（追加・削除・取得）
- [ ] **Step 1.3**: 単一JSONファイル永続化機能実装（メモリ常駐方式）
- [ ] **Step 1.4**: `PlayerState`構造体への統合（責任分離パターン維持）
- [ ] **Step 1.5**: 基本テストケース作成

**完了条件**: シークポイントの保存・読み込みが正常動作

### Phase 2: UI統合 - シークバー表示 (2-3時間)  
- [ ] **Step 2.1**: シークバー上のマーカー表示機能
- [ ] **Step 2.2**: マーカーのホバー表示（名前・位置）
- [ ] **Step 2.3**: マーカークリックでのシーク機能
- [ ] **Step 2.4**: 視覚的フィードバックの改善

**完了条件**: シークバー上でシークポイントが視覚的に確認・操作可能

### Phase 3: シークポイント管理UI (3-4時間)
- [ ] **Step 3.1**: 右クリックメニューで「シークポイント追加」
- [ ] **Step 3.2**: シークポイント追加ダイアログ実装
- [ ] **Step 3.3**: 情報タブにシークポイント一覧表示
- [ ] **Step 3.4**: シークポイント編集・削除機能

**完了条件**: 完全なシークポイント管理UIが動作

### Phase 4: ナビゲーション機能 (1-2時間)
- [ ] **Step 4.1**: 「前/次のシークポイント」ボタン追加
- [ ] **Step 4.2**: キーボードショートカット実装
- [ ] **Step 4.3**: 自動スクロール・ハイライト機能

**完了条件**: シームレスなシークポイントナビゲーション

### Phase 5: 品質向上・最適化 (1-2時間)
- [ ] **Step 5.1**: エラーハンドリング強化
- [ ] **Step 5.2**: パフォーマンス最適化
- [ ] **Step 5.3**: ユーザビリティ改善
- [ ] **Step 5.4**: 統合テスト追加

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
- 各Phase完了時にコミット・テスト実行
- 既存機能への影響を最小化
- ロールバック可能な実装アプローチ

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