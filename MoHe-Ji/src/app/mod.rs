pub mod update;
pub mod view;

use crate::model::document::Document;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::ppw::{path::PPWPath, Vec2};


#[derive(Debug, Clone)]
pub struct CopiedPathPoints {
    pub path: PPWPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    VectorBrush,
    AddPoint,
    Rectangle,
    Ellipse,
    RasterBrush,
    RasterEraser,
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointSelection {
    pub path_index: usize,
    pub point_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentSelection {
    pub path_index: usize,
    pub segment_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformDragHandle {
    UniformResize,
    ScaleX,
    ScaleY,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarTab {
    Tools,
    File,
    Edit,
    View,
    Path,
    Layer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamWheelTarget {
    Weight,
    PsiPrev,
    PsiNext,
    PhiPrev,
    PhiNext,
    StrokeWidth,
    FillHue,
    FillBrightness,
    FillAlpha,
    PaintHue,
    PaintBrightness,
    PaintAlpha,
}

impl ParamWheelTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Weight => "Weight",
            Self::PsiPrev => "Psi Prev",
            Self::PsiNext => "Psi Next",
            Self::PhiPrev => "Phi Prev",
            Self::PhiNext => "Phi Next",
            Self::StrokeWidth => "Stroke Width",
            Self::FillHue => "Fill Red",
            Self::FillBrightness => "Fill Green",
            Self::FillAlpha => "Fill Blue",
            Self::PaintHue => "Stroke/Raster Red",
            Self::PaintBrightness => "Stroke/Raster Green",
            Self::PaintAlpha => "Stroke/Raster Blue",
        }
    }
}

pub struct VectorEditorApp {
    pub document: Document,
    pub show_points: bool,
    pub show_segments: bool,
    pub show_fill: bool,
    pub show_triangles: bool,
    pub active_tool: Tool,
    pub selected_path: usize,

    pub selected_point: Option<usize>,
    pub selected_points: Vec<usize>,
    pub selected_nodes: Vec<PointSelection>,
    pub selected_segment: Option<SegmentSelection>,
    pub dragging_point: Option<usize>,
    pub drag_last_pos: Option<Vec2>,
    pub selection_rect_start: Option<Vec2>,
    pub selection_rect_current: Option<Vec2>,
    pub brush_points: Vec<Vec2>,
    pub brush_width: f32,
    pub simplify_tolerance: f32,
    pub simplify_path_index: Option<usize>,
    pub path_scale_x: f32,
    pub path_scale_y: f32,
    pub raster_brush_width: f32,
    pub raster_eraser_width: f32,
    pub raster_color: [u8; 4],
    pub pan_offset: Vec2,
    pub shape_start: Option<Vec2>,
    pub shape_current: Option<Vec2>,

    pub undo_stack: Vec<Document>,
    pub redo_stack: Vec<Document>,
    pub pending_drag_snapshot: Option<Document>,
    pub last_timed_undo_at: Option<Instant>,

    pub svg_file_path: String,
    pub io_status: String,
    pub active_toolbar_tab: ToolbarTab,
    pub zoom: f32,
    pub layer_drag_source: Option<usize>,
    pub param_wheel_target: Option<ParamWheelTarget>,
    pub dark_mode: bool,
    pub png_file_path: String,
    pub png_file_name: String,
    pub png_transparent_empty: bool,
    pub png_quality_scale: f32,
    pub copied_point_paths: Vec<CopiedPathPoints>,
    pub selected_image: Option<usize>,
    pub resizing_image: bool,
    pub resizing_path: bool,
    pub transform_mode: bool,
    pub rotating_selection: bool,
    pub active_transform_handle: Option<TransformDragHandle>,
    pub transform_anchor: Option<Vec2>,
    pub image_textures: HashMap<u64, egui::TextureHandle>,
    pub side_panel_open: bool,
}

impl VectorEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            document: Document::empty_path(),
            show_points: true,
            show_segments: false,
            show_fill: true,
            show_triangles: false,
            active_tool: Tool::Select,
            selected_path: 0,
            selected_point: None,
            selected_points: Vec::new(),
            selected_nodes: Vec::new(),
            selected_segment: None,
            dragging_point: None,
            drag_last_pos: None,
            selection_rect_start: None,
            selection_rect_current: None,
            brush_points: Vec::new(),
            brush_width: 4.0,
            simplify_tolerance: 8.0,
            simplify_path_index: None,
            path_scale_x: 1.0,
            path_scale_y: 1.0,
            raster_brush_width: 8.0,
            raster_eraser_width: 24.0,
            raster_color: [0, 0, 0, 180],
            pan_offset: Vec2::ZERO,
            shape_start: None,
            shape_current: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            pending_drag_snapshot: None,
            last_timed_undo_at: None,
            svg_file_path: "ppw_document.svg".to_string(),
            io_status: "New empty path. SVG file: ppw_document.svg".to_string(),
            active_toolbar_tab: ToolbarTab::Tools,
            zoom: 1.0,
            layer_drag_source: None,
            param_wheel_target: None,
            dark_mode: false,
            png_file_path: ".".to_string(),
            png_file_name: "canvas_export.png".to_string(),
            png_transparent_empty: true,
            png_quality_scale: 1.0,
            copied_point_paths: Vec::new(),
            selected_image: None,
            resizing_image: false,
            resizing_path: false,
            transform_mode: false,
            rotating_selection: false,
            active_transform_handle: None,
            transform_anchor: None,
            image_textures: HashMap::new(),
            side_panel_open: true,
        }
    }

    pub fn clear_selection_state(&mut self) {
        self.selected_point = None;
        self.selected_points.clear();
        self.selected_nodes.clear();
        self.selected_segment = None;
        self.dragging_point = None;
        self.drag_last_pos = None;
        self.selection_rect_start = None;
        self.selection_rect_current = None;
        self.brush_points.clear();
        self.shape_start = None;
        self.shape_current = None;
        self.selected_image = None;
        self.resizing_image = false;
        self.resizing_path = false;
        self.transform_mode = false;
        self.rotating_selection = false;
        self.active_transform_handle = None;
        self.transform_anchor = None;
    }

    pub fn push_undo_snapshot(&mut self) {
        self.push_undo_document(self.document.clone());
        self.last_timed_undo_at = None;
    }

    pub fn push_undo_document(&mut self, snapshot: Document) {
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn push_timed_undo_snapshot(&mut self, snapshot: Document) {
        let now = Instant::now();
        let should_save = self
            .last_timed_undo_at
            .map(|last| now.duration_since(last) >= Duration::from_secs(1))
            .unwrap_or(true);

        if should_save {
            self.push_undo_document(snapshot);
            self.last_timed_undo_at = Some(now);
        }
    }

    pub fn undo(&mut self) {
        if let Some(previous) = self.undo_stack.pop() {
            let current = std::mem::replace(&mut self.document, previous);
            self.redo_stack.push(current);
            self.document.normalize();
            self.image_textures.clear();
            self.clear_selection_state();
            self.selected_path = 0;
            self.last_timed_undo_at = None;
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let current = std::mem::replace(&mut self.document, next);
            self.undo_stack.push(current);
            self.document.normalize();
            self.image_textures.clear();
            self.clear_selection_state();
            self.selected_path = 0;
            self.last_timed_undo_at = None;
        }
    }

    pub fn begin_canvas_edit(&mut self) {
        if self.pending_drag_snapshot.is_none() {
            self.pending_drag_snapshot = Some(self.document.clone());
        }
    }

    pub fn commit_canvas_edit(&mut self) {
        if let Some(snapshot) = self.pending_drag_snapshot.take() {
            self.undo_stack.push(snapshot);
            if self.undo_stack.len() > 100 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
            self.last_timed_undo_at = None;
        }
    }

    pub fn cancel_canvas_edit(&mut self) {
        self.pending_drag_snapshot = None;
    }

    fn normalized_png_file_name(&self) -> String {
        let trimmed = self.png_file_name.trim();
        let base = if trimmed.is_empty() {
            "canvas_export"
        } else {
            trimmed
        };
        if base.to_ascii_lowercase().ends_with(".png") {
            base.to_string()
        } else {
            format!("{base}.png")
        }
    }

    fn resolved_png_path(&self) -> std::path::PathBuf {
        let name = self.normalized_png_file_name();
        let base = std::path::PathBuf::from(self.png_file_path.trim());
        if base.extension().is_some() {
            base
        } else {
            base.join(name)
        }
    }

    pub fn save_png(&mut self) {
        let path = self.resolved_png_path();
        let display_path = path.to_string_lossy().to_string();
        match crate::io::png_export::save_png(
            &self.document,
            &display_path,
            self.png_transparent_empty,
            self.png_quality_scale,
        ) {
            Ok(()) => {
                self.io_status = format!("Saved PNG: {display_path}");
            }
            Err(err) => {
                self.io_status = format!("PNG save failed: {err}");
            }
        }
    }

    pub fn choose_png_path_and_save(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(self.normalized_png_file_name())
            .save_file()
        {
            if let Some(parent) = path.parent() {
                self.png_file_path = parent.to_string_lossy().to_string();
            }
            if let Some(file_name) = path.file_name() {
                self.png_file_name = file_name.to_string_lossy().to_string();
            }
            self.save_png();
        }
    }

    pub fn choose_png_folder(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.png_file_path = path.to_string_lossy().to_string();
        }
    }


    pub fn add_image_from_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return;
        };

        let display = path.to_string_lossy().to_string();
        let file_name = path
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_else(|| "Image".to_string());

        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                if width == 0 || height == 0 {
                    self.io_status = format!("Image load failed: empty image {display}");
                    return;
                }

                self.push_undo_snapshot();
                let pixels = rgba.into_raw();
                match self.document.add_image_to_active_layer(file_name, pixels, width, height) {
                    Some((image_index, _id)) => {
                        self.clear_selection_state();
                        self.selected_image = Some(image_index);
                        self.active_tool = Tool::Select;
                        self.active_toolbar_tab = ToolbarTab::Tools;
                        self.io_status = format!("Added image: {display}");
                    }
                    None => {
                        self.undo();
                        self.io_status = "Image add failed: choose an unlocked layer".to_string();
                    }
                }
            }
            Err(err) => {
                self.io_status = format!("Image load failed: {err}");
            }
        }
    }


    pub fn finish_current_path(&mut self) {
        let is_vector = self
            .document
            .active_layer()
            .map(|layer| layer.kind == crate::model::document::LayerKind::Vector && !layer.locked)
            .unwrap_or(false);
        if !is_vector {
            self.io_status = "Finish Path failed: choose an unlocked vector layer".to_string();
            return;
        }

        self.push_undo_snapshot();
        if let Some(layer) = self.document.active_layer_mut() {
            let needs_new = layer
                .paths
                .last()
                .map(|path| !path.control_points.is_empty())
                .unwrap_or(true);
            if needs_new {
                layer.paths.push(PPWPath::empty());
            }
            self.selected_path = layer.paths.len().saturating_sub(1);
            self.clear_selection_state();
            self.active_tool = Tool::AddPoint;
            self.active_toolbar_tab = ToolbarTab::Tools;
            self.io_status = "Finished current path. Started a new empty path on the same layer.".to_string();
        }
    }


    pub fn scale_selected_path(&mut self, scale_x: f32, scale_y: f32) {
        if self.active_tool != Tool::Select {
            self.io_status = "Scale path: switch to Select tool first".to_string();
            return;
        }

        let Some(layer) = self.document.active_layer() else {
            self.io_status = "Scale path failed: no active layer".to_string();
            return;
        };
        if layer.kind != crate::model::document::LayerKind::Vector || layer.locked {
            self.io_status = "Scale path failed: choose an unlocked vector layer".to_string();
            return;
        }
        if layer.paths.get(self.selected_path).is_none() {
            self.io_status = "Scale path failed: no selected path".to_string();
            return;
        }

        self.push_undo_snapshot();
        if let Some(path) = self.document.active_path_mut(self.selected_path) {
            path.scale_about_center(scale_x.max(0.001), scale_y.max(0.001));
            self.selected_nodes.clear();
            for point_index in 0..path.control_points.len() {
                self.selected_nodes.push(PointSelection { path_index: self.selected_path, point_index });
            }
            self.selected_points = self.selected_nodes
                .iter()
                .filter(|node| node.path_index == self.selected_path)
                .map(|node| node.point_index)
                .collect();
            self.selected_point = self.selected_points.last().copied();
            self.io_status = format!("Scaled selected path: x {:.3}, y {:.3}", scale_x, scale_y);
        }
    }


    pub fn simplify_selected_path_with_shortcut(&mut self) {
        if self.simplify_path_index != Some(self.selected_path) {
            self.simplify_tolerance = 8.0;
            self.simplify_path_index = Some(self.selected_path);
        }
        let used = self.simplify_tolerance;
        self.simplify_selected_path();
        self.simplify_tolerance = (used + 5.0).min(200.0);
    }
    pub fn enable_transform_mode(&mut self) {
        if self.active_tool != Tool::Select {
            self.active_tool = Tool::Select;
        }
        if self.selected_image.is_some() || self.document.active_layer().and_then(|layer| layer.paths.get(self.selected_path)).is_some() {
            self.transform_mode = true;
            self.show_points = true;
            self.io_status = "Transform mode: drag orange corner = proportional resize, orange arrows = horizontal/vertical resize, blue circle = rotate.".to_string();
        } else {
            self.io_status = "Transform mode failed: select a path or image first".to_string();
        }
    }


    pub fn rotate_selected_object(&mut self, radians: f32) {
        if self.active_tool != Tool::Select {
            self.io_status = "Rotate: switch to Select tool first".to_string();
            return;
        }
        self.push_undo_snapshot();
        if let Some(image_index) = self.selected_image {
            if let Some(layer) = self.document.active_layer_mut() {
                if let Some(image) = layer.images.get_mut(image_index) {
                    image.rotation += radians;
                    self.io_status = format!("Rotated selected image: {:.1} deg", image.rotation.to_degrees());
                    return;
                }
            }
        }
        if let Some(path) = self.document.active_path_mut(self.selected_path) {
            path.rotate_about_center(radians);
            self.io_status = format!("Rotated selected path: {:.1} deg", radians.to_degrees());
        } else {
            let _ = self.undo_stack.pop();
            self.io_status = "Rotate failed: no selected path or image".to_string();
        }
    }

    pub fn simplify_selected_path(&mut self) {
        let tolerance = self.simplify_tolerance.max(0.0);
        let Some(layer) = self.document.active_layer() else {
            self.io_status = "Simplify failed: no active layer".to_string();
            return;
        };
        if layer.kind != crate::model::document::LayerKind::Vector || layer.locked {
            self.io_status = "Simplify failed: choose an unlocked vector layer".to_string();
            return;
        }
        let before_count = layer
            .paths
            .get(self.selected_path)
            .map(|path| path.control_points.len())
            .unwrap_or(0);
        if before_count < 3 {
            self.io_status = "Simplify skipped: path has too few points".to_string();
            return;
        }

        self.push_undo_snapshot();
        if let Some(path) = self.document.active_path_mut(self.selected_path) {
            path.simplify_control_points(tolerance);
            let after_count = path.control_points.len();
            self.clear_selection_state();
            self.io_status = format!(
                "Simplified path: {} -> {} points",
                before_count,
                after_count
            );
        }
    }

    pub fn save_svg(&mut self) {
        match crate::io::svg::save_svg(&self.document, &self.svg_file_path) {
            Ok(()) => {
                self.io_status = format!("Saved SVG: {}", self.svg_file_path);
            }
            Err(err) => {
                self.io_status = format!("Save failed: {err}");
            }
        }
    }

    pub fn load_svg(&mut self) {
        let path = self.svg_file_path.trim().to_string();
        if path.is_empty() {
            self.io_status = "Load failed: SVG file path is empty".to_string();
            return;
        }
        self.load_svg_from_path(path);
    }

    pub fn choose_svg_file_and_load(&mut self) {
        let mut dialog = rfd::FileDialog::new().add_filter("SVG file", &["svg"]);
        let current_path = std::path::PathBuf::from(self.svg_file_path.trim());
        if let Some(parent) = current_path.parent().filter(|p| p.exists()) {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.pick_file() else {
            return;
        };

        let display_path = path.to_string_lossy().to_string();
        self.svg_file_path = display_path.clone();
        self.load_svg_from_path(display_path);
    }

    fn load_svg_from_path(&mut self, file_path: String) {
        match crate::io::svg::load_svg(&file_path) {
            Ok(mut document) => {
                self.push_undo_snapshot();
                document.normalize();
                self.document = document;
                self.image_textures.clear();
                self.clear_selection_state();
                self.selected_path = 0;
                self.io_status = format!("Loaded SVG: {file_path}");
            }
            Err(err) => {
                self.io_status = format!("Load failed: {err}");
            }
        }
    }

    pub fn copy_selected_points(&mut self) {
        self.copied_point_paths.clear();
        let Some(layer) = self.document.active_layer() else {
            self.io_status = "Copy failed: no active layer".to_string();
            return;
        };
        if layer.kind != crate::model::document::LayerKind::Vector {
            self.io_status = "Copy skipped: active layer is not vector".to_string();
            return;
        }

        let mut groups: Vec<(usize, Vec<usize>)> = Vec::new();
        let source_nodes: Vec<_> = if self.selected_nodes.is_empty() {
            self.selected_points
                .iter()
                .copied()
                .map(|point_index| crate::app::PointSelection { path_index: self.selected_path, point_index })
                .collect()
        } else {
            self.selected_nodes.clone()
        };

        for node in source_nodes {
            if let Some((_, points)) = groups.iter_mut().find(|(path, _)| *path == node.path_index) {
                points.push(node.point_index);
            } else {
                groups.push((node.path_index, vec![node.point_index]));
            }
        }

        for (path_index, mut point_indices) in groups {
            let Some(src) = layer.paths.get(path_index) else { continue; };
            point_indices.sort_unstable();
            point_indices.dedup();
            let mut copied = PPWPath::empty();
            copied.is_closed = false;
            copied.fill_enabled = false;
            copied.fill_color = src.fill_color;
            copied.stroke_width = src.stroke_width;
            copied.stroke_color = src.stroke_color;
            for &idx in &point_indices {
                if let Some(point) = src.control_points.get(idx) {
                    copied.control_points.push(*point);
                    copied.weights.push(src.weights.get(idx).copied().unwrap_or(1.0));
                }
            }
            let seg_count = copied.control_points.len().saturating_sub(1);
            copied.phis.resize(seg_count, 2.0);
            copied.psis.resize(seg_count, 0.0);
            for seg in 0..seg_count {
                let original_a = point_indices[seg];
                let original_b = point_indices[seg + 1];
                if original_b == original_a + 1 {
                    if let Some(phi) = src.phis.get(original_a) {
                        copied.phis[seg] = *phi;
                    }
                    if let Some(psi) = src.psis.get(original_a) {
                        copied.psis[seg] = *psi;
                    }
                }
            }
            if !copied.control_points.is_empty() {
                self.copied_point_paths.push(CopiedPathPoints { path: copied });
            }
        }
        self.io_status = format!("Copied {} point path(s)", self.copied_point_paths.len());
    }

    pub fn paste_copied_points(&mut self) {
        if self.copied_point_paths.is_empty() {
            self.io_status = "Paste failed: copied point buffer is empty".to_string();
            return;
        }
        if self.document.active_layer().map(|l| l.kind != crate::model::document::LayerKind::Vector || l.locked).unwrap_or(true) {
            self.io_status = "Paste failed: choose an unlocked vector layer".to_string();
            return;
        }
        self.push_undo_snapshot();
        self.clear_selection_state();
        let offset = Vec2::new(20.0, 20.0);
        let mut pasted_nodes = Vec::new();
        let mut new_selected_path = self.selected_path;
        if let Some(layer) = self.document.active_layer_mut() {
            let start_path = layer.paths.len();
            for copied in &self.copied_point_paths {
                let mut path = copied.path.clone();
                for point in &mut path.control_points {
                    *point += offset;
                }
                let path_index = layer.paths.len();
                for point_index in 0..path.control_points.len() {
                    pasted_nodes.push(PointSelection { path_index, point_index });
                }
                layer.paths.push(path);
            }
            new_selected_path = start_path.min(layer.paths.len().saturating_sub(1));
        }
        self.selected_path = new_selected_path;
        self.selected_nodes = pasted_nodes;
        sync_after_paste_selection(self);
        self.io_status = "Pasted copied point path(s)".to_string();
    }
}

fn sync_after_paste_selection(app: &mut VectorEditorApp) {
    app.selected_nodes.sort_unstable();
    if let Some(active) = app.selected_nodes.last().copied() {
        app.selected_path = active.path_index;
        app.selected_point = Some(active.point_index);
        app.selected_points = app
            .selected_nodes
            .iter()
            .filter(|node| node.path_index == active.path_index)
            .map(|node| node.point_index)
            .collect();
    }
}

impl eframe::App for VectorEditorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        update::update(self, ctx, frame);
    }
}
