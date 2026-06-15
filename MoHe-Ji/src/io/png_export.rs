
use std::io;

use image::{Rgba, RgbaImage};

use crate::model::document::{Document, LayerKind, RasterImage, RasterStroke};
use crate::ppw::{PPWCurve, Vec2};

pub fn save_png(document: &Document, file_path: &str, transparent_empty_pixels: bool, quality_scale: f32) -> io::Result<()> {
    let scale = quality_scale.clamp(0.25, 4.0);
    let width = (document.canvas_width.max(1.0) * scale).round() as u32;
    let height = (document.canvas_height.max(1.0) * scale).round() as u32;
    let background = if transparent_empty_pixels {
        Rgba([255, 255, 255, 0])
    } else {
        Rgba([255, 255, 255, 255])
    };
    let mut image = RgbaImage::from_pixel(width, height, background);

    for layer in &document.layers {
        if !layer.visible {
            continue;
        }

        for raster_image in &layer.images {
            draw_raster_image(&mut image, raster_image, scale);
        }

        if layer.kind == LayerKind::Raster {
            draw_raster_canvas_pixels(&mut image, layer, scale);
            for stroke in &layer.raster_strokes {
                draw_stroke(&mut image, stroke, scale);
            }
            continue;
        }

        for path in &layer.paths {
            if path.control_points.is_empty() {
                continue;
            }

            let ppw = PPWCurve::convert(path);
            let points = if ppw.polygon.is_empty() {
                path.control_points.clone()
            } else {
                ppw.polygon
            };

            if path.is_closed && path.fill_enabled && points.len() >= 3 {
                fill_polygon(&mut image, &scale_points(&points, scale), path.fill_color);
            }

            if points.len() >= 2 {
                let stroke = RasterStroke {
                    points,
                    width: path.stroke_width,
                    color: path.stroke_color,
                };
                draw_stroke(&mut image, &stroke, scale);
            }
        }
    }

    image.save(file_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}



fn draw_raster_canvas_pixels(canvas: &mut RgbaImage, layer: &crate::model::document::Layer, scale: f32) {
    if layer.raster_width == 0 || layer.raster_height == 0 {
        return;
    }
    let expected = (layer.raster_width as usize).saturating_mul(layer.raster_height as usize).saturating_mul(4);
    if layer.raster_pixels_rgba.len() != expected {
        return;
    }
    let out_w = (layer.raster_width as f32 * scale).round().max(1.0) as u32;
    let out_h = (layer.raster_height as f32 * scale).round().max(1.0) as u32;
    for y in 0..out_h.min(canvas.height()) {
        let src_y = ((y as f32 / scale).floor() as u32).min(layer.raster_height - 1);
        for x in 0..out_w.min(canvas.width()) {
            let src_x = ((x as f32 / scale).floor() as u32).min(layer.raster_width - 1);
            let idx = ((src_y * layer.raster_width + src_x) as usize) * 4;
            blend_pixel(canvas, x, y, [
                layer.raster_pixels_rgba[idx],
                layer.raster_pixels_rgba[idx + 1],
                layer.raster_pixels_rgba[idx + 2],
                layer.raster_pixels_rgba[idx + 3],
            ]);
        }
    }
}

fn scale_points(points: &[Vec2], scale: f32) -> Vec<Vec2> {
    points.iter().map(|p| Vec2::new(p.x * scale, p.y * scale)).collect()
}

fn draw_raster_image(canvas: &mut RgbaImage, raster_image: &RasterImage, scale: f32) {
    if raster_image.width == 0 || raster_image.height == 0 {
        return;
    }
    if raster_image.pixels_rgba.len() != (raster_image.width as usize).saturating_mul(raster_image.height as usize).saturating_mul(4) {
        return;
    }

    let start_x = (raster_image.pos.x * scale).floor() as i32;
    let start_y = (raster_image.pos.y * scale).floor() as i32;
    let out_w = (raster_image.size.x.abs() * scale).round().max(1.0) as i32;
    let out_h = (raster_image.size.y.abs() * scale).round().max(1.0) as i32;

    for y in 0..out_h {
        let canvas_y = start_y + y;
        if canvas_y < 0 || canvas_y >= canvas.height() as i32 {
            continue;
        }
        let src_y = ((y as f32 / out_h.max(1) as f32) * raster_image.height as f32)
            .floor()
            .clamp(0.0, raster_image.height.saturating_sub(1) as f32) as u32;

        for x in 0..out_w {
            let canvas_x = start_x + x;
            if canvas_x < 0 || canvas_x >= canvas.width() as i32 {
                continue;
            }
            let src_x = ((x as f32 / out_w.max(1) as f32) * raster_image.width as f32)
                .floor()
                .clamp(0.0, raster_image.width.saturating_sub(1) as f32) as u32;
            let src_index = ((src_y * raster_image.width + src_x) as usize) * 4;
            let src = [
                raster_image.pixels_rgba[src_index],
                raster_image.pixels_rgba[src_index + 1],
                raster_image.pixels_rgba[src_index + 2],
                raster_image.pixels_rgba[src_index + 3],
            ];
            blend_pixel(canvas, canvas_x as u32, canvas_y as u32, src);
        }
    }
}


fn draw_stroke(image: &mut RgbaImage, stroke: &RasterStroke, scale: f32) {
    if stroke.points.is_empty() || stroke.width <= 0.0 || stroke.color[3] == 0 {
        return;
    }

    let points = scale_points(&stroke.points, scale);
    if points.len() == 1 {
        stamp_circle(image, points[0], stroke.width * scale * 0.5, stroke.color);
        return;
    }

    let radius = (stroke.width * scale * 0.5).max(0.5);
    let step = (radius * 0.35).clamp(0.25, 4.0);

    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let delta = b - a;
        let len = delta.length();
        if len <= f32::EPSILON {
            stamp_circle(image, a, radius, stroke.color);
            continue;
        }

        let count = (len / step).ceil().max(1.0) as usize;
        for i in 0..=count {
            let t = i as f32 / count as f32;
            stamp_circle(image, a + delta * t, radius, stroke.color);
        }
    }
}

fn stamp_circle(image: &mut RgbaImage, center: Vec2, radius: f32, color: [u8; 4]) {
    let min_x = (center.x - radius).floor().max(0.0) as i32;
    let max_x = (center.x + radius).ceil().min(image.width() as f32 - 1.0) as i32;
    let min_y = (center.y - radius).floor().max(0.0) as i32;
    let max_y = (center.y + radius).ceil().min(image.height() as f32 - 1.0) as i32;
    let r2 = radius * radius;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - center.x;
            let dy = y as f32 + 0.5 - center.y;
            if dx * dx + dy * dy <= r2 {
                blend_pixel(image, x as u32, y as u32, color);
            }
        }
    }
}

fn fill_polygon(image: &mut RgbaImage, points: &[Vec2], color: [u8; 4]) {
    if points.len() < 3 || color[3] == 0 {
        return;
    }

    let min_y = points
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as i32;
    let max_y = points
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(image.height() as f32 - 1.0) as i32;

    for y in min_y..=max_y {
        let scan_y = y as f32 + 0.5;
        let mut xs = Vec::new();

        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];

            if (a.y <= scan_y && b.y > scan_y) || (b.y <= scan_y && a.y > scan_y) {
                let t = (scan_y - a.y) / (b.y - a.y);
                xs.push(a.x + (b.x - a.x) * t);
            }
        }

        xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        for pair in xs.chunks(2) {
            if pair.len() != 2 {
                continue;
            }
            let start = pair[0].floor().max(0.0) as i32;
            let end = pair[1].ceil().min(image.width() as f32 - 1.0) as i32;
            for x in start..=end {
                blend_pixel(image, x as u32, y as u32, color);
            }
        }
    }
}

fn blend_pixel(image: &mut RgbaImage, x: u32, y: u32, src: [u8; 4]) {
    let dst = image.get_pixel_mut(x, y);
    let sa = src[3] as f32 / 255.0;
    if sa <= 0.0 {
        return;
    }

    let da = dst[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);

    if out_a <= f32::EPSILON {
        *dst = Rgba([0, 0, 0, 0]);
        return;
    }

    let mut out = [0u8; 4];
    for c in 0..3 {
        let sc = src[c] as f32 / 255.0;
        let dc = dst[c] as f32 / 255.0;
        let oc = (sc * sa + dc * da * (1.0 - sa)) / out_a;
        out[c] = (oc * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    *dst = Rgba(out);
}
