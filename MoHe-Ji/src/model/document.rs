use crate::ppw::{path::PPWPath, vec2::Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKind {
    Vector,
    Raster,
}

impl LayerKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Vector => "Vector",
            Self::Raster => "Raster",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RasterStroke {
    pub points: Vec<Vec2>,
    pub width: f32,
    pub color: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct RasterImage {
    pub id: u64,
    pub name: String,
    pub pixels_rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub pos: Vec2,
    pub size: Vec2,
    pub rotation: f32,
}


#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub kind: LayerKind,
    pub paths: Vec<PPWPath>,
    pub raster_strokes: Vec<RasterStroke>,
    pub raster_pixels_rgba: Vec<u8>,
    pub raster_width: u32,
    pub raster_height: u32,
    pub images: Vec<RasterImage>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visible: true,
            locked: false,
            kind: LayerKind::Vector,
            paths: vec![PPWPath::empty()],
            raster_strokes: Vec::new(),
            raster_pixels_rgba: Vec::new(),
            raster_width: 0,
            raster_height: 0,
            images: Vec::new(),
        }
    }


    pub fn new_raster(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visible: true,
            locked: false,
            kind: LayerKind::Raster,
            paths: Vec::new(),
            raster_strokes: Vec::new(),
            raster_pixels_rgba: Vec::new(),
            raster_width: 0,
            raster_height: 0,
            images: Vec::new(),
        }
    }

    pub fn with_paths(name: impl Into<String>, paths: Vec<PPWPath>) -> Self {
        Self {
            name: name.into(),
            visible: true,
            locked: false,
            kind: LayerKind::Vector,
            paths,
            raster_strokes: Vec::new(),
            raster_pixels_rgba: Vec::new(),
            raster_width: 0,
            raster_height: 0,
            images: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub layers: Vec<Layer>,
    pub active_layer: usize,
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub next_image_id: u64,
}

impl Document {
    pub fn sample_ppw() -> Self {
        let control_points = vec![
            Vec2::new(90.0, 440.0),
            Vec2::new(210.0, 180.0),
            Vec2::new(410.0, 430.0),
            Vec2::new(590.0, 170.0),
            Vec2::new(760.0, 420.0),
        ];

        let point_count = control_points.len();

        Self {
            layers: vec![Layer::with_paths(
                "Layer 1",
                vec![PPWPath {
                    is_closed: false,
                    control_points,
                    weights: vec![1.0, 1.4, 0.7, 1.8, 1.0],
                    phis: vec![2.0; point_count - 1],
                    psis: vec![0.0; point_count - 1],
                    fill_enabled: true,
                    fill_color: [120, 170, 255, 70],
                    stroke_width: 2.5,
                    stroke_color: [0, 0, 0, 255],
                }],
            )],
            active_layer: 0,
            canvas_width: 600.0,
            canvas_height: 800.0,
            next_image_id: 1,
        }
    }

    pub fn empty_path() -> Self {
        Self {
            layers: vec![Layer::new("Layer 1")],
            active_layer: 0,
            canvas_width: 600.0,
            canvas_height: 800.0,
            next_image_id: 1,
        }
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers.get(self.active_layer)
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers.get_mut(self.active_layer)
    }

    pub fn active_paths(&self) -> &[PPWPath] {
        self.active_layer()
            .map(|layer| layer.paths.as_slice())
            .unwrap_or(&[])
    }

    pub fn active_paths_mut(&mut self) -> Option<&mut Vec<PPWPath>> {
        self.active_layer_mut().map(|layer| &mut layer.paths)
    }

    pub fn active_path_mut(&mut self, path_index: usize) -> Option<&mut PPWPath> {
        self.active_layer_mut()?.paths.get_mut(path_index)
    }

    pub fn add_layer(&mut self) {
        self.add_vector_layer();
    }

    pub fn add_vector_layer(&mut self) {
        let name = format!("Vector Layer {}", self.layers.len() + 1);
        self.layers.push(Layer::new(name));
        self.active_layer = self.layers.len() - 1;
    }

    pub fn add_raster_layer(&mut self) {
        let name = format!("Raster Layer {}", self.layers.len() + 1);
        let mut layer = Layer::new_raster(name);
        ensure_layer_raster_canvas(
            &mut layer,
            self.canvas_width.max(1.0).round() as u32,
            self.canvas_height.max(1.0).round() as u32,
        );
        self.layers.push(layer);
        self.active_layer = self.layers.len() - 1;
    }

    pub fn duplicate_active_layer(&mut self) {
        if let Some(layer) = self.active_layer().cloned() {
            let mut duplicated = layer;
            duplicated.name = format!("{} Copy", duplicated.name);
            let insert_at = self.active_layer + 1;
            self.layers.insert(insert_at, duplicated);
            self.active_layer = insert_at;
        }
    }

    pub fn delete_active_layer(&mut self) {
        if self.layers.len() <= 1 {
            self.layers[0] = Layer::new("Layer 1");
            self.active_layer = 0;
            return;
        }

        if self.active_layer < self.layers.len() {
            self.layers.remove(self.active_layer);
        }
        self.active_layer = self.active_layer.min(self.layers.len().saturating_sub(1));
    }

    pub fn move_active_layer_up(&mut self) {
        if self.active_layer + 1 < self.layers.len() {
            self.layers.swap(self.active_layer, self.active_layer + 1);
            self.active_layer += 1;
        }
    }

    pub fn move_active_layer_down(&mut self) {
        if self.active_layer > 0 {
            self.layers.swap(self.active_layer, self.active_layer - 1);
            self.active_layer -= 1;
        }
    }


    pub fn add_image_to_active_layer(
        &mut self,
        name: impl Into<String>,
        pixels_rgba: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Option<(usize, u64)> {
        let id = self.next_image_id;
        self.next_image_id = self.next_image_id.saturating_add(1);
        let max_w = (self.canvas_width * 0.5).max(32.0);
        let max_h = (self.canvas_height * 0.5).max(32.0);
        let scale = (max_w / width.max(1) as f32).min(max_h / height.max(1) as f32).min(1.0);
        let size = Vec2::new(width as f32 * scale, height as f32 * scale);
        let layer = self.active_layer_mut()?;
        if layer.locked {
            return None;
        }
        layer.images.push(RasterImage {
            id,
            name: name.into(),
            pixels_rgba,
            width,
            height,
            pos: Vec2::new(40.0, 40.0),
            size,
            rotation: 0.0,
        });
        Some((layer.images.len().saturating_sub(1), id))
    }

    pub fn remove_selected_points(
        &mut self,
        fallback_path_index: usize,
        fallback_points: &[usize],
        selected_nodes: &[crate::app::PointSelection],
    ) {
        let Some(layer) = self.active_layer_mut() else {
            return;
        };

        let mut grouped: Vec<(usize, Vec<usize>)> = Vec::new();

        if selected_nodes.is_empty() {
            if !fallback_points.is_empty() {
                grouped.push((fallback_path_index, fallback_points.to_vec()));
            }
        } else {
            for node in selected_nodes {
                if let Some((_, points)) = grouped.iter_mut().find(|(path, _)| *path == node.path_index) {
                    points.push(node.point_index);
                } else {
                    grouped.push((node.path_index, vec![node.point_index]));
                }
            }
        }

        grouped.sort_by_key(|(path_index, _)| *path_index);
        for (path_index, mut points) in grouped.into_iter().rev() {
            if let Some(path) = layer.paths.get_mut(path_index) {
                points.sort_unstable();
                points.dedup();
                path.remove_points(&points);
            }
        }
    }


    pub fn ensure_active_raster_canvas(&mut self) {
        let width = self.canvas_width.max(1.0).round() as u32;
        let height = self.canvas_height.max(1.0).round() as u32;
        if let Some(layer) = self.active_layer_mut() {
            if layer.kind == LayerKind::Raster {
                ensure_layer_raster_canvas(layer, width, height);
            }
        }
    }

    pub fn normalize(&mut self) {
        if self.layers.is_empty() {
            self.layers.push(Layer::new("Layer 1"));
            self.active_layer = 0;
        }
        self.active_layer = self.active_layer.min(self.layers.len() - 1);
        let max_image_id = self.layers.iter()
            .flat_map(|layer| layer.images.iter().map(|image| image.id))
            .max()
            .unwrap_or(0);
        self.next_image_id = self.next_image_id.max(max_image_id.saturating_add(1));
        let raster_canvas_width = self.canvas_width.max(1.0).round() as u32;
        let raster_canvas_height = self.canvas_height.max(1.0).round() as u32;
        for layer in &mut self.layers {
            match layer.kind {
                LayerKind::Vector => {
                    if layer.paths.is_empty() {
                        layer.paths.push(PPWPath::empty());
                    }
                    for path in &mut layer.paths {
                        path.rebuild_open_segment_params();
                    }
                }
                LayerKind::Raster => {
                    layer.paths.clear();
                    ensure_layer_raster_canvas(layer, raster_canvas_width, raster_canvas_height);
                }
            }
        }
    }
}

pub fn ensure_layer_raster_canvas(layer: &mut Layer, width: u32, height: u32) {
    if layer.kind != LayerKind::Raster {
        return;
    }
    let width = width.max(1);
    let height = height.max(1);
    let needed = (width as usize).saturating_mul(height as usize).saturating_mul(4);
    if layer.raster_width == width && layer.raster_height == height && layer.raster_pixels_rgba.len() == needed {
        return;
    }

    let mut next = vec![0u8; needed];
    let copy_w = layer.raster_width.min(width);
    let copy_h = layer.raster_height.min(height);
    for y in 0..copy_h {
        let old_start = ((y * layer.raster_width) as usize) * 4;
        let new_start = ((y * width) as usize) * 4;
        let len = (copy_w as usize) * 4;
        if old_start + len <= layer.raster_pixels_rgba.len() && new_start + len <= next.len() {
            next[new_start..new_start + len].copy_from_slice(&layer.raster_pixels_rgba[old_start..old_start + len]);
        }
    }
    layer.raster_width = width;
    layer.raster_height = height;
    layer.raster_pixels_rgba = next;
}

impl PPWPath {
    pub fn empty() -> Self {
        Self {
            is_closed: false,
            control_points: Vec::new(),
            weights: Vec::new(),
            phis: Vec::new(),
            psis: Vec::new(),
            fill_enabled: false,
            fill_color: [120, 170, 255, 70],
            stroke_width: 2.5,
            stroke_color: [0, 0, 0, 255],
        }
    }


    pub fn from_rectangle(a: Vec2, b: Vec2, stroke_width: f32) -> Self {
        // Kept for compatibility with older SVG/imported data.
        // New rectangle-tool creation uses `from_rectangle_diagonal_paths`.
        let min_x = a.x.min(b.x);
        let max_x = a.x.max(b.x);
        let min_y = a.y.min(b.y);
        let max_y = a.y.max(b.y);

        let mut path = Self {
            is_closed: true,
            control_points: vec![
                Vec2::new(min_x, min_y),
                Vec2::new(max_x, min_y),
                Vec2::new(max_x, max_y),
                Vec2::new(min_x, max_y),
            ],
            weights: Vec::new(),
            phis: Vec::new(),
            psis: Vec::new(),
            fill_enabled: false,
            fill_color: [120, 170, 255, 70],
            stroke_width: stroke_width.max(0.1),
            stroke_color: [0, 0, 0, 255],
        };
        path.rebuild_open_segment_params();
        path
    }

    pub fn from_rectangle_diagonal_paths(a: Vec2, b: Vec2, stroke_width: f32) -> Vec<Self> {
        let min_x = a.x.min(b.x);
        let max_x = a.x.max(b.x);
        let min_y = a.y.min(b.y);
        let max_y = a.y.max(b.y);

        let top_left = Vec2::new(min_x, min_y);
        let top_right = Vec2::new(max_x, min_y);
        let bottom_right = Vec2::new(max_x, max_y);
        let bottom_left = Vec2::new(min_x, max_y);

        let make_open_path = |points: Vec<Vec2>| {
            let mut path = Self {
                is_closed: false,
                control_points: points,
                weights: Vec::new(),
                phis: Vec::new(),
                psis: Vec::new(),
                fill_enabled: false,
                fill_color: [120, 170, 255, 70],
                stroke_width: stroke_width.max(0.1),
                stroke_color: [0, 0, 0, 255],
            };
            path.rebuild_open_segment_params();
            if path.psis.len() >= 2 {
                path.psis[0] = -2.0;
                path.psis[1] = 2.0;
            }
            path
        };

        vec![
            // First PPW curve: top-left -> top-right -> bottom-right.
            make_open_path(vec![top_left, top_right, bottom_right]),
            // Second PPW curve: bottom-right -> bottom-left -> top-left.
            make_open_path(vec![bottom_right, bottom_left, top_left]),
        ]
    }

    pub fn from_ellipse(a: Vec2, b: Vec2, stroke_width: f32) -> Self {
        let min_x = a.x.min(b.x);
        let max_x = a.x.max(b.x);
        let min_y = a.y.min(b.y);
        let max_y = a.y.max(b.y);
        let cx = (min_x + max_x) * 0.5;
        let cy = (min_y + max_y) * 0.5;
        let rx = ((max_x - min_x) * 0.5).max(1.0);
        let ry = ((max_y - min_y) * 0.5).max(1.0);

        let mut points = Vec::new();
        for i in 0..8 {
            let t = i as f32 * std::f32::consts::TAU / 8.0;
            points.push(Vec2::new(cx + rx * t.cos(), cy + ry * t.sin()));
        }

        let mut path = Self {
            is_closed: true,
            control_points: points,
            weights: Vec::new(),
            phis: Vec::new(),
            psis: Vec::new(),
            fill_enabled: false,
            fill_color: [120, 170, 255, 70],
            stroke_width: stroke_width.max(0.1),
            stroke_color: [0, 0, 0, 255],
        };
        path.rebuild_open_segment_params();
        path
    }

    pub fn add_point(&mut self, point: Vec2) {
        self.control_points.push(point);
        self.weights.push(1.0);
        self.rebuild_open_segment_params();
    }

    pub fn move_point(&mut self, index: usize, point: Vec2) {
        if let Some(cp) = self.control_points.get_mut(index) {
            *cp = point;
        }
    }

    pub fn remove_last_point(&mut self) {
        self.control_points.pop();
        self.weights.pop();
        self.rebuild_open_segment_params();
    }

    pub fn rebuild_open_segment_params(&mut self) {
        let n = self.control_points.len();

        if self.is_closed {
            self.weights.resize(n, 1.0);
            self.phis.resize(n, 2.0);
            self.psis.resize(n, 0.0);
        } else {
            self.weights.resize(n, 1.0);
            let seg_count = n.saturating_sub(1);
            self.phis.resize(seg_count, 2.0);
            self.psis.resize(seg_count, 0.0);
        }
    }

    pub fn remove_points(&mut self, indices: &[usize]) {
        if indices.is_empty() {
            return;
        }

        let mut remove = indices.to_vec();
        remove.sort_unstable();
        remove.dedup();

        for index in remove.into_iter().rev() {
            if index < self.control_points.len() {
                self.control_points.remove(index);
            }
            if index < self.weights.len() {
                self.weights.remove(index);
            }
        }

        self.rebuild_open_segment_params();
    }


    pub fn scale_about_center(&mut self, scale_x: f32, scale_y: f32) {
        if self.control_points.is_empty() {
            return;
        }

        let mut min_x = self.control_points[0].x;
        let mut max_x = self.control_points[0].x;
        let mut min_y = self.control_points[0].y;
        let mut max_y = self.control_points[0].y;

        for point in &self.control_points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        let center = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
        let sx = scale_x.max(0.001);
        let sy = scale_y.max(0.001);

        for point in &mut self.control_points {
            point.x = center.x + (point.x - center.x) * sx;
            point.y = center.y + (point.y - center.y) * sy;
        }
    }

    pub fn bounds(&self) -> Option<(Vec2, Vec2)> {
        let first = *self.control_points.first()?;
        let mut min = first;
        let mut max = first;
        for point in &self.control_points {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
        }
        Some((min, max))
    }

    pub fn scale_about_anchor(&mut self, anchor: Vec2, scale_x: f32, scale_y: f32) {
        let sx = scale_x.max(0.001);
        let sy = scale_y.max(0.001);
        for point in &mut self.control_points {
            point.x = anchor.x + (point.x - anchor.x) * sx;
            point.y = anchor.y + (point.y - anchor.y) * sy;
        }
    }

    pub fn rotate_about_center(&mut self, radians: f32) {
        let Some((min, max)) = self.bounds() else { return; };
        let center = Vec2::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5);
        self.rotate_about(center, radians);
    }

    pub fn rotate_about(&mut self, center: Vec2, radians: f32) {
        let (sin, cos) = radians.sin_cos();
        for point in &mut self.control_points {
            let x = point.x - center.x;
            let y = point.y - center.y;
            point.x = center.x + x * cos - y * sin;
            point.y = center.y + x * sin + y * cos;
        }
    }

    pub fn move_points_by_delta(&mut self, indices: &[usize], delta: Vec2) {
        for &index in indices {
            if let Some(cp) = self.control_points.get_mut(index) {
                *cp += delta;
            }
        }
    }

    pub fn points_in_rect(&self, a: Vec2, b: Vec2) -> Vec<usize> {
        let min_x = a.x.min(b.x);
        let max_x = a.x.max(b.x);
        let min_y = a.y.min(b.y);
        let max_y = a.y.max(b.y);

        self.control_points
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }


    pub fn simplify_control_points(&mut self, tolerance: f32) {
        let n = self.control_points.len();
        if n < 3 {
            return;
        }

        let tolerance = tolerance.max(0.0);
        let simplified = if self.is_closed {
            simplify_closed_points(&self.control_points, tolerance)
        } else {
            simplify_open_points(&self.control_points, tolerance)
        };

        if simplified.len() >= if self.is_closed { 3 } else { 2 } {
            self.control_points = simplified;
            self.rebuild_open_segment_params();
        }
    }

    pub fn nearest_control_point(&self, point: Vec2, radius: f32) -> Option<usize> {
        let radius2 = radius * radius;

        self.control_points
            .iter()
            .enumerate()
            .filter_map(|(i, cp)| {
                let d2 = (*cp - point).length_squared();
                if d2 <= radius2 {
                    Some((i, d2))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }
}


fn simplify_open_points(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    rdp_mark(points, 0, points.len() - 1, tolerance * tolerance, &mut keep);

    points
        .iter()
        .zip(keep.iter())
        .filter_map(|(point, keep)| if *keep { Some(*point) } else { None })
        .collect()
}

fn simplify_closed_points(points: &[Vec2], tolerance: f32) -> Vec<Vec2> {
    if points.len() <= 3 {
        return points.to_vec();
    }

    // For closed paths, keep the first point and simplify the open sequence with
    // the first point appended at the end, then remove the duplicate.
    let mut loop_points = points.to_vec();
    loop_points.push(points[0]);
    let mut simplified = simplify_open_points(&loop_points, tolerance);
    if simplified.len() > 1 {
        simplified.pop();
    }
    if simplified.len() < 3 {
        points.to_vec()
    } else {
        simplified
    }
}

fn rdp_mark(points: &[Vec2], start: usize, end: usize, tolerance2: f32, keep: &mut [bool]) {
    if end <= start + 1 {
        return;
    }

    let a = points[start];
    let b = points[end];
    let mut max_distance2 = 0.0_f32;
    let mut max_index = start;

    for i in (start + 1)..end {
        let d2 = distance_point_to_segment_squared(points[i], a, b);
        if d2 > max_distance2 {
            max_distance2 = d2;
            max_index = i;
        }
    }

    if max_distance2 > tolerance2 {
        keep[max_index] = true;
        rdp_mark(points, start, max_index, tolerance2, keep);
        rdp_mark(points, max_index, end, tolerance2, keep);
    }
}

fn distance_point_to_segment_squared(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 <= f32::EPSILON {
        return (p - a).length_squared();
    }

    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    let projection = a + ab * t;
    (p - projection).length_squared()
}

