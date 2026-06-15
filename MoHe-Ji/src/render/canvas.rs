use crate::app::{PointSelection, SegmentSelection, Tool, TransformDragHandle};
use crate::model::document::{ensure_layer_raster_canvas, Document, LayerKind, RasterImage, RasterStroke};
use std::collections::HashMap;
use crate::ppw::{path::PPWPath, shape_util::ShapeUtil, PPWCurve, Vec2};

pub struct CanvasView<'a> {
    pub document: &'a mut Document,
    pub show_points: bool,
    pub show_segments: bool,
    pub show_fill: bool,
    pub show_triangles: bool,
    pub active_tool: Tool,
    pub selected_path: &'a mut usize,
    pub selected_point: &'a mut Option<usize>,
    pub selected_points: &'a mut Vec<usize>,
    pub selected_nodes: &'a mut Vec<PointSelection>,
    pub selected_segment: &'a mut Option<SegmentSelection>,
    pub dragging_point: &'a mut Option<usize>,
    pub drag_last_pos: &'a mut Option<Vec2>,
    pub selection_rect_start: &'a mut Option<Vec2>,
    pub selection_rect_current: &'a mut Option<Vec2>,
    pub undo_stack: &'a mut Vec<Document>,
    pub redo_stack: &'a mut Vec<Document>,
    pub pending_drag_snapshot: &'a mut Option<Document>,
    pub brush_points: &'a mut Vec<Vec2>,
    pub brush_width: f32,
    pub raster_brush_width: f32,
    pub raster_eraser_width: f32,
    pub raster_color: [u8; 4],
    pub zoom: f32,
    pub pan_offset: &'a mut Vec2,
    pub shape_start: &'a mut Option<Vec2>,
    pub shape_current: &'a mut Option<Vec2>,
    pub selected_image: &'a mut Option<usize>,
    pub resizing_image: &'a mut bool,
    pub resizing_path: &'a mut bool,
    pub transform_mode: &'a mut bool,
    pub rotating_selection: &'a mut bool,
    pub active_transform_handle: &'a mut Option<TransformDragHandle>,
    pub transform_anchor: &'a mut Option<Vec2>,
    pub image_textures: &'a mut HashMap<u64, egui::TextureHandle>,
}

impl<'a> CanvasView<'a> {
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(245));

        if response.hovered() && ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
            let delta = ui.input(|i| i.pointer.delta());
            if delta != egui::Vec2::ZERO {
                self.pan_offset.x += delta.x;
                self.pan_offset.y += delta.y;
                ui.ctx().request_repaint();
            }
        }

        let canvas_origin = rect.left_top().to_vec2();
        let pan = *self.pan_offset;
        let zoom = self.zoom.max(0.1);
        let to_screen = |p: Vec2| -> egui::Pos2 {
            egui::pos2(canvas_origin.x + pan.x + p.x * zoom, canvas_origin.y + pan.y + p.y * zoom)
        };
        let from_screen = |p: egui::Pos2| -> Vec2 {
            Vec2::new((p.x - canvas_origin.x - pan.x) / zoom, (p.y - canvas_origin.y - pan.y) / zoom)
        };

        draw_grid(&painter, rect, zoom, pan);


        let canvas_rect = egui::Rect::from_min_size(
            to_screen(Vec2::ZERO),
            egui::vec2(self.document.canvas_width * zoom, self.document.canvas_height * zoom),
        );
        painter.rect_filled(canvas_rect, 0.0, egui::Color32::WHITE);
        painter.rect_stroke(
            canvas_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
            egui::StrokeKind::Inside,
        );

        let pointer_pos = response.interact_pointer_pos().map(from_screen);
        let shift = ui.input(|i| i.modifiers.shift);
        let active_layer_locked = self
            .document
            .active_layer()
            .map(|layer| layer.locked)
            .unwrap_or(true);

        if !active_layer_locked {
            if response.drag_started() && ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                if let Some(pos) = pointer_pos {
                    match self.active_tool {
                        Tool::Select => {
                            let mut transform_started = false;
                            if *self.transform_mode {
                                if let Some(handle) = hit_transform_handle(self.document, *self.selected_path, *self.selected_image, pos, 14.0 / zoom) {
                                    *self.active_transform_handle = Some(handle.to_drag());
                                    *self.rotating_selection = matches!(handle, TransformHandle::Rotate);
                                    *self.resizing_path = !matches!(handle, TransformHandle::Rotate) && self.selected_image.is_none();
                                    *self.resizing_image = !matches!(handle, TransformHandle::Rotate) && self.selected_image.is_some();
                                    *self.drag_last_pos = Some(pos);
                                    if matches!(handle, TransformHandle::Rotate) {
                                        *self.transform_anchor = selected_transform_center(self.document, *self.selected_path, *self.selected_image);
                                    } else {
                                        *self.transform_anchor = None;
                                    }
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                    transform_started = true;
                                }
                            }
                            if !transform_started {
                                if let Some((image_index, resizing)) = hit_image_in_active_layer(self.document, pos, 10.0 / zoom) {
                                    *self.selected_image = Some(image_index);
                                    *self.resizing_image = resizing;
                                    *self.active_transform_handle = if resizing { Some(TransformDragHandle::UniformResize) } else { None };
                                    self.selected_nodes.clear();
                                    self.selected_points.clear();
                                    *self.selected_point = None;
                                    *self.selected_segment = None;
                                    *self.drag_last_pos = Some(pos);
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                } else if let Some(path_index) = hit_selected_path_resize_handle(self.document, *self.selected_path, pos, 10.0 / zoom) {
                                    *self.selected_path = path_index;
                                    *self.resizing_path = true;
                                    *self.active_transform_handle = Some(TransformDragHandle::UniformResize);
                                    *self.selected_image = None;
                                    *self.drag_last_pos = Some(pos);
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                } else if let Some((path_index, point_index)) =
                                    nearest_control_point_in_active_layer(self.document, pos, 10.0 / zoom)
                                {
                                    *self.selected_path = path_index;
                                    let node = PointSelection { path_index, point_index };

                                    if shift {
                                        toggle_node(self.selected_nodes, node);
                                    } else if !self.selected_nodes.contains(&node) {
                                        self.selected_nodes.clear();
                                        self.selected_nodes.push(node);
                                    }

                                    sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                    *self.selected_segment = None;
                                    *self.selected_point = Some(point_index);
                                    *self.dragging_point = Some(point_index);
                                    *self.drag_last_pos = Some(pos);
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                } else if let Some((path_index, segment_index)) =
                                    nearest_path_segment_in_active_layer(self.document, pos, 8.0 / zoom)
                                {
                                    *self.selected_path = path_index;
                                    *self.selected_segment = Some(SegmentSelection { path_index, segment_index });
                                    if !shift {
                                        self.selected_nodes.clear();
                                        self.selected_points.clear();
                                        *self.selected_point = None;
                                    }
                                } else {
                                    if !shift {
                                        self.selected_nodes.clear();
                                        self.selected_points.clear();
                                        *self.selected_point = None;
                                        *self.selected_segment = None;
                                    }
                                    *self.selection_rect_start = Some(pos);
                                    *self.selection_rect_current = Some(pos);
                                }
                            }
                        }
                        Tool::AddPoint => {
                            if let Some(path) = self.document.active_path_mut(*self.selected_path) {
                                if let Some(hit) = path.nearest_control_point(pos, 10.0 / zoom) {
                                    self.selected_nodes.clear();
                                    self.selected_nodes.push(PointSelection {
                                        path_index: *self.selected_path,
                                        point_index: hit,
                                    });
                                    sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                    *self.selected_segment = None;
                                    *self.selected_point = Some(hit);
                                    *self.dragging_point = Some(hit);
                                    *self.drag_last_pos = Some(pos);
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                } else {
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                    if let Some(path) = self.document.active_path_mut(*self.selected_path) {
                                        path.add_point(pos);
                                        let new_index = path.control_points.len().saturating_sub(1);
                                        self.selected_nodes.clear();
                                        self.selected_nodes.push(PointSelection {
                                            path_index: *self.selected_path,
                                            point_index: new_index,
                                        });
                                        sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                        *self.selected_segment = None;
                                        *self.selected_point = Some(new_index);
                                        *self.dragging_point = Some(new_index);
                                        *self.drag_last_pos = Some(pos);
                                    }
                                }
                            }
                        }
                        Tool::Rectangle | Tool::Ellipse => {
                            *self.shape_start = Some(pos);
                            *self.shape_current = Some(pos);
                            begin_edit(self.document, self.pending_drag_snapshot);
                        }
                        Tool::VectorBrush => {
                            if self.document.active_layer().map(|l| l.kind == LayerKind::Vector).unwrap_or(false) {
                                begin_edit(self.document, self.pending_drag_snapshot);
                                self.brush_points.clear();
                                self.brush_points.push(pos);
                            }
                        }
                        Tool::RasterBrush => {
                            if self.document.active_layer().map(|l| l.kind == LayerKind::Raster).unwrap_or(false) {
                                begin_edit(self.document, self.pending_drag_snapshot);
                                paint_raster_line(self.document, pos, pos, self.raster_brush_width * 0.5, self.raster_color);
                                self.brush_points.clear();
                                self.brush_points.push(pos);
                            }
                        }
                        Tool::RasterEraser => {
                            if self.document.active_layer().map(|l| l.kind == LayerKind::Raster).unwrap_or(false) {
                                begin_edit(self.document, self.pending_drag_snapshot);
                                erase_raster_line(self.document, pos, pos, self.raster_eraser_width * 0.5);
                                self.brush_points.clear();
                                self.brush_points.push(pos);
                            }
                        }
                        Tool::Image => {}
                    }
                }
            }

            if response.dragged() && ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)) {
                if let Some(pos) = pointer_pos {
                    if self.active_tool == Tool::VectorBrush && !self.brush_points.is_empty() {
                        if self.brush_points.last().map(|p| (*p - pos).length_squared() > 1.0).unwrap_or(true) {
                            self.brush_points.push(pos);
                            ui.ctx().request_repaint();
                        }
                    } else if self.active_tool == Tool::RasterBrush && !self.brush_points.is_empty() {
                        let prev = *self.brush_points.last().unwrap_or(&pos);
                        if (prev - pos).length_squared() > 0.01 {
                            paint_raster_line(self.document, prev, pos, self.raster_brush_width * 0.5, self.raster_color);
                            self.brush_points.push(pos);
                            ui.ctx().request_repaint();
                        }
                    } else if self.active_tool == Tool::RasterEraser && !self.brush_points.is_empty() {
                        let prev = *self.brush_points.last().unwrap_or(&pos);
                        if (prev - pos).length_squared() > 0.01 {
                            erase_raster_line(self.document, prev, pos, self.raster_eraser_width * 0.5);
                            self.brush_points.push(pos);
                            ui.ctx().request_repaint();
                        }
                    } else if matches!(self.active_tool, Tool::Rectangle | Tool::Ellipse) && self.shape_start.is_some() {
                        *self.shape_current = Some(if shift {
                            square_constrained_point(self.shape_start.unwrap(), pos)
                        } else {
                            pos
                        });
                        ui.ctx().request_repaint();
                    } else if *self.rotating_selection && self.drag_last_pos.is_some() {
                        if let Some(prev) = *self.drag_last_pos {
                            let center = match *self.transform_anchor {
                                Some(center) => Some(center),
                                None => {
                                    let center = selected_transform_center(self.document, *self.selected_path, *self.selected_image);
                                    *self.transform_anchor = center;
                                    center
                                }
                            };
                            if let Some(center) = center {
                                let a0 = (prev.y - center.y).atan2(prev.x - center.x);
                                let a1 = (pos.y - center.y).atan2(pos.x - center.x);
                                let delta_angle = a1 - a0;
                                if delta_angle.abs() > 0.0001 {
                                    rotate_selected_transform(self.document, *self.selected_path, *self.selected_image, center, delta_angle);
                                    *self.drag_last_pos = Some(pos);
                                    ui.ctx().request_repaint();
                                }
                            }
                        }
                    } else if (*self.resizing_path || *self.resizing_image) && self.drag_last_pos.is_some() {
                        if let Some(prev) = *self.drag_last_pos {
                            let delta = pos - prev;
                            if !delta.is_zero_approx() {
                                let handle = self.active_transform_handle.unwrap_or(TransformDragHandle::UniformResize);
                                resize_selected_transform_from_drag(self.document, *self.selected_path, *self.selected_image, handle, delta);
                                *self.drag_last_pos = Some(pos);
                                ui.ctx().request_repaint();
                            }
                        }
                    } else if self.selected_image.is_some() && self.drag_last_pos.is_some() {
                        if let Some(prev) = *self.drag_last_pos {
                            let delta = pos - prev;
                            if !delta.is_zero_approx() {
                                move_or_resize_selected_image(self.document, *self.selected_image, false, delta, shift);
                                *self.drag_last_pos = Some(pos);
                                ui.ctx().request_repaint();
                            }
                        }
                    } else if self.selection_rect_start.is_some() {
                        *self.selection_rect_current = Some(pos);
                        ui.ctx().request_repaint();
                    } else if let Some(prev) = *self.drag_last_pos {
                        let delta = pos - prev;
                        if !delta.is_zero_approx() {
                            move_selected_points_by_delta(
                                self.document,
                                *self.selected_path,
                                self.selected_points,
                                self.selected_nodes,
                                delta,
                            );
                            *self.drag_last_pos = Some(pos);
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }

            if response.drag_stopped() {
                if self.active_tool == Tool::VectorBrush && !self.brush_points.is_empty() {
                    if self.brush_points.len() >= 2 {
                        if let Some(paths) = self.document.active_paths_mut() {
                            let mut path = PPWPath::empty();
                            path.control_points = sample_polyline(self.brush_points, 2.0);
                            path.stroke_width = self.brush_width.max(0.1);
                            path.stroke_color = [0, 0, 0, 255];
                            path.fill_enabled = false;
                            path.rebuild_open_segment_params();
                            path.simplify_control_points(8.0);
                            paths.push(path);
                            *self.selected_path = paths.len().saturating_sub(1);
                            self.selected_nodes.clear();
                            if let Some(new_path) = paths.get(*self.selected_path) {
                                for point_index in 0..new_path.control_points.len() {
                                    self.selected_nodes.push(PointSelection { path_index: *self.selected_path, point_index });
                                }
                            }
                            sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                            *self.selected_segment = None;
                            *self.selected_point = self.selected_points.last().copied();
                            commit_edit(self.pending_drag_snapshot, self.undo_stack, self.redo_stack);
                        } else {
                            self.pending_drag_snapshot.take();
                        }
                    } else {
                        self.pending_drag_snapshot.take();
                    }
                    self.brush_points.clear();
                } else if self.active_tool == Tool::RasterBrush && !self.brush_points.is_empty() {
                    commit_edit(self.pending_drag_snapshot, self.undo_stack, self.redo_stack);
                    self.brush_points.clear();
                } else if self.active_tool == Tool::RasterEraser && !self.brush_points.is_empty() {
                    commit_edit(self.pending_drag_snapshot, self.undo_stack, self.redo_stack);
                    self.brush_points.clear();
                } else if matches!(self.active_tool, Tool::Rectangle | Tool::Ellipse) && self.shape_start.is_some() {
                    if let (Some(start), Some(end)) = (*self.shape_start, *self.shape_current) {
                        if (end - start).length_squared() >= 4.0 {
                            if let Some(paths) = self.document.active_paths_mut() {
                                match self.active_tool {
                                    Tool::Rectangle => {
                                        let mut rectangle_paths =
                                            PPWPath::from_rectangle_diagonal_paths(start, end, self.brush_width);
                                        let first_new_index = paths.len();
                                        paths.append(&mut rectangle_paths);
                                        *self.selected_path = first_new_index;
                                    }
                                    Tool::Ellipse => {
                                        let path = PPWPath::from_ellipse(start, end, self.brush_width);
                                        paths.push(path);
                                        *self.selected_path = paths.len().saturating_sub(1);
                                    }
                                    _ => unreachable!(),
                                }

                                self.selected_nodes.clear();
                                match self.active_tool {
                                    Tool::Rectangle => {
                                        for path_index in *self.selected_path..=(*self.selected_path + 1) {
                                            if let Some(path) = paths.get(path_index) {
                                                for point_index in 0..path.control_points.len() {
                                                    self.selected_nodes.push(PointSelection { path_index, point_index });
                                                }
                                            }
                                        }
                                    }
                                    Tool::Ellipse => {
                                        if let Some(path) = paths.get(*self.selected_path) {
                                            for point_index in 0..path.control_points.len() {
                                                self.selected_nodes.push(PointSelection { path_index: *self.selected_path, point_index });
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                *self.selected_segment = None;
                                *self.selected_point = self.selected_points.last().copied();
                            }
                            commit_edit(self.pending_drag_snapshot, self.undo_stack, self.redo_stack);
                        } else {
                            self.pending_drag_snapshot.take();
                        }
                    }
                    *self.shape_start = None;
                    *self.shape_current = None;
                } else if let (Some(start), Some(end)) =
                    (*self.selection_rect_start, *self.selection_rect_current)
                {
                    let mut hits = points_in_rect_in_active_layer(self.document, start, end);
                    hits.sort_unstable();
                    hits.dedup();

                    if shift {
                        for hit in hits {
                            if !self.selected_nodes.contains(&hit) {
                                self.selected_nodes.push(hit);
                            }
                        }
                        self.selected_nodes.sort_unstable();
                    } else {
                        *self.selected_nodes = hits;
                    }

                    if let Some(active) = self.selected_nodes.last().copied() {
                        *self.selected_path = active.path_index;
                        *self.selected_point = Some(active.point_index);
                    } else {
                        *self.selected_point = None;
                    }
                    sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                    *self.selected_segment = None;
                } else if self.pending_drag_snapshot.is_some() {
                    commit_edit(self.pending_drag_snapshot, self.undo_stack, self.redo_stack);
                }

                *self.dragging_point = None;
                *self.drag_last_pos = None;
                *self.resizing_image = false;
                *self.resizing_path = false;
                *self.rotating_selection = false;
                *self.active_transform_handle = None;
                *self.selection_rect_start = None;
                *self.selection_rect_current = None;
            }

            if response.clicked() {
                if let Some(pos) = pointer_pos {
                    match self.active_tool {
                        Tool::Select => {
                            if let Some((image_index, _resizing)) = hit_image_in_active_layer(self.document, pos, 10.0 / zoom) {
                                *self.selected_image = Some(image_index);
                                *self.resizing_image = false;
                                self.selected_nodes.clear();
                                self.selected_points.clear();
                                *self.selected_point = None;
                                *self.selected_segment = None;
                            } else if let Some((path_index, point_index)) =
                                nearest_control_point_in_active_layer(self.document, pos, 10.0 / zoom)
                            {
                                *self.selected_path = path_index;
                                let node = PointSelection { path_index, point_index };
                                if shift {
                                    toggle_node(self.selected_nodes, node);
                                } else {
                                    self.selected_nodes.clear();
                                    self.selected_nodes.push(node);
                                }
                                sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                *self.selected_segment = None;
                                *self.selected_point = self.selected_points.last().copied();
                            } else if let Some((path_index, segment_index)) =
                                nearest_path_segment_in_active_layer(self.document, pos, 8.0 / zoom)
                            {
                                *self.selected_path = path_index;
                                *self.selected_segment = Some(SegmentSelection { path_index, segment_index });
                                if !shift {
                                    self.selected_nodes.clear();
                                    self.selected_points.clear();
                                    *self.selected_point = None;
                                }
                            } else if !shift {
                                self.selected_nodes.clear();
                                self.selected_points.clear();
                                *self.selected_point = None;
                                *self.selected_segment = None;
                            }
                        }
                        Tool::AddPoint => {
                            if let Some(path) = self.document.active_path_mut(*self.selected_path) {
                                if let Some(hit) = path.nearest_control_point(pos, 10.0 / zoom) {
                                    self.selected_nodes.clear();
                                    self.selected_nodes.push(PointSelection {
                                        path_index: *self.selected_path,
                                        point_index: hit,
                                    });
                                    sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                    *self.selected_segment = None;
                                    *self.selected_point = Some(hit);
                                } else {
                                    begin_edit(self.document, self.pending_drag_snapshot);
                                    if let Some(path) = self.document.active_path_mut(*self.selected_path) {
                                        path.add_point(pos);
                                        let new_index = path.control_points.len().saturating_sub(1);
                                        self.selected_nodes.clear();
                                        self.selected_nodes.push(PointSelection {
                                            path_index: *self.selected_path,
                                            point_index: new_index,
                                        });
                                        sync_legacy_selection(*self.selected_path, self.selected_points, self.selected_nodes);
                                        *self.selected_segment = None;
                                        *self.selected_point = Some(new_index);
                                        commit_edit(self.pending_drag_snapshot, self.undo_stack, self.redo_stack);
                                    }
                                }
                            }
                        }
                        Tool::Rectangle | Tool::Ellipse | Tool::VectorBrush | Tool::RasterBrush | Tool::RasterEraser | Tool::Image => {}
                    }
                }
            }
        }

        draw_document(
            &painter,
            self.document,
            &to_screen,
            self.show_points,
            self.show_segments,
            self.show_fill,
            self.show_triangles,
            self.document.active_layer,
            *self.selected_path,
            self.selected_point,
            self.selected_points,
            self.selected_nodes,
            self.selected_segment,
            self.selected_image,
            *self.transform_mode,
            self.image_textures,
            ui.ctx(),
            zoom,
        );

        if self.active_tool == Tool::VectorBrush && self.brush_points.len() >= 2 {
            let points = self.brush_points.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
            painter.add(egui::Shape::line(points, egui::Stroke::new(
                self.brush_width * zoom,
                egui::Color32::BLACK,
            )));
        } else if self.active_tool == Tool::RasterBrush && self.brush_points.len() >= 2 {
            let points = self.brush_points.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
            painter.add(egui::Shape::line(points, egui::Stroke::new(
                self.raster_brush_width * zoom,
                egui::Color32::from_rgba_unmultiplied(self.raster_color[0], self.raster_color[1], self.raster_color[2], self.raster_color[3]),
            )));
        }

        if let (Some(start), Some(current)) = (*self.shape_start, *self.shape_current) {
            draw_shape_preview(&painter, &to_screen, self.active_tool, start, current);
        }

        if let (Some(start), Some(current)) =
            (*self.selection_rect_start, *self.selection_rect_current)
        {
            draw_selection_rect(&painter, to_screen(start), to_screen(current));
        }

        response
    }
}

fn nearest_control_point_in_active_layer(
    document: &Document,
    point: Vec2,
    radius: f32,
) -> Option<(usize, usize)> {
    let radius2 = radius * radius;
    document.active_layer().and_then(|layer| {
        layer
            .paths
            .iter()
            .enumerate()
            .flat_map(|(path_index, path)| {
                path.control_points
                    .iter()
                    .enumerate()
                    .filter_map(move |(point_index, cp)| {
                        let d2 = (*cp - point).length_squared();
                        if d2 <= radius2 {
                            Some((path_index, point_index, d2))
                        } else {
                            None
                        }
                    })
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(path_index, point_index, _)| (path_index, point_index))
    })
}


fn nearest_path_segment_in_active_layer(
    document: &Document,
    point: Vec2,
    radius: f32,
) -> Option<(usize, usize)> {
    let Some(layer) = document.active_layer() else {
        return None;
    };

    let mut best: Option<(usize, usize, f32)> = None;

    for (path_index, path) in layer.paths.iter().enumerate() {
        if path.control_points.len() < 2 || !path.validate() {
            continue;
        }

        let ppw = PPWCurve::convert(path);
        for (segment_index, segment) in ppw.segments.iter().enumerate() {
            for pair in segment.windows(2) {
                let distance = distance_point_to_segment(point, pair[0], pair[1]);
                if distance <= radius {
                    match best {
                        Some((_, _, best_distance)) if best_distance <= distance => {}
                        _ => best = Some((path_index, segment_index, distance)),
                    }
                }
            }
        }

        if ppw.segments.is_empty() && ppw.polygon.len() >= 2 {
            for (segment_index, pair) in ppw.polygon.windows(2).enumerate() {
                let distance = distance_point_to_segment(point, pair[0], pair[1]);
                if distance <= radius {
                    match best {
                        Some((_, _, best_distance)) if best_distance <= distance => {}
                        _ => best = Some((path_index, segment_index, distance)),
                    }
                }
            }
        }
    }

    best.map(|(path_index, segment_index, _)| (path_index, segment_index))
}

fn distance_point_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 <= f32::EPSILON {
        return (point - a).length();
    }
    let t = ((point - a).dot(ab) / len2).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (point - projection).length()
}


fn begin_edit(document: &Document, pending: &mut Option<Document>) {
    if pending.is_none() {
        *pending = Some(document.clone());
    }
}

fn commit_edit(
    pending: &mut Option<Document>,
    undo_stack: &mut Vec<Document>,
    redo_stack: &mut Vec<Document>,
) {
    if let Some(snapshot) = pending.take() {
        undo_stack.push(snapshot);
        if undo_stack.len() > 100 {
            undo_stack.remove(0);
        }
        redo_stack.clear();
    }
}

fn toggle_node(nodes: &mut Vec<PointSelection>, node: PointSelection) {
    if let Some(pos) = nodes.iter().position(|v| *v == node) {
        nodes.remove(pos);
    } else {
        nodes.push(node);
        nodes.sort_unstable();
    }
}

fn sync_legacy_selection(
    selected_path: usize,
    selected_points: &mut Vec<usize>,
    selected_nodes: &[PointSelection],
) {
    selected_points.clear();
    selected_points.extend(
        selected_nodes
            .iter()
            .filter(|node| node.path_index == selected_path)
            .map(|node| node.point_index),
    );
    selected_points.sort_unstable();
    selected_points.dedup();
}

fn points_in_rect_in_active_layer(document: &Document, a: Vec2, b: Vec2) -> Vec<PointSelection> {
    let Some(layer) = document.active_layer() else {
        return Vec::new();
    };

    let min_x = a.x.min(b.x);
    let max_x = a.x.max(b.x);
    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);

    layer
        .paths
        .iter()
        .enumerate()
        .flat_map(|(path_index, path)| {
            path.control_points
                .iter()
                .enumerate()
                .filter_map(move |(point_index, p)| {
                    if p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y {
                        Some(PointSelection { path_index, point_index })
                    } else {
                        None
                    }
                })
        })
        .collect()
}

fn move_selected_points_by_delta(
    document: &mut Document,
    selected_path: usize,
    selected_points: &[usize],
    selected_nodes: &[PointSelection],
    delta: Vec2,
) {
    let Some(layer) = document.active_layer_mut() else {
        return;
    };

    if selected_nodes.is_empty() {
        if let Some(path) = layer.paths.get_mut(selected_path) {
            path.move_points_by_delta(selected_points, delta);
        }
        return;
    }

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

fn toggle_index(indices: &mut Vec<usize>, index: usize) {
    if let Some(pos) = indices.iter().position(|v| *v == index) {
        indices.remove(pos);
    } else {
        indices.push(index);
        indices.sort_unstable();
    }
}

fn draw_document(
    painter: &egui::Painter,
    document: &Document,
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    show_points: bool,
    show_segments: bool,
    show_fill: bool,
    show_triangles: bool,
    active_layer_index: usize,
    selected_path: usize,
    selected_point: &Option<usize>,
    selected_points: &[usize],
    selected_nodes: &[PointSelection],
    selected_segment: &Option<SegmentSelection>,
    selected_image: &Option<usize>,
    transform_mode: bool,
    image_textures: &mut HashMap<u64, egui::TextureHandle>,
    ctx: &egui::Context,
    zoom: f32,
) {
    for (layer_index, layer) in document.layers.iter().enumerate() {
        if !layer.visible {
            continue;
        }

        if layer.kind == LayerKind::Raster {
            draw_raster_canvas(painter, layer, to_screen, image_textures, ctx);
        }

        draw_raster_images(
            painter,
            &layer.images,
            to_screen,
            image_textures,
            ctx,
            zoom,
            layer_index == active_layer_index,
            selected_image,
            transform_mode,
        );

        if layer.kind == LayerKind::Raster {
            draw_raster_strokes(painter, &layer.raster_strokes, to_screen, zoom);
            continue;
        }

        for (path_index, path) in layer.paths.iter().enumerate() {
            let is_active_path = layer_index == active_layer_index && path_index == selected_path;
            let node_points_for_path: Vec<usize> = if layer_index == active_layer_index && !selected_nodes.is_empty() {
                selected_nodes
                    .iter()
                    .filter(|node| node.path_index == path_index)
                    .map(|node| node.point_index)
                    .collect()
            } else if is_active_path {
                selected_points.to_vec()
            } else {
                Vec::new()
            };
            let active_point_for_path = if is_active_path { *selected_point } else { None };
            let selected_segment_for_path = if layer_index == active_layer_index {
                selected_segment
                    .as_ref()
                    .filter(|seg| seg.path_index == path_index)
                    .map(|seg| seg.segment_index)
            } else {
                None
            };
            draw_path(
                painter,
                path,
                to_screen,
                show_points,
                show_segments,
                show_fill,
                show_triangles,
                &active_point_for_path,
                &node_points_for_path,
                selected_segment_for_path,
                layer.locked,
                zoom,
                is_active_path,
                transform_mode,
            );
        }
    }
}




fn draw_raster_canvas(
    painter: &egui::Painter,
    layer: &crate::model::document::Layer,
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    image_textures: &mut HashMap<u64, egui::TextureHandle>,
    ctx: &egui::Context,
) {
    if layer.raster_width == 0 || layer.raster_height == 0 {
        return;
    }
    let expected = (layer.raster_width as usize).saturating_mul(layer.raster_height as usize).saturating_mul(4);
    if layer.raster_pixels_rgba.len() != expected {
        return;
    }
    let id = layer.name.as_ptr() as usize as u64 ^ ((layer.raster_width as u64) << 32) ^ layer.raster_height as u64;
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [layer.raster_width as usize, layer.raster_height as usize],
        &layer.raster_pixels_rgba,
    );
    let texture = image_textures.entry(id).or_insert_with(|| {
        ctx.load_texture(format!("raster_canvas_{}", id), color_image.clone(), egui::TextureOptions::NEAREST)
    });
    texture.set(color_image, egui::TextureOptions::NEAREST);

    let min = to_screen(Vec2::ZERO);
    let max = to_screen(Vec2::new(layer.raster_width as f32, layer.raster_height as f32));
    painter.image(
        texture.id(),
        egui::Rect::from_min_max(min, max),
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

fn draw_raster_images(
    painter: &egui::Painter,
    images: &[RasterImage],
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    image_textures: &mut HashMap<u64, egui::TextureHandle>,
    ctx: &egui::Context,
    zoom: f32,
    is_active_layer: bool,
    selected_image: &Option<usize>,
    transform_mode: bool,
) {
    for (image_index, image) in images.iter().enumerate() {
        if image.pixels_rgba.len() != (image.width as usize).saturating_mul(image.height as usize).saturating_mul(4) {
            continue;
        }

        let texture = image_textures.entry(image.id).or_insert_with(|| {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [image.width as usize, image.height as usize],
                &image.pixels_rgba,
            );
            ctx.load_texture(
                format!("raster_image_{}", image.id),
                color_image,
                egui::TextureOptions::LINEAR,
            )
        });

        let center = image.pos + image.size * 0.5;
        let corners = [
            Vec2::new(image.pos.x, image.pos.y),
            Vec2::new(image.pos.x + image.size.x, image.pos.y),
            Vec2::new(image.pos.x + image.size.x, image.pos.y + image.size.y),
            Vec2::new(image.pos.x, image.pos.y + image.size.y),
        ];
        let rotated = corners.map(|p| rotate_point(p, center, image.rotation));
        let screen = rotated.map(|p| to_screen(p));
        let mut mesh = egui::Mesh::with_texture(texture.id());
        mesh.vertices.push(egui::epaint::Vertex { pos: screen[0], uv: egui::pos2(0.0, 0.0), color: egui::Color32::WHITE });
        mesh.vertices.push(egui::epaint::Vertex { pos: screen[1], uv: egui::pos2(1.0, 0.0), color: egui::Color32::WHITE });
        mesh.vertices.push(egui::epaint::Vertex { pos: screen[2], uv: egui::pos2(1.0, 1.0), color: egui::Color32::WHITE });
        mesh.vertices.push(egui::epaint::Vertex { pos: screen[3], uv: egui::pos2(0.0, 1.0), color: egui::Color32::WHITE });
        mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
        painter.add(egui::Shape::mesh(mesh));

        if is_active_layer && *selected_image == Some(image_index) {
            for edge in 0..4 {
                painter.line_segment([screen[edge], screen[(edge + 1) % 4]], egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 170, 40)));
            }
            let handle_center = rotated[2];
            painter.rect_filled(
                egui::Rect::from_center_size(to_screen(handle_center), egui::vec2(10.0, 10.0)),
                0.0,
                egui::Color32::from_rgb(255, 170, 40),
            );
            if transform_mode {
                let right_mid = (rotated[1] + rotated[2]) * 0.5;
                let bottom_mid = (rotated[2] + rotated[3]) * 0.5;
                painter.text(to_screen(right_mid), egui::Align2::CENTER_CENTER, "⇔", egui::FontId::proportional(20.0), egui::Color32::from_rgb(255, 120, 20));
                painter.text(to_screen(bottom_mid), egui::Align2::CENTER_CENTER, "⇕", egui::FontId::proportional(20.0), egui::Color32::from_rgb(255, 120, 20));
                let top_mid = (rotated[0] + rotated[1]) * 0.5;
                let center = image.pos + image.size * 0.5;
                let dir = top_mid - center;
                let len = dir.length().max(1.0);
                let rotate_handle = top_mid + dir / len * 32.0;
                painter.line_segment([to_screen(top_mid), to_screen(rotate_handle)], egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 150, 255)));
                painter.circle_filled(to_screen(rotate_handle), 6.0, egui::Color32::from_rgb(80, 150, 255));
                painter.circle_stroke(to_screen(rotate_handle), 6.0, egui::Stroke::new(1.5, egui::Color32::DARK_GRAY));
            }
        }
    }
}

fn rotate_point(point: Vec2, center: Vec2, radians: f32) -> Vec2 {
    let (sin, cos) = radians.sin_cos();
    let x = point.x - center.x;
    let y = point.y - center.y;
    Vec2::new(center.x + x * cos - y * sin, center.y + x * sin + y * cos)
}

fn hit_image_in_active_layer(
    document: &Document,
    point: Vec2,
    handle_radius: f32,
) -> Option<(usize, bool)> {
    let layer = document.active_layer()?;
    let mut hit_body: Option<usize> = None;

    for (image_index, image) in layer.images.iter().enumerate().rev() {
        let min_x = image.pos.x.min(image.pos.x + image.size.x);
        let max_x = image.pos.x.max(image.pos.x + image.size.x);
        let min_y = image.pos.y.min(image.pos.y + image.size.y);
        let max_y = image.pos.y.max(image.pos.y + image.size.y);
        let handle = Vec2::new(max_x, max_y);

        if (point - handle).length() <= handle_radius {
            return Some((image_index, true));
        }

        if point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y {
            hit_body = Some(image_index);
        }
    }

    hit_body.map(|index| (index, false))
}

fn move_or_resize_selected_image(
    document: &mut Document,
    selected_image: Option<usize>,
    resizing: bool,
    delta: Vec2,
    keep_original_aspect: bool,
) {
    let Some(image_index) = selected_image else {
        return;
    };
    let Some(layer) = document.active_layer_mut() else {
        return;
    };
    let Some(image) = layer.images.get_mut(image_index) else {
        return;
    };

    if resizing {
        if keep_original_aspect {
            // Shift + drag keeps the imported image's original aspect ratio.
            // The bottom-right resize handle is still used, so the top-left position remains fixed.
            let original_w = image.width.max(1) as f32;
            let original_h = image.height.max(1) as f32;
            let aspect = (original_w / original_h).max(0.0001);

            let requested_w = (image.size.x + delta.x).max(4.0);
            let requested_h = (image.size.y + delta.y).max(4.0);

            // Choose the dimension that changed more during this drag step, then derive the other
            // dimension from the original image ratio. This feels natural for horizontal or vertical drags.
            if delta.x.abs() >= delta.y.abs() {
                image.size.x = requested_w;
                image.size.y = (requested_w / aspect).max(4.0);
            } else {
                image.size.y = requested_h;
                image.size.x = (requested_h * aspect).max(4.0);
            }
        } else {
            image.size.x = (image.size.x + delta.x).max(4.0);
            image.size.y = (image.size.y + delta.y).max(4.0);
        }
    } else {
        image.pos += delta;
    }
}


fn draw_raster_strokes(
    painter: &egui::Painter,
    strokes: &[RasterStroke],
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    zoom: f32,
) {
    for stroke in strokes {
        if stroke.points.len() < 2 {
            continue;
        }
        let points = stroke.points.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
        let color = egui::Color32::from_rgba_unmultiplied(
            stroke.color[0],
            stroke.color[1],
            stroke.color[2],
            stroke.color[3],
        );
        painter.add(egui::Shape::line(points, egui::Stroke::new(stroke.width * zoom, color)));
    }
}

fn paint_raster_line(document: &mut Document, a: Vec2, b: Vec2, radius: f32, color: [u8; 4]) {
    let width = document.canvas_width.max(1.0).round() as u32;
    let height = document.canvas_height.max(1.0).round() as u32;
    let Some(layer) = document.active_layer_mut() else { return; };
    if layer.kind != LayerKind::Raster { return; }
    ensure_layer_raster_canvas(layer, width, height);

    let delta = b - a;
    let len = delta.length();
    let step = radius.max(0.5) * 0.35;
    let count = (len / step.max(0.25)).ceil().max(1.0) as usize;
    for i in 0..=count {
        let t = i as f32 / count as f32;
        stamp_raster(layer, a + delta * t, radius, color);
    }
}

fn erase_raster_line(document: &mut Document, a: Vec2, b: Vec2, radius: f32) {
    paint_raster_line(document, a, b, radius, [255, 255, 255, 0]);
}

fn stamp_raster(layer: &mut crate::model::document::Layer, center: Vec2, radius: f32, color: [u8; 4]) {
    let radius = radius.max(0.5);
    let min_x = (center.x - radius).floor().max(0.0) as i32;
    let max_x = (center.x + radius).ceil().min(layer.raster_width.saturating_sub(1) as f32) as i32;
    let min_y = (center.y - radius).floor().max(0.0) as i32;
    let max_y = (center.y + radius).ceil().min(layer.raster_height.saturating_sub(1) as f32) as i32;
    let r2 = radius * radius;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;
            if dx * dx + dy * dy <= r2 {
                let idx = ((y as u32 * layer.raster_width + x as u32) as usize) * 4;
                if idx + 3 < layer.raster_pixels_rgba.len() {
                    layer.raster_pixels_rgba[idx..idx + 4].copy_from_slice(&color);
                }
            }
        }
    }
}


fn sample_polyline(points: &[Vec2], step: f32) -> Vec<Vec2> {
    if points.len() < 2 {
        return points.to_vec();
    }

    let step = step.max(0.25);
    let mut sampled = Vec::new();
    sampled.push(points[0]);

    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let delta = b - a;
        let distance = delta.length();
        if distance <= f32::EPSILON {
            continue;
        }

        let count = (distance / step).ceil() as usize;
        for i in 1..=count {
            let t = i as f32 / count as f32;
            sampled.push(a + delta * t);
        }
    }

    sampled
}

fn draw_path(
    painter: &egui::Painter,
    path: &crate::ppw::path::PPWPath,
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    show_points: bool,
    show_segments: bool,
    show_fill: bool,
    show_triangles: bool,
    selected_point: &Option<usize>,
    selected_points: &[usize],
    selected_segment: Option<usize>,
    locked: bool,
    zoom: f32,
    is_active_path: bool,
    transform_mode: bool,
) {
    if path.control_points.len() >= 2 && path.validate() {
        let ppw = PPWCurve::convert(path);

        if show_fill && path.is_closed && path.fill_enabled && ppw.polygon.len() >= 3 {
            draw_filled_polygon(painter, &ppw.polygon, to_screen, path.fill_color);
        }

        if show_triangles && path.is_closed && ppw.polygon.len() >= 3 {
            draw_triangulation(painter, &ppw.polygon, to_screen);
        }

        if show_segments {
            for (i, segment) in ppw.segments.iter().enumerate() {
                let points = segment.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
                let color = match i % 4 {
                    0 => egui::Color32::from_rgb(40, 80, 200),
                    1 => egui::Color32::from_rgb(190, 60, 60),
                    2 => egui::Color32::from_rgb(40, 150, 80),
                    _ => egui::Color32::from_rgb(160, 90, 190),
                };
                painter.add(egui::Shape::line(points, egui::Stroke::new(path.stroke_width * zoom, color)));
            }
        } else {
            let points = ppw.polygon.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
            let stroke_color = egui::Color32::from_rgba_unmultiplied(
                path.stroke_color[0],
                path.stroke_color[1],
                path.stroke_color[2],
                path.stroke_color[3],
            );
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(path.stroke_width * zoom, stroke_color),
            ));
        }

        if let Some(segment_index) = selected_segment {
            if let Some(segment) = ppw.segments.get(segment_index) {
                let points = segment.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
                if points.len() >= 2 {
                    painter.add(egui::Shape::line(
                        points,
                        egui::Stroke::new((path.stroke_width * zoom + 4.0).max(4.0), egui::Color32::from_rgb(255, 170, 30)),
                    ));
                }
            }
        }
    }

    if is_active_path {
        if let Some((min, max)) = path.bounds() {
            let center = Vec2::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5);
            let corner = to_screen(max);
            let right_mid = to_screen(Vec2::new(max.x, center.y));
            let bottom_mid = to_screen(Vec2::new(center.x, max.y));
            painter.rect_filled(
                egui::Rect::from_center_size(corner, egui::vec2(10.0, 10.0)),
                0.0,
                egui::Color32::from_rgb(255, 170, 40),
            );
            if transform_mode {
                painter.text(right_mid, egui::Align2::CENTER_CENTER, "⇔", egui::FontId::proportional(20.0), egui::Color32::from_rgb(255, 120, 20));
                painter.text(bottom_mid, egui::Align2::CENTER_CENTER, "⇕", egui::FontId::proportional(20.0), egui::Color32::from_rgb(255, 120, 20));
                let top_mid = Vec2::new(center.x, min.y);
                let rotate_handle = Vec2::new(center.x, min.y - 32.0 / zoom.max(0.1));
                painter.line_segment([to_screen(top_mid), to_screen(rotate_handle)], egui::Stroke::new(1.5, egui::Color32::from_rgb(80, 150, 255)));
                painter.circle_filled(to_screen(rotate_handle), 6.0, egui::Color32::from_rgb(80, 150, 255));
                painter.circle_stroke(to_screen(rotate_handle), 6.0, egui::Stroke::new(1.5, egui::Color32::DARK_GRAY));
            }
        }
    }

    if show_points {
        for pair in path.control_points.windows(2) {
            painter.line_segment(
                [to_screen(pair[0]), to_screen(pair[1])],
                egui::Stroke::new(1.0, egui::Color32::from_gray(170)),
            );
        }

        if path.is_closed && path.control_points.len() >= 2 {
            painter.line_segment(
                [
                    to_screen(*path.control_points.last().unwrap()),
                    to_screen(path.control_points[0]),
                ],
                egui::Stroke::new(1.0, egui::Color32::from_gray(170)),
            );
        }

        for (i, cp) in path.control_points.iter().enumerate() {
            let pos = to_screen(*cp);
            let selected = selected_points.contains(&i);
            let active = *selected_point == Some(i);
            let radius = if active { 8.0 } else if selected { 7.0 } else { 5.0 };
            let fill = if locked {
                egui::Color32::from_gray(190)
            } else if active {
                egui::Color32::from_rgb(255, 190, 60)
            } else if selected {
                egui::Color32::from_rgb(255, 230, 100)
            } else {
                egui::Color32::WHITE
            };

            painter.circle_filled(pos, radius, fill);
            painter.circle_stroke(pos, radius, egui::Stroke::new(2.0, egui::Color32::DARK_GRAY));
            painter.text(
                pos + egui::vec2(8.0, -8.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{i}"),
                egui::FontId::monospace(12.0),
                egui::Color32::DARK_GRAY,
            );
        }
    }
}

fn draw_filled_polygon(
    painter: &egui::Painter,
    polygon: &[Vec2],
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    fill_color: [u8; 4],
) {
    let triangulation = ShapeUtil::triangulate_polygon(polygon);
    if triangulation.vertices.len() < 3 || triangulation.indices.len() < 3 {
        return;
    }

    let color = egui::Color32::from_rgba_unmultiplied(
        fill_color[0],
        fill_color[1],
        fill_color[2],
        fill_color[3],
    );

    let mut mesh = egui::Mesh::default();
    for vertex in &triangulation.vertices {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: to_screen(*vertex),
            uv: egui::epaint::WHITE_UV,
            color,
        });
    }
    mesh.indices = triangulation.indices.iter().map(|&i| i as u32).collect();
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_triangulation(
    painter: &egui::Painter,
    polygon: &[Vec2],
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
) {
    let triangulation = ShapeUtil::triangulate_polygon(polygon);
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(120, 120, 180));

    for tri in triangulation.indices.chunks(3) {
        if tri.len() != 3 {
            continue;
        }
        let a = to_screen(triangulation.vertices[tri[0]]);
        let b = to_screen(triangulation.vertices[tri[1]]);
        let c = to_screen(triangulation.vertices[tri[2]]);
        painter.line_segment([a, b], stroke);
        painter.line_segment([b, c], stroke);
        painter.line_segment([c, a], stroke);
    }
}

fn draw_shape_preview(
    painter: &egui::Painter,
    to_screen: &dyn Fn(Vec2) -> egui::Pos2,
    tool: Tool,
    start: Vec2,
    current: Vec2,
) {
    let paths = match tool {
        Tool::Rectangle => PPWPath::from_rectangle_diagonal_paths(start, current, 2.0),
        Tool::Ellipse => vec![PPWPath::from_ellipse(start, current, 2.0)],
        _ => return,
    };

    for path in paths {
        let ppw = PPWCurve::convert(&path);
        let points = ppw.polygon.iter().map(|p| to_screen(*p)).collect::<Vec<_>>();
        if points.len() >= 2 {
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(40, 120, 220, 190)),
            ));
        }
    }
}

fn square_constrained_point(start: Vec2, current: Vec2) -> Vec2 {
    let delta = current - start;
    let size = delta.x.abs().max(delta.y.abs());
    Vec2::new(start.x + size * delta.x.signum(), start.y + size * delta.y.signum())
}

fn draw_selection_rect(painter: &egui::Painter, a: egui::Pos2, b: egui::Pos2) {
    let rect = egui::Rect::from_two_pos(a, b);
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(80, 140, 255, 28),
    );
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, egui::Color32::from_rgb(40, 100, 220)),
        egui::StrokeKind::Inside,
    );
}

fn draw_grid(painter: &egui::Painter, rect: egui::Rect, zoom: f32, pan: Vec2) {
    let spacing = 25.0 * zoom.max(0.1);
    let stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(225));

    let mut x = rect.left() + pan.x.rem_euclid(spacing);
    while x <= rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            stroke,
        );
        x += spacing;
    }

    let mut y = rect.top() + pan.y.rem_euclid(spacing);
    while y <= rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            stroke,
        );
        y += spacing;
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformHandle {
    UniformResize,
    ScaleX,
    ScaleY,
    Rotate,
}

impl TransformHandle {
    fn to_drag(self) -> TransformDragHandle {
        match self {
            TransformHandle::UniformResize => TransformDragHandle::UniformResize,
            TransformHandle::ScaleX => TransformDragHandle::ScaleX,
            TransformHandle::ScaleY => TransformDragHandle::ScaleY,
            TransformHandle::Rotate => TransformDragHandle::Rotate,
        }
    }
}

fn selected_transform_center(document: &Document, selected_path: usize, selected_image: Option<usize>) -> Option<Vec2> {
    if let Some(image_index) = selected_image {
        let image = document.active_layer()?.images.get(image_index)?;
        return Some(image.pos + image.size * 0.5);
    }
    let path = document.active_layer()?.paths.get(selected_path)?;
    let (min, max) = path.bounds()?;
    Some(Vec2::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5))
}

fn hit_transform_handle(document: &Document, selected_path: usize, selected_image: Option<usize>, point: Vec2, radius: f32) -> Option<TransformHandle> {
    if let Some(image_index) = selected_image {
        let image = document.active_layer()?.images.get(image_index)?;
        let center = image.pos + image.size * 0.5;
        let corners = [
            Vec2::new(image.pos.x, image.pos.y),
            Vec2::new(image.pos.x + image.size.x, image.pos.y),
            Vec2::new(image.pos.x + image.size.x, image.pos.y + image.size.y),
            Vec2::new(image.pos.x, image.pos.y + image.size.y),
        ];
        let rotated = corners.map(|p| rotate_point(p, center, image.rotation));
        let right_mid = (rotated[1] + rotated[2]) * 0.5;
        let bottom_mid = (rotated[2] + rotated[3]) * 0.5;
        if (point - rotated[2]).length() <= radius { return Some(TransformHandle::UniformResize); }
        if (point - right_mid).length() <= radius { return Some(TransformHandle::ScaleX); }
        if (point - bottom_mid).length() <= radius { return Some(TransformHandle::ScaleY); }
        let top_mid = (rotated[0] + rotated[1]) * 0.5;
        let dir = top_mid - center;
        let len = dir.length().max(1.0);
        let rotate_handle = top_mid + dir / len * 32.0;
        if (point - rotate_handle).length() <= radius { return Some(TransformHandle::Rotate); }
        return None;
    }

    let path = document.active_layer()?.paths.get(selected_path)?;
    let (min, max) = path.bounds()?;
    let center = Vec2::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5);
    let right_mid = Vec2::new(max.x, center.y);
    let bottom_mid = Vec2::new(center.x, max.y);
    if (point - max).length() <= radius { return Some(TransformHandle::UniformResize); }
    if (point - right_mid).length() <= radius { return Some(TransformHandle::ScaleX); }
    if (point - bottom_mid).length() <= radius { return Some(TransformHandle::ScaleY); }
    let rotate_handle = Vec2::new(center.x, min.y - 32.0);
    if (point - rotate_handle).length() <= radius { return Some(TransformHandle::Rotate); }
    None
}

fn rotate_selected_transform(document: &mut Document, selected_path: usize, selected_image: Option<usize>, center: Vec2, radians: f32) {
    if let Some(image_index) = selected_image {
        if let Some(image) = document.active_layer_mut().and_then(|layer| layer.images.get_mut(image_index)) {
            image.rotation += radians;
        }
        return;
    }
    if let Some(path) = document.active_path_mut(selected_path) {
        path.rotate_about(center, radians);
    }
}

fn scale_selected_transform(document: &mut Document, selected_path: usize, selected_image: Option<usize>, sx: f32, sy: f32) {
    if let Some(image_index) = selected_image {
        if let Some(image) = document.active_layer_mut().and_then(|layer| layer.images.get_mut(image_index)) {
            let center = image.pos + image.size * 0.5;
            image.size.x = (image.size.x * sx).max(4.0);
            image.size.y = (image.size.y * sy).max(4.0);
            image.pos = center - image.size * 0.5;
        }
        return;
    }
    if let Some(path) = document.active_path_mut(selected_path) {
        path.scale_about_center(sx, sy);
    }
}

fn hit_selected_path_resize_handle(
    document: &Document,
    selected_path: usize,
    point: Vec2,
    handle_radius: f32,
) -> Option<usize> {
    let layer = document.active_layer()?;
    let path = layer.paths.get(selected_path)?;
    let (_min, max) = path.bounds()?;
    if (point - max).length() <= handle_radius {
        Some(selected_path)
    } else {
        None
    }
}


fn resize_selected_transform_from_drag(
    document: &mut Document,
    selected_path: usize,
    selected_image: Option<usize>,
    handle: TransformDragHandle,
    delta: Vec2,
) {
    if let Some(image_index) = selected_image {
        if let Some(image) = document.active_layer_mut().and_then(|layer| layer.images.get_mut(image_index)) {
            match handle {
                TransformDragHandle::UniformResize => {
                    let original_w = image.width.max(1) as f32;
                    let original_h = image.height.max(1) as f32;
                    let aspect = (original_w / original_h).max(0.0001);
                    if delta.x.abs() >= delta.y.abs() {
                        image.size.x = (image.size.x + delta.x).max(4.0);
                        image.size.y = (image.size.x / aspect).max(4.0);
                    } else {
                        image.size.y = (image.size.y + delta.y).max(4.0);
                        image.size.x = (image.size.y * aspect).max(4.0);
                    }
                }
                TransformDragHandle::ScaleX => {
                    image.size.x = (image.size.x + delta.x).max(4.0);
                }
                TransformDragHandle::ScaleY => {
                    image.size.y = (image.size.y + delta.y).max(4.0);
                }
                TransformDragHandle::Rotate => {}
            }
        }
        return;
    }

    let Some(path) = document.active_path_mut(selected_path) else { return; };
    let Some((min, max)) = path.bounds() else { return; };
    let w = (max.x - min.x).abs().max(1.0);
    let h = (max.y - min.y).abs().max(1.0);

    match handle {
        TransformDragHandle::UniformResize => {
            let sx_from_x = ((w + delta.x) / w).max(0.01);
            let sy_from_y = ((h + delta.y) / h).max(0.01);
            let factor = if delta.x.abs() >= delta.y.abs() { sx_from_x } else { sy_from_y };
            path.scale_about_anchor(min, factor, factor);
        }
        TransformDragHandle::ScaleX => {
            let sx = ((w + delta.x) / w).max(0.01);
            path.scale_about_anchor(min, sx, 1.0);
        }
        TransformDragHandle::ScaleY => {
            let sy = ((h + delta.y) / h).max(0.01);
            path.scale_about_anchor(min, 1.0, sy);
        }
        TransformDragHandle::Rotate => {}
    }
}
