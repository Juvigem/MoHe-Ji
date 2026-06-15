use super::{ParamWheelTarget, PointSelection, SegmentSelection, Tool, ToolbarTab, VectorEditorApp};
use crate::render::canvas::CanvasView;
use crate::ppw::Vec2;

pub fn show(app: &mut VectorEditorApp, ctx: &egui::Context) {
    handle_shortcuts(app, ctx);
    ctx.set_visuals(if app.dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() });

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.heading("MoHe-Ji");
            if ui.button(if app.side_panel_open { "Hide Panel" } else { "Show Panel" }).clicked() { app.side_panel_open = !app.side_panel_open; }
            ui.separator();

            ui.selectable_value(&mut app.active_toolbar_tab, ToolbarTab::Tools, "1 Tools");
            ui.selectable_value(&mut app.active_toolbar_tab, ToolbarTab::File, "2 File");
            ui.selectable_value(&mut app.active_toolbar_tab, ToolbarTab::Edit, "3 Edit");
            ui.selectable_value(&mut app.active_toolbar_tab, ToolbarTab::View, "4 View");
            ui.selectable_value(&mut app.active_toolbar_tab, ToolbarTab::Path, "5 Path");
            ui.selectable_value(&mut app.active_toolbar_tab, ToolbarTab::Layer, "6 Layer");
        });

        ui.separator();

        ui.horizontal_wrapped(|ui| {
            match app.active_toolbar_tab {
                ToolbarTab::Tools => toolbar_tools(app, ui),
                ToolbarTab::File => toolbar_file(app, ui),
                ToolbarTab::Edit => toolbar_edit(app, ui),
                ToolbarTab::View => toolbar_view(app, ui),
                ToolbarTab::Path => toolbar_path(app, ui),
                ToolbarTab::Layer => toolbar_layer(app, ui),
            }
        });
    });

    if app.side_panel_open {
    egui::SidePanel::left("side_panel")
        .default_width(360.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("left_side_panel_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::CollapsingHeader::new("SVG")
                        .default_open(false)
                        .show(ui, |ui| svg_panel(app, ui));

                    egui::CollapsingHeader::new("Layers")
                        .default_open(true)
                        .show(ui, |ui| layer_panel(app, ui));

                    egui::CollapsingHeader::new("Tool Help")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label("Select: click / rectangle select / move selected points");
                            ui.label("Vector Brush: drag on a vector layer to create a PPW path.");
                            ui.label("Add Point: append points to the selected existing path. Shift+V resumes adding to selected path.");
                            ui.label("Rectangle/Ellipse: drag to size, hold Shift for square/circle");
                            ui.label("Middle mouse drag: pan canvas");
                            ui.label("Ctrl+Z: Undo / Ctrl+Y: Redo");
                            ui.label("Select + point: Q/W/E/S/D choose PPW wheel target. Z/X/C choose Red/Green/Blue color wheel targets.");
                            ui.label("Raster Brush/Eraser: use on raster layers. Press R + wheel to change width.");
                            ui.label("Image: Add Image... loads PNG/JPEG. Select and drag to move; drag bottom-right handle to resize. Hold Shift while resizing to keep the original aspect ratio.");
                            ui.label("T: simplify selected path. Repeated T increases tolerance by 5; selecting another path resets it.");
                            ui.label("Y: transform selected path/image. Corner = proportional resize, arrows = X/Y resize, blue circle = rotate.");
                        });

                    egui::CollapsingHeader::new("Path / Stroke / PPW")
                        .default_open(true)
                        .show(ui, |ui| path_panel(app, ui));
                });
        });
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        let response = CanvasView {
            document: &mut app.document,
            show_points: app.show_points,
            show_segments: app.show_segments,
            show_fill: app.show_fill,
            show_triangles: app.show_triangles,
            active_tool: app.active_tool,
            selected_path: &mut app.selected_path,
            selected_point: &mut app.selected_point,
            selected_points: &mut app.selected_points,
            selected_nodes: &mut app.selected_nodes,
            selected_segment: &mut app.selected_segment,
            dragging_point: &mut app.dragging_point,
            drag_last_pos: &mut app.drag_last_pos,
            selection_rect_start: &mut app.selection_rect_start,
            selection_rect_current: &mut app.selection_rect_current,
            undo_stack: &mut app.undo_stack,
            redo_stack: &mut app.redo_stack,
            pending_drag_snapshot: &mut app.pending_drag_snapshot,
            brush_points: &mut app.brush_points,
            brush_width: app.brush_width,
            raster_brush_width: app.raster_brush_width,
            raster_eraser_width: app.raster_eraser_width,
            raster_color: app.raster_color,
            zoom: app.zoom,
            pan_offset: &mut app.pan_offset,
            shape_start: &mut app.shape_start,
            shape_current: &mut app.shape_current,
            selected_image: &mut app.selected_image,
            resizing_image: &mut app.resizing_image,
            resizing_path: &mut app.resizing_path,
            transform_mode: &mut app.transform_mode,
            rotating_selection: &mut app.rotating_selection,
            active_transform_handle: &mut app.active_transform_handle,
            transform_anchor: &mut app.transform_anchor,
            image_textures: &mut app.image_textures,
        }
        .show(ui);

        response.context_menu(|ui| {
            ui.label("Toolbar Tab");
            ui.horizontal_wrapped(|ui| {
                context_tab_hover_button(ui, app, ToolbarTab::Tools, "1 Tools");
                context_tab_hover_button(ui, app, ToolbarTab::File, "2 File");
                context_tab_hover_button(ui, app, ToolbarTab::Edit, "3 Edit");
                context_tab_hover_button(ui, app, ToolbarTab::View, "4 View");
                context_tab_hover_button(ui, app, ToolbarTab::Path, "5 Path");
                context_tab_hover_button(ui, app, ToolbarTab::Layer, "6 Layer");
            });
            ui.separator();
            ui.label(format!("Tab: {:?}", app.active_toolbar_tab));
            match app.active_toolbar_tab {
                ToolbarTab::Tools => toolbar_tools(app, ui),
                ToolbarTab::File => toolbar_file(app, ui),
                ToolbarTab::Edit => toolbar_edit(app, ui),
                ToolbarTab::View => toolbar_view(app, ui),
                ToolbarTab::Path => toolbar_path(app, ui),
                ToolbarTab::Layer => toolbar_layer(app, ui),
            }
        });
    });

    show_param_wheel_cursor_label(app, ctx);
}


fn context_tab_hover_button(
    ui: &mut egui::Ui,
    app: &mut VectorEditorApp,
    tab: ToolbarTab,
    label: &str,
) {
    let response = ui.selectable_label(app.active_toolbar_tab == tab, label);
    if response.hovered() {
        app.active_toolbar_tab = tab;
    }
    if response.clicked() {
        app.active_toolbar_tab = tab;
    }
}

fn show_param_wheel_cursor_label(app: &VectorEditorApp, ctx: &egui::Context) {
    let Some(target) = app.param_wheel_target else {
        return;
    };

    if app.active_tool != Tool::Select && !is_global_wheel_target(target) {
        return;
    }

    if requires_selected_nodes(target) && app.selected_nodes.is_empty() {
        return;
    }

    let Some(pointer_pos) = ctx.pointer_hover_pos() else {
        return;
    };

    let text = format!("Wheel: {}", target.label());
    egui::Area::new(egui::Id::new("ppw_param_wheel_cursor_label"))
        .order(egui::Order::Foreground)
        .fixed_pos(pointer_pos + egui::vec2(18.0, 8.0))
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .show(ui, |ui| {
                    ui.label(text);
                    if target == ParamWheelTarget::StrokeWidth {
                        ui.small("Mouse wheel edits stroke / raster brush width");
                    } else if is_fill_color_target(target) {
                        ui.small("Mouse wheel edits selected path fill color");
                    } else if is_paint_color_target(target) {
                        ui.small("Mouse wheel edits stroke color or raster brush color");
                    } else {
                        ui.small("Mouse wheel edits selected point(s)");
                    }
                });
        });
}

fn toolbar_tools(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    ui.label("Tool:");
    ui.selectable_value(&mut app.active_tool, Tool::Select, "Select (F)");
    ui.selectable_value(&mut app.active_tool, Tool::VectorBrush, "Vector Brush (B)");
    ui.selectable_value(&mut app.active_tool, Tool::AddPoint, "Add Point (V / Shift+V)");
    ui.selectable_value(&mut app.active_tool, Tool::Rectangle, "Rectangle");
    ui.selectable_value(&mut app.active_tool, Tool::Ellipse, "Ellipse");
    ui.selectable_value(&mut app.active_tool, Tool::RasterBrush, "Raster Brush (V on raster)");
    ui.selectable_value(&mut app.active_tool, Tool::RasterEraser, "Raster Eraser (Space toggles)");
    if ui.button("Add Image...").on_hover_text("PNG / JPEG").clicked() {
        app.add_image_from_file();
    }

    ui.separator();
    ui.label(format!("Active: {:?}", app.active_tool));
}


fn png_quality_ui(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("PNG Quality / Scale:");
        egui::ComboBox::from_id_salt("png_quality_scale")
            .selected_text(format!("{:.0}%", app.png_quality_scale * 100.0))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut app.png_quality_scale, 0.5, "50% (Small)");
                ui.selectable_value(&mut app.png_quality_scale, 1.0, "100% (Normal)");
                ui.selectable_value(&mut app.png_quality_scale, 2.0, "200% (High)");
                ui.selectable_value(&mut app.png_quality_scale, 3.0, "300% (Very High)");
                ui.selectable_value(&mut app.png_quality_scale, 4.0, "400% (Maximum)");
            });
    });
    app.png_quality_scale = app.png_quality_scale.clamp(0.25, 4.0);
    ui.small("PNG is lossless. This setting changes the exported pixel resolution.");
}

fn toolbar_file(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    ui.label("SVG:");
    ui.text_edit_singleline(&mut app.svg_file_path);

    if ui.button("Save SVG").on_hover_text("Ctrl+S").clicked() {
        app.save_svg();
    }
    if ui.button("Load SVG...").on_hover_text("Ctrl+O").clicked() {
        app.choose_svg_file_and_load();
    }

    ui.separator();
    if ui.button("Add Image...").on_hover_text("Load PNG/JPEG as a raster-cached image object").clicked() {
        app.add_image_from_file();
    }

    ui.separator();
    ui.label("PNG Export:");
    ui.horizontal(|ui| {
        ui.label("Folder / full path:");
        ui.text_edit_singleline(&mut app.png_file_path);
    });
    ui.horizontal(|ui| {
        ui.label("File name:");
        ui.text_edit_singleline(&mut app.png_file_name);
    });
    ui.checkbox(&mut app.png_transparent_empty, "Transparent empty pixels");
    png_quality_ui(app, ui);
    ui.horizontal(|ui| {
        if ui.button("Choose Folder").clicked() {
            app.choose_png_folder();
        }
        if ui.button("Export Canvas PNG...").clicked() {
            app.choose_png_path_and_save();
        }
    });

    ui.separator();
    ui.label(&app.io_status);
}

fn toolbar_edit(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    if ui
        .add_enabled(!app.undo_stack.is_empty(), egui::Button::new("Undo"))
        .on_hover_text("Ctrl+Z")
        .clicked()
    {
        app.undo();
    }
    if ui
        .add_enabled(!app.redo_stack.is_empty(), egui::Button::new("Redo"))
        .on_hover_text("Ctrl+Y / Ctrl+Shift+Z")
        .clicked()
    {
        app.redo();
    }

    ui.separator();
    if ui.button("Copy Selected Points").clicked() {
        app.copy_selected_points();
    }
    if ui.button("Paste Points").clicked() {
        app.paste_copied_points();
    }
    ui.small(format!("Copied point paths: {}", app.copied_point_paths.len()));

    ui.separator();

    if ui.button("Delete Last Point").clicked() {
        app.push_undo_snapshot();
        if let Some(path) = app.document.active_path_mut(app.selected_path) {
            path.remove_last_point();
            app.clear_selection_state();
        }
    }

    if ui.button("Delete Selected Points").on_hover_text("Space while Select is active").clicked() {
        delete_selected_points_action(app);
    }

    if ui.button("Split Selected Path Line").on_hover_text("G while Select is active").clicked() {
        split_selected_path_line_action(app);
    }
    if ui.button("Finish Current Path").on_hover_text("End this path and start a new empty path on the same layer").clicked() {
        app.finish_current_path();
    }
    if ui.button("Simplify Selected Path").on_hover_text("Reduce control points on the selected path").clicked() {
        app.simplify_selected_path();
    }
}

fn toolbar_view(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    ui.checkbox(&mut app.dark_mode, "Dark Mode");
    ui.small("The canvas itself stays white.");
    ui.separator();
    ui.checkbox(&mut app.show_points, "Points (P)");
    ui.checkbox(&mut app.show_fill, "Fill");
    ui.checkbox(&mut app.show_triangles, "Triangles");
    ui.separator();
    if ui.button("Zoom In (+)").clicked() {
        app.zoom = (app.zoom * 1.1).min(8.0);
    }
    if ui.button("Zoom Out (-)").clicked() {
        app.zoom = (app.zoom / 1.1).max(0.1);
    }
    if ui.button("Reset Zoom").clicked() {
        app.zoom = 1.0;
    }
    if ui.button("Reset Pan").clicked() {
        app.pan_offset = crate::ppw::Vec2::ZERO;
    }
    ui.label(format!("Zoom: {:.0}%", app.zoom * 100.0));
}

fn toolbar_path(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    if ui.button("Reset Sample").clicked() {
        app.push_undo_snapshot();
        app.document = crate::model::document::Document::sample_ppw();
        app.selected_path = 0;
        app.clear_selection_state();
    }

    if ui.button("New Empty Path").clicked() {
        app.push_undo_snapshot();
        app.document = crate::model::document::Document::empty_path();
        app.selected_path = 0;
        app.clear_selection_state();
    }

    ui.separator();
    ui.add(egui::Slider::new(&mut app.brush_width, 0.5..=32.0).text("New Shape Stroke Width"));
}

fn toolbar_layer(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    if ui.button("Add Vector Layer").clicked() {
        app.push_undo_snapshot();
        app.document.add_vector_layer();
        app.selected_path = 0;
        app.clear_selection_state();
    }

    if ui.button("Add Raster Layer").clicked() {
        app.push_undo_snapshot();
        app.document.add_raster_layer();
        app.selected_path = 0;
        app.clear_selection_state();
        app.active_tool = Tool::RasterBrush;
    }

    if ui.button("Duplicate").clicked() {
        app.push_undo_snapshot();
        app.document.duplicate_active_layer();
        app.selected_path = 0;
        app.clear_selection_state();
    }

    if ui.button("Delete Layer").clicked() {
        app.push_undo_snapshot();
        app.document.delete_active_layer();
        app.selected_path = 0;
        app.clear_selection_state();
    }

    ui.small("Reorder layers by dragging rows in the Layers panel.");

    ui.separator();
    ui.label(format!("Active Layer: {}", app.document.active_layer + 1));
}


fn handle_shortcuts(app: &mut VectorEditorApp, ctx: &egui::Context) {
    let undo = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift);
    let redo = ctx.input(|i| {
        (i.modifiers.command && i.key_pressed(egui::Key::Y))
            || (i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
    });
    let save = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S));
    let open = ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O));
let wants_text = ctx.wants_keyboard_input();
    let typed = ctx.input(|i| {
        i.events
            .iter()
            .filter_map(|event| match event {
                egui::Event::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    });

    if !wants_text {
        for text in typed {
            match text.as_str() {
                "+" | "=" => app.zoom = (app.zoom * 1.1).min(8.0),
                "-" | "_" => app.zoom = (app.zoom / 1.1).max(0.1),
                "1" => app.active_toolbar_tab = ToolbarTab::Tools,
                "2" => app.active_toolbar_tab = ToolbarTab::File,
                "3" => app.active_toolbar_tab = ToolbarTab::Edit,
                "4" => app.active_toolbar_tab = ToolbarTab::View,
                "5" => app.active_toolbar_tab = ToolbarTab::Path,
                "6" => app.active_toolbar_tab = ToolbarTab::Layer,
                _ => {}
            }
        }

        let p_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::P));
        if p_pressed {
            app.show_points = !app.show_points;
        }

        let f_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::F));
        if f_pressed {
            app.active_tool = Tool::Select;
            app.active_toolbar_tab = ToolbarTab::Tools;
        }

        let v_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::V));
        if v_pressed {
            let shift = ctx.input(|i| i.modifiers.shift);
            match app.document.active_layer().map(|l| l.kind) {
                Some(crate::model::document::LayerKind::Vector) => {
                    app.active_tool = Tool::AddPoint;
                    if shift {
                        app.io_status = "Add Point: appending to selected path".to_string();
                    } else {
                        app.io_status = "Add Point: appending to selected path".to_string();
                    }
                }
                Some(crate::model::document::LayerKind::Raster) => app.active_tool = Tool::RasterBrush,
                None => {}
            }
            app.active_toolbar_tab = ToolbarTab::Tools;
        }


        let b_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::B));
        if b_pressed {
            if app.document.active_layer().map(|l| l.kind == crate::model::document::LayerKind::Vector).unwrap_or(false) {
                app.active_tool = Tool::VectorBrush;
                app.active_toolbar_tab = ToolbarTab::Tools;
                app.io_status = "Vector Brush: drag to create a vector PPW path".to_string();
            }
        }

        let a_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::A));
        if a_pressed {
            app.finish_current_path();
        }

        let t_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::T));
        if t_pressed && app.active_tool == Tool::Select {
            app.simplify_selected_path_with_shortcut();
        }

        let y_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::Y));
        if y_pressed && app.active_tool == Tool::Select {
            app.enable_transform_mode();
        }

        let space_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::Space));
        if space_pressed {
            if app.selected_image.is_some() {
                delete_selected_image_action(app);
            } else {
                match app.active_tool {
                    Tool::AddPoint => delete_last_point_action(app),
                    Tool::Select => delete_selected_points_action(app),
                    Tool::RasterBrush => app.active_tool = Tool::RasterEraser,
                    Tool::RasterEraser => app.active_tool = Tool::RasterBrush,
                    _ => {}
                }
            }
        }


        let arrow_delta = ctx.input(|i| {
            let step = if i.modifiers.shift { 10.0 } else { 1.0 };
            let mut dx = 0.0_f32;
            let mut dy = 0.0_f32;
            if i.key_pressed(egui::Key::ArrowLeft) {
                dx -= step;
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                dx += step;
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                dy -= step;
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                dy += step;
            }
            if dx != 0.0 || dy != 0.0 {
                Some(Vec2::new(dx, dy))
            } else {
                None
            }
        });

        if let Some(delta) = arrow_delta {
            if app.active_tool == Tool::Select && (!app.selected_nodes.is_empty() || !app.selected_points.is_empty()) {
                move_selected_points_by_arrow(app, delta);
            } else {
                // Pan the view in screen pixels. This mirrors middle-mouse dragging.
                app.pan_offset += delta * 20.0;
            }
            ctx.request_repaint();
        }

        let g_pressed = ctx.input(|i| !i.modifiers.command && !i.modifiers.ctrl && !i.modifiers.alt && i.key_pressed(egui::Key::G));
        if g_pressed && app.active_tool == Tool::Select {
            split_selected_path_line_action(app);
        }

        let target = ctx.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl || i.modifiers.alt {
                None
            } else {
                let shift = i.modifiers.shift;
                if i.key_pressed(egui::Key::Q) {
                    Some(ParamWheelTarget::Weight)
                } else if i.key_pressed(egui::Key::W) {
                    Some(ParamWheelTarget::PsiPrev)
                } else if i.key_pressed(egui::Key::E) {
                    Some(ParamWheelTarget::PsiNext)
                } else if i.key_pressed(egui::Key::S) {
                    Some(ParamWheelTarget::PhiPrev)
                } else if i.key_pressed(egui::Key::D) {
                    Some(ParamWheelTarget::PhiNext)
                } else if i.key_pressed(egui::Key::R) {
                    Some(ParamWheelTarget::StrokeWidth)
                } else if i.key_pressed(egui::Key::Z) {
                    Some(ParamWheelTarget::PaintHue)
                } else if i.key_pressed(egui::Key::X) {
                    Some(ParamWheelTarget::PaintBrightness)
                } else if i.key_pressed(egui::Key::C) {
                    Some(ParamWheelTarget::PaintAlpha)
                } else {
                    None
                }
            }
        });

        if let Some(target) = target {
            app.param_wheel_target = Some(target);
            app.active_toolbar_tab = ToolbarTab::Path;
        }
    }

    let scroll_y = ctx.input(|i| i.raw_scroll_delta.y);
    if !wants_text
        && scroll_y.abs() > 0.0
    {
        if let Some(target) = app.param_wheel_target {
            if requires_selected_nodes(target) && app.selected_nodes.is_empty() {
                return;
            }
            let direction = if scroll_y > 0.0 { 1.0 } else { -1.0 };
            if adjust_selected_ppw_parameter_by_wheel(app, target, direction) {
                ctx.request_repaint();
            }
        }
    }

    if undo {
        app.undo();
    }
    if redo {
        app.redo();
    }
    if save {
        app.save_svg();
    }
    if open {
        app.choose_svg_file_and_load();
    }
}



fn move_selected_points_by_arrow(app: &mut VectorEditorApp, screen_delta: Vec2) {
    let world_delta = screen_delta / app.zoom.max(0.1);

    let mut moved = false;
    if !app.selected_nodes.is_empty() {
        app.push_undo_snapshot();
        let mut nodes = app.selected_nodes.clone();
        nodes.sort_unstable();
        nodes.dedup();

        if let Some(layer) = app.document.active_layer_mut() {
            for node in nodes {
                if let Some(path) = layer.paths.get_mut(node.path_index) {
                    if let Some(point) = path.control_points.get_mut(node.point_index) {
                        *point += world_delta;
                        moved = true;
                    }
                }
            }
        }
    } else if !app.selected_points.is_empty() {
        app.push_undo_snapshot();
        let mut points = app.selected_points.clone();
        points.sort_unstable();
        points.dedup();

        if let Some(path) = app.document.active_path_mut(app.selected_path) {
            for point_index in points {
                if let Some(point) = path.control_points.get_mut(point_index) {
                    *point += world_delta;
                    moved = true;
                }
            }
        }
    }

    if moved {
        app.io_status = format!("Moved selected point(s) by {:.1}, {:.1}", world_delta.x, world_delta.y);
    }
}


fn delete_last_point_action(app: &mut VectorEditorApp) {
    app.push_undo_snapshot();
    if let Some(path) = app.document.active_path_mut(app.selected_path) {
        path.remove_last_point();
    }
    app.clear_selection_state();
    app.io_status = "Deleted last point".to_string();
}

fn delete_selected_points_action(app: &mut VectorEditorApp) {
    if app.selected_nodes.is_empty() && app.selected_points.is_empty() {
        app.io_status = "Delete selected points: no points selected".to_string();
        return;
    }
    app.push_undo_snapshot();
    app.document
        .remove_selected_points(app.selected_path, &app.selected_points, &app.selected_nodes);
    app.clear_selection_state();
    app.io_status = "Deleted selected points".to_string();
}

fn delete_selected_image_action(app: &mut VectorEditorApp) {
    let Some(image_index) = app.selected_image else {
        return;
    };

    let Some(layer) = app.document.active_layer() else {
        app.selected_image = None;
        app.resizing_image = false;
        return;
    };

    if image_index >= layer.images.len() {
        app.selected_image = None;
        app.resizing_image = false;
        return;
    }

    app.push_undo_snapshot();

    if let Some(layer) = app.document.active_layer_mut() {
        if image_index < layer.images.len() {
            let removed = layer.images.remove(image_index);
            app.image_textures.remove(&removed.id);
        }
    }

    app.selected_image = None;
    app.resizing_image = false;
    app.io_status = "Deleted selected image.".to_string();
}

fn split_selected_path_line_action(app: &mut VectorEditorApp) {
    if app.document
        .active_layer()
        .map(|layer| layer.kind != crate::model::document::LayerKind::Vector || layer.locked)
        .unwrap_or(true)
    {
        app.io_status = "Split failed: choose an unlocked vector layer".to_string();
        return;
    }

    let Some(path_index) = app.selected_segment.map(|seg| seg.path_index).or(Some(app.selected_path)) else {
        return;
    };
    let segment_index = app.selected_segment.map(|seg| seg.segment_index);

    app.push_undo_snapshot();

    let mut new_point_index = None;
    if let Some(path) = app.document.active_path_mut(path_index) {
        new_point_index = split_path_at_segment(path, segment_index);
    }

    if let Some(point_index) = new_point_index {
        app.selected_path = path_index;
        app.selected_point = Some(point_index);
        app.selected_points = vec![point_index];
        app.selected_nodes = vec![PointSelection { path_index, point_index }];
        app.selected_segment = None;
        app.io_status = format!("Split path {} at point {}", path_index, point_index);
    } else {
        app.undo();
        app.io_status = "Split failed: selected path has no splittable segment".to_string();
    }
}

fn split_path_at_segment(path: &mut crate::ppw::path::PPWPath, segment_index: Option<usize>) -> Option<usize> {
    if path.control_points.len() < 2 {
        return None;
    }

    let seg_count = if path.is_closed {
        path.control_points.len()
    } else {
        path.control_points.len().saturating_sub(1)
    };
    if seg_count == 0 {
        return None;
    }

    let seg_index = segment_index
        .filter(|index| *index < seg_count)
        .unwrap_or_else(|| longest_control_segment_index(path));

    let a_index = seg_index;
    let b_index = if a_index + 1 < path.control_points.len() {
        a_index + 1
    } else if path.is_closed {
        0
    } else {
        return None;
    };

    let a = path.control_points[a_index];
    let b = path.control_points[b_index];
    let mid = (a + b) * 0.5;

    let insert_index = a_index + 1;
    if insert_index <= path.control_points.len() {
        path.control_points.insert(insert_index, mid);
        path.weights.insert(insert_index, 1.0);
        path.rebuild_open_segment_params();
        Some(insert_index)
    } else {
        None
    }
}

fn longest_control_segment_index(path: &crate::ppw::path::PPWPath) -> usize {
    let n = path.control_points.len();
    if n < 2 {
        return 0;
    }
    let seg_count = if path.is_closed { n } else { n - 1 };
    let mut best = 0usize;
    let mut best_len = -1.0f32;
    for i in 0..seg_count {
        let j = if i + 1 < n { i + 1 } else { 0 };
        let len = (path.control_points[j] - path.control_points[i]).length_squared();
        if len > best_len {
            best = i;
            best_len = len;
        }
    }
    best
}


fn adjust_selected_ppw_parameter_by_wheel(
    app: &mut VectorEditorApp,
    target: ParamWheelTarget,
    direction: f32,
) -> bool {
    if app.active_tool != Tool::Select && !is_global_wheel_target(target) {
        return false;
    }
    if requires_selected_nodes(target) && app.selected_nodes.is_empty() {
        return false;
    }

    let active_layer_locked = app
        .document
        .active_layer()
        .map(|layer| layer.locked)
        .unwrap_or(true);
    if active_layer_locked {
        return false;
    }

    if is_fill_color_target(target) || is_paint_color_target(target) {
        return adjust_color_by_wheel(app, target, direction);
    }

    if target == ParamWheelTarget::StrokeWidth
        && app
            .document
            .active_layer()
            .map(|layer| layer.kind == crate::model::document::LayerKind::Raster)
            .unwrap_or(false)
    {
        match app.active_tool {
            Tool::RasterEraser => {
                let next = log_wheel_size(app.raster_eraser_width, direction, 0.5, 256.0);
                if (next - app.raster_eraser_width).abs() > f32::EPSILON {
                    app.raster_eraser_width = next;
                    return true;
                }
            }
            _ => {
                let next = log_wheel_size(app.raster_brush_width, direction, 0.5, 128.0);
                if (next - app.raster_brush_width).abs() > f32::EPSILON {
                    app.raster_brush_width = next;
                    return true;
                }
            }
        }
        return false;
    }

    let snapshot = app.document.clone();
    let nodes = app.selected_nodes.clone();
    let mut changed = false;

    let Some(layer) = app.document.active_layer_mut() else {
        return false;
    };

    match target {
        ParamWheelTarget::StrokeWidth => {
            let mut edited_paths: Vec<usize> = Vec::new();
            if nodes.is_empty() {
                edited_paths.push(app.selected_path);
            } else {
                for node in nodes {
                    if !edited_paths.contains(&node.path_index) {
                        edited_paths.push(node.path_index);
                    }
                }
            }
            for path_index in edited_paths {
                if let Some(path) = layer.paths.get_mut(path_index) {
                    let next = log_wheel_size(path.stroke_width, direction, 0.1, 64.0);
                    if (next - path.stroke_width).abs() > f32::EPSILON {
                        path.stroke_width = next;
                        changed = true;
                    }
                }
            }
        }
        ParamWheelTarget::Weight => {
            let step = 0.05 * direction;
            for node in nodes {
                if let Some(path) = layer.paths.get_mut(node.path_index) {
                    if let Some(weight) = path.weights.get_mut(node.point_index) {
                        let next = (*weight + step).clamp(0.1, 4.0);
                        if (next - *weight).abs() > f32::EPSILON {
                            *weight = next;
                            changed = true;
                        }
                    }
                }
            }
        }
        ParamWheelTarget::PsiPrev | ParamWheelTarget::PsiNext => {
            let step = 0.05 * direction;
            let mut edited_segments: Vec<(usize, usize)> = Vec::new();
            for node in nodes {
                let Some(path) = layer.paths.get_mut(node.path_index) else {
                    continue;
                };
                let Some(segment_index) =
                    segment_index_for_wheel_target(path.control_points.len(), path.is_closed, node.point_index, target)
                else {
                    continue;
                };
                if edited_segments.contains(&(node.path_index, segment_index)) {
                    continue;
                }
                if let Some(psi) = path.psis.get_mut(segment_index) {
                    let next = (*psi + step).clamp(-2.0, 2.0);
                    if (next - *psi).abs() > f32::EPSILON {
                        *psi = next;
                        changed = true;
                    }
                    edited_segments.push((node.path_index, segment_index));
                }
            }
        }
        ParamWheelTarget::PhiPrev | ParamWheelTarget::PhiNext => {
            let step = 0.20 * direction;
            let mut edited_segments: Vec<(usize, usize)> = Vec::new();
            for node in nodes {
                let Some(path) = layer.paths.get_mut(node.path_index) else {
                    continue;
                };
                let Some(segment_index) =
                    segment_index_for_wheel_target(path.control_points.len(), path.is_closed, node.point_index, target)
                else {
                    continue;
                };
                if edited_segments.contains(&(node.path_index, segment_index)) {
                    continue;
                }
                if let Some(phi) = path.phis.get_mut(segment_index) {
                    let next = (*phi + step).clamp(0.05, 8.0);
                    if (next - *phi).abs() > f32::EPSILON {
                        *phi = next;
                        changed = true;
                    }
                    edited_segments.push((node.path_index, segment_index));
                }
            }
        }
        ParamWheelTarget::FillHue
        | ParamWheelTarget::FillBrightness
        | ParamWheelTarget::FillAlpha
        | ParamWheelTarget::PaintHue
        | ParamWheelTarget::PaintBrightness
        | ParamWheelTarget::PaintAlpha => {}
    }

    if changed {
        app.push_timed_undo_snapshot(snapshot);
    }

    changed
}


fn is_global_wheel_target(target: ParamWheelTarget) -> bool {
    matches!(
        target,
        ParamWheelTarget::StrokeWidth
            | ParamWheelTarget::FillHue
            | ParamWheelTarget::FillBrightness
            | ParamWheelTarget::FillAlpha
            | ParamWheelTarget::PaintHue
            | ParamWheelTarget::PaintBrightness
            | ParamWheelTarget::PaintAlpha
    )
}

fn requires_selected_nodes(target: ParamWheelTarget) -> bool {
    matches!(
        target,
        ParamWheelTarget::Weight
            | ParamWheelTarget::PsiPrev
            | ParamWheelTarget::PsiNext
            | ParamWheelTarget::PhiPrev
            | ParamWheelTarget::PhiNext
    )
}

fn is_fill_color_target(target: ParamWheelTarget) -> bool {
    matches!(
        target,
        ParamWheelTarget::FillHue | ParamWheelTarget::FillBrightness | ParamWheelTarget::FillAlpha
    )
}

fn is_paint_color_target(target: ParamWheelTarget) -> bool {
    matches!(
        target,
        ParamWheelTarget::PaintHue
            | ParamWheelTarget::PaintBrightness
            | ParamWheelTarget::PaintAlpha
    )
}

fn adjust_color_channel_by_wheel(color: &mut [u8; 4], target: ParamWheelTarget, direction: f32) -> bool {
    let before = *color;
    let delta = (direction * 6.0).round() as i32;
    let edit = |value: &mut u8| {
        *value = ((*value as i32) + delta).clamp(0, 255) as u8;
    };

    match target {
        ParamWheelTarget::FillHue | ParamWheelTarget::PaintHue => edit(&mut color[0]),
        ParamWheelTarget::FillBrightness | ParamWheelTarget::PaintBrightness => edit(&mut color[1]),
        ParamWheelTarget::FillAlpha | ParamWheelTarget::PaintAlpha => edit(&mut color[2]),
        _ => {}
    }

    *color != before
}

fn adjust_color_by_wheel(app: &mut VectorEditorApp, target: ParamWheelTarget, direction: f32) -> bool {
    if is_paint_color_target(target)
        && app
            .document
            .active_layer()
            .map(|layer| layer.kind == crate::model::document::LayerKind::Raster)
            .unwrap_or(false)
    {
        return adjust_color_channel_by_wheel(&mut app.raster_color, target, direction);
    }

    let snapshot = app.document.clone();
    let selected_path = app.selected_path;

    let Some(layer) = app.document.active_layer_mut() else {
        return false;
    };
    if layer.kind != crate::model::document::LayerKind::Vector {
        return false;
    }
    let Some(path) = layer.paths.get_mut(selected_path) else {
        return false;
    };

    let changed = if is_fill_color_target(target) {
        adjust_color_channel_by_wheel(&mut path.fill_color, target, direction)
    } else {
        adjust_color_channel_by_wheel(&mut path.stroke_color, target, direction)
    };

    if changed {
        app.push_timed_undo_snapshot(snapshot);
    }

    changed
}


fn log_wheel_size(current: f32, direction: f32, min: f32, max: f32) -> f32 {
    // Multiplicative change feels natural across both small and large brush sizes.
    // Wheel up grows by about 12%; wheel down shrinks by the same ratio.
    let factor = if direction > 0.0 { 1.12 } else { 1.0 / 1.12 };
    (current.max(min) * factor).clamp(min, max)
}

fn segment_index_for_wheel_target(
    point_count: usize,
    is_closed: bool,
    point_index: usize,
    target: ParamWheelTarget,
) -> Option<usize> {
    if point_count < 2 || point_index >= point_count {
        return None;
    }

    match target {
        ParamWheelTarget::Weight
        | ParamWheelTarget::StrokeWidth
        | ParamWheelTarget::FillHue
        | ParamWheelTarget::FillBrightness
        | ParamWheelTarget::FillAlpha
        | ParamWheelTarget::PaintHue
        | ParamWheelTarget::PaintBrightness
        | ParamWheelTarget::PaintAlpha => None,
        ParamWheelTarget::PsiPrev | ParamWheelTarget::PhiPrev => {
            if is_closed {
                Some((point_index + point_count - 1) % point_count)
            } else if point_index > 0 {
                Some(point_index - 1)
            } else {
                None
            }
        }
        ParamWheelTarget::PsiNext | ParamWheelTarget::PhiNext => {
            if is_closed {
                Some(point_index)
            } else if point_index + 1 < point_count {
                Some(point_index)
            } else {
                None
            }
        }
    }
}


fn svg_panel(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    ui.label("SVG file path");
    ui.text_edit_singleline(&mut app.svg_file_path);

    ui.horizontal(|ui| {
        if ui.button("Save SVG").on_hover_text("Ctrl+S").clicked() {
            app.save_svg();
        }
        if ui.button("Load SVG...").on_hover_text("Ctrl+O").clicked() {
            app.choose_svg_file_and_load();
        }
    });

    ui.separator();
    ui.label("PNG export");
    ui.horizontal(|ui| {
        ui.label("Folder / full path");
        ui.text_edit_singleline(&mut app.png_file_path);
    });
    ui.horizontal(|ui| {
        ui.label("File name");
        ui.text_edit_singleline(&mut app.png_file_name);
    });
    ui.checkbox(&mut app.png_transparent_empty, "Transparent empty pixels");
    png_quality_ui(app, ui);
    ui.horizontal(|ui| {
        if ui.button("Choose Folder").clicked() {
            app.choose_png_folder();
        }
        if ui.button("Export PNG...").clicked() {
            app.choose_png_path_and_save();
        }
    });

    ui.separator();
    ui.label(&app.io_status);
    ui.small("SVG keeps editable PPW data. PNG exports the current canvas size.");
}

fn layer_panel(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("Add Vector Layer").clicked() {
            app.push_undo_snapshot();
            app.document.add_vector_layer();
            app.selected_path = 0;
            app.clear_selection_state();
        }

        if ui.button("Add Raster Layer").clicked() {
            app.push_undo_snapshot();
            app.document.add_raster_layer();
            app.selected_path = 0;
            app.clear_selection_state();
            app.active_tool = Tool::RasterBrush;
        }

        if ui.button("Duplicate").clicked() {
            app.push_undo_snapshot();
            app.document.duplicate_active_layer();
            app.selected_path = 0;
            app.clear_selection_state();
        }
    });

    ui.horizontal(|ui| {
        if ui.button("Delete Layer").clicked() {
            app.push_undo_snapshot();
            app.document.delete_active_layer();
            app.selected_path = 0;
            app.clear_selection_state();
        }
    });

    ui.small("Drag a layer row to reorder it. Top row is drawn above lower rows.");
    ui.separator();

    let mut next_active_layer = app.document.active_layer;
    let mut visibility_or_lock_changed = false;
    let visibility_snapshot = app.document.clone();
    let mut drop_target: Option<usize> = None;

    for i in (0..app.document.layers.len()).rev() {
        let selected = i == app.document.active_layer;
        let layer_name = app.document.layers[i].name.clone();
        let layer_kind = app.document.layers[i].kind.label();
        let is_drag_source = app.layer_drag_source == Some(i);

        ui.horizontal(|ui| {
            let drag_text = if is_drag_source { "==" } else { "::" };
            let drag_response = ui
                .add(egui::Label::new(drag_text).sense(egui::Sense::click_and_drag()))
                .on_hover_text("Drag to reorder layer");

            if drag_response.drag_started() {
                app.layer_drag_source = Some(i);
            }
            if drag_response.hovered() && ui.input(|input| input.pointer.any_released()) {
                drop_target = Some(i);
            }

            let row_response = ui.selectable_label(selected, format!("{}: [{}] {}", i + 1, layer_kind, layer_name));
            if row_response.clicked() {
                next_active_layer = i;
            }
            if row_response.drag_started() {
                app.layer_drag_source = Some(i);
            }
            if row_response.hovered() && ui.input(|input| input.pointer.any_released()) {
                drop_target = Some(i);
            }

            visibility_or_lock_changed |= ui
                .checkbox(&mut app.document.layers[i].visible, "V")
                .on_hover_text("Visible")
                .changed();

            visibility_or_lock_changed |= ui
                .checkbox(&mut app.document.layers[i].locked, "L")
                .on_hover_text("Locked")
                .changed();
        });
    }

    if ui.input(|input| input.pointer.any_released()) {
        if let (Some(from), Some(to)) = (app.layer_drag_source.take(), drop_target) {
            if from != to && from < app.document.layers.len() && to < app.document.layers.len() {
                app.push_undo_snapshot();
                let layer = app.document.layers.remove(from);
                let insert_at = if from < to { to.saturating_sub(1) } else { to };
                app.document.layers.insert(insert_at, layer);
                app.document.active_layer = insert_at;
                app.selected_path = 0;
                app.clear_selection_state();
            }
        } else {
            app.layer_drag_source = None;
        }
    }

    if next_active_layer != app.document.active_layer && app.layer_drag_source.is_none() {
        app.document.active_layer = next_active_layer;
        app.selected_path = 0;
        app.clear_selection_state();
    }

    if visibility_or_lock_changed {
        app.undo_stack.push(visibility_snapshot);
        if app.undo_stack.len() > 100 {
            app.undo_stack.remove(0);
        }
        app.redo_stack.clear();
    }

    let name_snapshot = app.document.clone();
    let mut name_changed = false;
    if let Some(layer) = app.document.active_layer_mut() {
        ui.separator();
        ui.label("Active Layer Name");
        name_changed = ui.text_edit_singleline(&mut layer.name).changed();
    }
    if name_changed {
        app.undo_stack.push(name_snapshot);
        if app.undo_stack.len() > 100 {
            app.undo_stack.remove(0);
        }
        app.redo_stack.clear();
    }
}
fn path_panel(app: &mut VectorEditorApp, ui: &mut egui::Ui) {
    let active_layer_locked = app
        .document
        .active_layer()
        .map(|layer| layer.locked)
        .unwrap_or(true);

    if active_layer_locked {
        ui.colored_label(egui::Color32::DARK_RED, "Active layer is locked.");
    }

    if app.document.active_layer().map(|l| l.kind == crate::model::document::LayerKind::Raster).unwrap_or(false) {
        ui.separator();
        ui.label("Raster Layer");
        ui.add_enabled_ui(!active_layer_locked, |ui| {
            ui.add(egui::Slider::new(&mut app.raster_brush_width, 0.5..=128.0).text("Brush Width"));
            ui.add(egui::Slider::new(&mut app.raster_eraser_width, 0.5..=256.0).text("Eraser Width"));
            color_hue_brightness_alpha_ui(ui, "Raster Brush Color", &mut app.raster_color);
            ui.small("Raster Brush replaces touched pixels with the selected RGBA. Eraser replaces touched pixels with RGBA(255,255,255,0). Z/X/C choose R/G/B wheel targets.");
        });
        if let Some(layer) = app.document.active_layer() {
            ui.label(format!("Raster pixels: {} x {}", layer.raster_width, layer.raster_height));
        }
        return;
    }

    ui.separator();
    ui.label("Canvas Size");
    let canvas_snapshot = app.document.clone();
    let mut canvas_changed = false;
    ui.horizontal(|ui| {
        canvas_changed |= ui
            .add(egui::DragValue::new(&mut app.document.canvas_width).speed(10.0).range(1.0..=10000.0).suffix(" px").prefix("W "))
            .changed();
        canvas_changed |= ui
            .add(egui::DragValue::new(&mut app.document.canvas_height).speed(10.0).range(1.0..=10000.0).suffix(" px").prefix("H "))
            .changed();
    });
    if canvas_changed {
        app.document.canvas_width = app.document.canvas_width.max(1.0);
        app.document.canvas_height = app.document.canvas_height.max(1.0);
        app.undo_stack.push(canvas_snapshot);
        if app.undo_stack.len() > 100 {
            app.undo_stack.remove(0);
        }
        app.redo_stack.clear();
    }

    ui.separator();
    ui.label("Path Drawing");
    ui.add_enabled_ui(!active_layer_locked, |ui| {
        if ui.button("Finish Current Path").on_hover_text("End current path and create a new empty path in this vector layer").clicked() {
            app.finish_current_path();
        }
        ui.horizontal(|ui| {
            ui.label("Simplify tolerance");
            ui.add(egui::DragValue::new(&mut app.simplify_tolerance).speed(0.5).range(0.0..=200.0));
        });
        if ui.button("Simplify Selected Path").on_hover_text("Remove unnecessary control points from the selected path").clicked() {
            app.simplify_selected_path();
        }
        ui.small("Shift+V: resume Add Point on the currently selected finished path.");
    });

    ui.separator();
    ui.label("Selected Path Size");
    ui.add_enabled_ui(!active_layer_locked && app.active_tool == Tool::Select, |ui| {
        ui.horizontal(|ui| {
            ui.label("Scale");
            ui.add(egui::DragValue::new(&mut app.path_scale_x).speed(0.01).range(0.01..=100.0).prefix("X "));
            ui.add(egui::DragValue::new(&mut app.path_scale_y).speed(0.01).range(0.01..=100.0).prefix("Y "));
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Apply Scale").on_hover_text("Scale the selected vector path around its center").clicked() {
                app.scale_selected_path(app.path_scale_x, app.path_scale_y);
                app.path_scale_x = 1.0;
                app.path_scale_y = 1.0;
            }
            if ui.button("90%").clicked() {
                app.scale_selected_path(0.9, 0.9);
            }
            if ui.button("110%").clicked() {
                app.scale_selected_path(1.1, 1.1);
            }
            if ui.button("Wider").clicked() {
                app.scale_selected_path(1.1, 1.0);
            }
            if ui.button("Taller").clicked() {
                app.scale_selected_path(1.0, 1.1);
            }
        });
        ui.small("Available while Select tool is active. The selected path is scaled around its bounding-box center.");
    });

    ui.separator();
    ui.label("New Shape Default Stroke");
    ui.add_enabled_ui(!active_layer_locked, |ui| {
        ui.add(egui::Slider::new(&mut app.brush_width, 0.5..=32.0).text("New Shape Stroke Width"));
        ui.small("Rectangle/Ellipse tools use this width for newly created PPW paths.");
    });

    let path_exists = app
        .document
        .active_layer()
        .and_then(|layer| layer.paths.get(app.selected_path))
        .is_some();

    if !path_exists {
        ui.label("No active path.");
        return;
    }

    ui.separator();
    ui.label(format!("Selected Points Across Paths: {}", app.selected_nodes.len()));
    ui.label(format!(
        "Wheel PPW Target: {}",
        app.param_wheel_target.map(|target| target.label()).unwrap_or("None")
    ));
    ui.small("Select tool + selected point(s): Q=Weight, W/E=Psi, S/D=Phi. R=Stroke/Brush width. Z/X/C=Red/Green/Blue color wheel targets.");
    if !app.selected_nodes.is_empty() {
        let before_multi_edit = app.document.clone();
        let mut dx = 0.0_f32;
        let mut dy = 0.0_f32;
        let mut moved = false;
        ui.add_enabled_ui(!active_layer_locked, |ui| {
            ui.horizontal(|ui| {
                ui.label("Move selection by:");
                moved |= ui.add(egui::DragValue::new(&mut dx).speed(1.0).prefix("dx ")).changed();
                moved |= ui.add(egui::DragValue::new(&mut dy).speed(1.0).prefix("dy ")).changed();
            });
        });
        if moved && (dx != 0.0 || dy != 0.0) {
            move_selected_nodes_by_delta_in_view(&mut app.document, &app.selected_nodes, crate::ppw::Vec2::new(dx, dy));
            app.undo_stack.push(before_multi_edit);
            if app.undo_stack.len() > 100 {
                app.undo_stack.remove(0);
            }
            app.redo_stack.clear();
        }
        ui.small("Rectangle selection can include points from every path in the active layer.");
    }

    let path_count = app
        .document
        .active_layer()
        .map(|layer| layer.paths.len())
        .unwrap_or(0);
    ui.separator();
    ui.label("Paths");
    for i in 0..path_count {
        if ui
            .selectable_label(app.selected_path == i, format!("Path {}", i + 1))
            .clicked()
        {
            app.selected_path = i;
            app.clear_selection_state();
        }
    }

    let before_edit = app.document.clone();
    let mut changed = false;
    let selected_point = app.selected_point;
    let selected_points_count = if app.selected_nodes.is_empty() { app.selected_points.len() } else { app.selected_nodes.len() };

    {
        let Some(path) = app.document.active_path_mut(app.selected_path) else {
            return;
        };

        let old_closed = path.is_closed;
        ui.add_enabled_ui(!active_layer_locked, |ui| {
            changed |= ui.checkbox(&mut path.is_closed, "Close Path").changed();
        });
        if path.is_closed != old_closed {
            path.rebuild_open_segment_params();
        }

        ui.separator();
        ui.label("Fill");
        ui.add_enabled_ui(!active_layer_locked && path.is_closed && path.control_points.len() >= 3, |ui| {
            changed |= ui.checkbox(&mut path.fill_enabled, "Enable Fill").changed();
            changed |= color_hue_brightness_alpha_ui(ui, "Fill Color", &mut path.fill_color);
        });
        if !path.is_closed {
            ui.small("Fill is available only for closed paths.");
        }

        ui.separator();
        ui.label("Stroke");
        ui.add_enabled_ui(!active_layer_locked, |ui| {
            changed |= ui
                .add(egui::Slider::new(&mut path.stroke_width, 0.1..=32.0).text("Stroke Width"))
                .changed();
            changed |= color_hue_brightness_alpha_ui(ui, "Stroke Color", &mut path.stroke_color);
        });

        ui.label(format!("Control Points: {}", path.control_points.len()));
        ui.label(format!("Selected Points: {}", selected_points_count));

        if let Some(index) = selected_point {
            if index < path.control_points.len() {
                ui.separator();
                ui.label(format!("Selected Point: {index}"));

                ui.add_enabled_ui(!active_layer_locked, |ui| {
                    changed |= ui
                        .add(egui::DragValue::new(&mut path.control_points[index].x).speed(1.0).prefix("x: "))
                        .changed();
                    changed |= ui
                        .add(egui::DragValue::new(&mut path.control_points[index].y).speed(1.0).prefix("y: "))
                        .changed();
                });

                ui.separator();
                ui.label("PPW Parameters for selected point only");

                ui.add_enabled_ui(!active_layer_locked, |ui| {
                    changed |= ui
                        .add(egui::Slider::new(&mut path.weights[index], 0.1..=4.0).text(format!("weight[{index}]")))
                        .changed();

                    for segment_index in editable_segment_indices(path.control_points.len(), path.is_closed, index) {
                        if segment_index < path.phis.len() && segment_index < path.psis.len() {
                            ui.group(|ui| {
                                ui.label(format!("Segment {segment_index}"));
                                changed |= ui
                                    .add(egui::Slider::new(&mut path.phis[segment_index], 0.05..=8.0).text(format!("phi[{segment_index}]")))
                                    .changed();
                                changed |= ui
                                    .add(egui::Slider::new(&mut path.psis[segment_index], -2.0..=2.0).text(format!("psi[{segment_index}]")))
                                    .changed();
                            });
                        }
                    }
                });

                ui.small("For point i, only weight[i] and adjacent phi/psi segments are editable.");
            } else {
                ui.label("Selected point is out of range.");
            }
        } else {
            ui.separator();
            ui.label("PPW Parameters");
            ui.small("Select one or more points. Multiple selected points can be moved together; Weight/Phi/Psi editing is shown for the active point.");
        }
    }

    if changed {
        app.push_timed_undo_snapshot(before_edit);
    }
}


fn color_hue_brightness_alpha_ui(ui: &mut egui::Ui, label: &str, color: &mut [u8; 4]) -> bool {
    let before = *color;

    let (mut hue, mut saturation, mut value) = rgb_to_hsv(color[0], color[1], color[2]);
    if saturation <= 0.001 {
        saturation = 1.0;
    }
    let mut alpha = color[3] as f32 / 255.0;

    let mut rgba = [
        color[0] as i32,
        color[1] as i32,
        color[2] as i32,
        color[3] as i32,
    ];

    let mut hsv_changed = false;
    let mut rgba_changed = false;

    ui.group(|ui| {
        ui.label(label);

        ui.horizontal(|ui| {
            let preview = egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            let (rect, _) = ui.allocate_exact_size(egui::vec2(34.0, 18.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 3.0, preview);
            ui.painter().rect_stroke(
                rect,
                3.0,
                egui::Stroke::new(1.0, egui::Color32::GRAY),
                egui::StrokeKind::Inside,
            );
            ui.label(format!("rgba({}, {}, {}, {})", color[0], color[1], color[2], color[3]));
        });

        ui.separator();
        ui.label("Hue / Brightness / Alpha");
        hsv_changed |= ui.add(egui::Slider::new(&mut hue, 0.0..=360.0).text("Hue")).changed();
        hsv_changed |= ui.add(egui::Slider::new(&mut value, 0.0..=1.0).text("Brightness")).changed();
        hsv_changed |= ui.add(egui::Slider::new(&mut alpha, 0.0..=1.0).text("Alpha")).changed();

        ui.separator();
        ui.label("RGBA");
        rgba_changed |= ui.add(egui::Slider::new(&mut rgba[0], 0..=255).text("R")).changed();
        rgba_changed |= ui.add(egui::Slider::new(&mut rgba[1], 0..=255).text("G")).changed();
        rgba_changed |= ui.add(egui::Slider::new(&mut rgba[2], 0..=255).text("B")).changed();
        rgba_changed |= ui.add(egui::Slider::new(&mut rgba[3], 0..=255).text("A")).changed();

        ui.small("Z/X/C wheel editing controls R / G / B. RGBA sliders allow direct numeric color adjustment.");
    });

    if rgba_changed {
        *color = [
            rgba[0].clamp(0, 255) as u8,
            rgba[1].clamp(0, 255) as u8,
            rgba[2].clamp(0, 255) as u8,
            rgba[3].clamp(0, 255) as u8,
        ];
    } else if hsv_changed {
        let (r, g, b) = hsv_to_rgb(hue, saturation, value);
        *color = [r, g, b, (alpha * 255.0).round().clamp(0.0, 255.0) as u8];
    }

    *color != before
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta <= f32::EPSILON {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    let saturation = if max <= f32::EPSILON { 0.0 } else { delta / max };
    (hue, saturation, max)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let h = (hue / 60.0).rem_euclid(6.0);
    let c = value * saturation.clamp(0.0, 1.0);
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = value - c;
    let (r1, g1, b1) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

fn move_selected_nodes_by_delta_in_view(
    document: &mut crate::model::document::Document,
    selected_nodes: &[crate::app::PointSelection],
    delta: crate::ppw::Vec2,
) {
    let Some(layer) = document.active_layer_mut() else {
        return;
    };

    let mut grouped: Vec<(usize, Vec<usize>)> = Vec::new();
    for node in selected_nodes {
        if let Some((_, points)) = grouped.iter_mut().find(|(path_index, _)| *path_index == node.path_index) {
            points.push(node.point_index);
        } else {
            grouped.push((node.path_index, vec![node.point_index]));
        }
    }

    for (path_index, mut points) in grouped {
        points.sort_unstable();
        points.dedup();
        if let Some(path) = layer.paths.get_mut(path_index) {
            path.move_points_by_delta(&points, delta);
        }
    }
}

fn editable_segment_indices(point_count: usize, is_closed: bool, point_index: usize) -> Vec<usize> {
    if point_count < 2 || point_index >= point_count {
        return Vec::new();
    }

    if is_closed {
        let prev = (point_index + point_count - 1) % point_count;
        let next = point_index;
        if prev == next {
            vec![prev]
        } else {
            vec![prev, next]
        }
    } else {
        let mut indices = Vec::new();
        if point_index > 0 {
            indices.push(point_index - 1);
        }
        if point_index + 1 < point_count {
            indices.push(point_index);
        }
        indices
    }
}
