use eframe::egui;
use crate::app::MyApp;

pub struct SeekPoints;

impl SeekPoints {
    pub fn show(app: &mut MyApp, ui: &mut egui::Ui) {
        if let Some(selected_track) = &app.selection_state.selected_track {
            let track_info = format!("{} - {}", selected_track.artist, selected_track.title);

            // モード切り替え処理のための変数
            let mut mode_changed = false;
            let mut should_start_editing = false;
            let mut should_stop_editing = false;

            // 現在の楽曲情報とモード切り替えボタンを表示
            ui.horizontal(|ui| {
                ui.strong("♪");
                ui.label(&track_info);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 編集/表示モード切り替えボタン
                    let button_text = if app.seek_point_edit_state.is_editing {
                        "表示"
                    } else {
                        "編集"
                    };

                    if ui.button(button_text).clicked() {
                        mode_changed = true;
                        if app.seek_point_edit_state.is_editing {
                            should_stop_editing = true;
                        } else {
                            should_start_editing = true;
                        }
                    }
                });
            });

            // モード変更処理
            if mode_changed {
                if should_stop_editing {
                    app.save_seek_point_edits();
                    app.seek_point_edit_state.stop_editing();
                } else if should_start_editing {
                    if let Some(points) = app.get_selected_track_seek_points() {
                        let points_clone: Vec<_> = points.iter().cloned().collect();
                        app.seek_point_edit_state.start_editing(&points_clone);
                    }
                }
            }
            ui.add_space(10.0);

            // 操作の収集用変数
            let mut seek_point_to_delete: Option<String> = None;

            // 編集中のテキストを一時的に取得（借用問題回避）
            let mut temp_editing_names = if app.seek_point_edit_state.is_editing {
                app.seek_point_edit_state.editing_names.clone()
            } else {
                std::collections::HashMap::new()
            };

            // 選択楽曲のシークポイントを取得
            if let Some(points) = app.get_selected_track_seek_points() {
                if points.is_empty() {
                    ui.label("シークポイントがありません");
                    ui.label("選択楽曲で再生中に「シークポイント追加」ボタンで追加できます");
                } else {
                    ui.label(format!("シークポイント数: {}", points.len()));
                    ui.add_space(5.0);

                    // シークポイント一覧を表示
                    egui::Grid::new("seek_points_grid")
                        .num_columns(3)
                        .striped(true)
                        .show(ui, |ui| {
                            // ヘッダー
                            ui.strong("名前");
                            ui.strong("位置");
                            ui.strong("操作");
                            ui.end_row();

                            // 各シークポイントを表示
                            for seek_point in points {
                                // 名前の表示/編集
                                if app.seek_point_edit_state.is_editing {
                                    // 編集モード：テキストボックス
                                    if let Some(editing_text) = temp_editing_names.get_mut(&seek_point.id) {
                                        ui.text_edit_singleline(editing_text);
                                    }
                                } else {
                                    // 表示モード：読み取り専用ラベル
                                    ui.label(&seek_point.name);
                                }

                                // 位置表示（MM:SS.sss形式）- クリック可能
                                let duration = std::time::Duration::from_millis(seek_point.position_ms);
                                let total_seconds = duration.as_secs_f64();
                                let minutes = (total_seconds / 60.0) as u32;
                                let seconds = total_seconds % 60.0;
                                let time_text = format!("{:02}:{:06.3}", minutes, seconds);

                                // 編集モード・表示モード共通：読み取り専用ラベル
                                ui.label(&time_text);

                                // 削除ボタン
                                ui.horizontal(|ui| {
                                    if ui.small_button("✕").clicked() {
                                        seek_point_to_delete = Some(seek_point.id.clone());
                                    }
                                });

                                ui.end_row();
                            }
                        });
                }
            } else {
                ui.label("シークポイントがありません");
                ui.label("選択楽曲で再生中に「シークポイント追加」ボタンで追加できます");
            }

            // 編集中の場合、変更されたテキストを戻す
            if app.seek_point_edit_state.is_editing {
                app.seek_point_edit_state.editing_names = temp_editing_names;
            }

            // 削除処理の実行（借用チェッカー対応）
            if let Some(seek_point_id) = seek_point_to_delete {
                if let Some(selected_track) = &app.selection_state.selected_track {
                    let track_path = selected_track.path.clone();
                    if let Err(error) = app.remove_seek_point(&track_path, &seek_point_id) {
                        eprintln!("Error removing seek point: {}", error);
                    }
                }
            }
        } else {
            ui.label("楽曲が選択されていません");
        }
    }
}