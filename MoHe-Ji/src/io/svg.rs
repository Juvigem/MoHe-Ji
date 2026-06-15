use std::fs;
use std::io;
use std::io::Cursor;

use base64::{engine::general_purpose, Engine as _};
use image::{ColorType, ImageEncoder};
use image::codecs::png::PngEncoder;

use crate::model::document::{Document, Layer, LayerKind, RasterImage, RasterStroke};
use crate::ppw::{PPWCurve, PPWPath, Vec2};

pub fn save_svg(document: &Document, file_path: &str) -> io::Result<()> {
    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{:.0}" height="{:.0}" viewBox="0 0 {:.0} {:.0}">"#, document.canvas_width, document.canvas_height, document.canvas_width, document.canvas_height).as_str());
    out.push('\n');
    out.push_str("  <desc>MoHe-Ji SVG. PPW data, raster layers, and placed images are stored in data-ppw-* attributes.</desc>\n");

    for (layer_index, layer) in document.layers.iter().enumerate() {
        out.push_str(&format!(
            "  <g id=\"layer-{}\" data-layer-name=\"{}\" data-layer-kind=\"{}\" data-visible=\"{}\" data-locked=\"{}\"{}>\n",
            layer_index + 1,
            escape_xml(&layer.name),
            layer.kind.label(),
            layer.visible,
            layer.locked,
            if layer.visible { "" } else { " display=\"none\"" }
        ));

        if layer.kind == LayerKind::Raster {
            if let Some(data_uri) = rgba_to_png_data_uri(&layer.raster_pixels_rgba, layer.raster_width, layer.raster_height) {
                out.push_str(&format!(
                    "    <image id=\"raster-canvas-{}\" x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" href=\"{}\" xlink:href=\"{}\" data-ppw-raster-canvas=\"true\" data-raster-width=\"{}\" data-raster-height=\"{}\" />\n",
                    layer_index + 1,
                    layer.raster_width,
                    layer.raster_height,
                    data_uri,
                    data_uri,
                    layer.raster_width,
                    layer.raster_height,
                ));
            }

            // 旧形式との互換用。ベクター風に保存されていたラスターストロークも残す。
            for (stroke_index, stroke) in layer.raster_strokes.iter().enumerate() {
                if stroke.points.len() < 2 {
                    continue;
                }
                out.push_str(&format!(
                    "    <polyline id=\"raster-{}-{}\" fill=\"none\" stroke=\"#{:02x}{:02x}{:02x}\" stroke-opacity=\"{:.4}\" stroke-width=\"{:.4}\" stroke-linecap=\"round\" stroke-linejoin=\"round\" points=\"{}\" data-raster-stroke=\"true\" data-raster-color=\"{}\" data-raster-width=\"{:.4}\" />\n",
                    layer_index + 1,
                    stroke_index + 1,
                    stroke.color[0], stroke.color[1], stroke.color[2],
                    stroke.color[3] as f32 / 255.0,
                    stroke.width,
                    format_points(&stroke.points),
                    format_color(stroke.color),
                    stroke.width,
                ));
            }
        }

        for (image_index, image) in layer.images.iter().enumerate() {
            if let Some(data_uri) = rgba_to_png_data_uri(&image.pixels_rgba, image.width, image.height) {
                out.push_str(&format!(
                    "    <image id=\"placed-image-{}-{}\" x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" href=\"{}\" xlink:href=\"{}\" data-ppw-image=\"true\" data-image-id=\"{}\" data-image-name=\"{}\" data-image-original-width=\"{}\" data-image-original-height=\"{}\" />\n",
                    layer_index + 1,
                    image_index + 1,
                    image.pos.x,
                    image.pos.y,
                    image.size.x,
                    image.size.y,
                    data_uri,
                    data_uri,
                    image.id,
                    escape_xml(&image.name),
                    image.width,
                    image.height,
                ));
            }
        }

        for (path_index, path) in layer.paths.iter().enumerate() {
            if path.control_points.is_empty() {
                continue;
            }

            let polygon = PPWCurve::convert(path);
            let points = if polygon.polygon.is_empty() { path.control_points.clone() } else { polygon.polygon };

            out.push_str(&format!(
                "    <polyline id=\"path-{}-{}\" fill=\"{}\" fill-opacity=\"{:.4}\" stroke=\"{}\" stroke-width=\"{:.4}\" points=\"{}\" data-ppw=\"true\" data-ppw-closed=\"{}\" data-ppw-control-points=\"{}\" data-ppw-weights=\"{}\" data-ppw-phis=\"{}\" data-ppw-psis=\"{}\" data-ppw-fill-enabled=\"{}\" data-ppw-fill-color=\"{}\" data-ppw-stroke-width=\"{:.4}\" data-ppw-stroke-color=\"{}\" />\n",
                layer_index + 1,
                path_index + 1,
                if path.fill_enabled { format!("#{:02x}{:02x}{:02x}", path.fill_color[0], path.fill_color[1], path.fill_color[2]) } else { "none".to_string() },
                path.fill_color[3] as f32 / 255.0,
                format!("#{:02x}{:02x}{:02x}", path.stroke_color[0], path.stroke_color[1], path.stroke_color[2]),
                path.stroke_width,
                format_points(&points),
                path.is_closed,
                format_points(&path.control_points),
                format_numbers(&path.weights),
                format_numbers(&path.phis),
                format_numbers(&path.psis),
                path.fill_enabled,
                format_color(path.fill_color),
                path.stroke_width,
                format_color(path.stroke_color),
            ));
        }

        out.push_str("  </g>\n");
    }

    out.push_str("</svg>\n");
    fs::write(file_path, out)
}

pub fn load_svg(file_path: &str) -> Result<Document, String> {
    let text = fs::read_to_string(file_path).map_err(|e| e.to_string())?;

    let canvas_width = svg_attr(&text, "width").and_then(|v| parse_dimension(&v)).unwrap_or(600.0);
    let canvas_height = svg_attr(&text, "height").and_then(|v| parse_dimension(&v)).unwrap_or(800.0);

    let mut layers: Vec<Layer> = Vec::new();
    let mut current_layer: Option<Layer> = None;
    let mut max_image_id = 0u64;

    for raw_line in text.lines() {
        let line = raw_line.trim();

        if line.starts_with("<g ") || line.starts_with("<g>") {
            if let Some(layer) = current_layer.take() { layers.push(layer); }

            let name = attr(line, "data-layer-name").unwrap_or_else(|| format!("Layer {}", layers.len() + 1));
            let visible = attr(line, "data-visible").and_then(|v| v.parse::<bool>().ok()).unwrap_or(!line.contains("display=\"none\""));
            let locked = attr(line, "data-locked").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);
            let kind = match attr(line, "data-layer-kind").as_deref() { Some("Raster") => LayerKind::Raster, _ => LayerKind::Vector };

            current_layer = Some(Layer {
                name: unescape_xml(&name), visible, locked, kind,
                paths: Vec::new(), raster_strokes: Vec::new(), raster_pixels_rgba: Vec::new(), raster_width: 0, raster_height: 0, images: Vec::new(),
            });
        }

        if line.contains("data-ppw-raster-canvas=\"true\"") {
            if current_layer.is_none() {
                current_layer = Some(Layer { name: format!("Raster Layer {}", layers.len() + 1), visible: true, locked: false, kind: LayerKind::Raster, paths: Vec::new(), raster_strokes: Vec::new(), raster_pixels_rgba: Vec::new(), raster_width: 0, raster_height: 0, images: Vec::new() });
            }
            if let Some((pixels, width, height)) = image_from_data_uri_attr(line) {
                if let Some(layer) = &mut current_layer {
                    layer.kind = LayerKind::Raster;
                    layer.raster_pixels_rgba = pixels;
                    layer.raster_width = width;
                    layer.raster_height = height;
                }
            }
        }

        if line.contains("data-ppw-image=\"true\"") {
            if current_layer.is_none() {
                current_layer = Some(Layer { name: format!("Layer {}", layers.len() + 1), visible: true, locked: false, kind: LayerKind::Vector, paths: Vec::new(), raster_strokes: Vec::new(), raster_pixels_rgba: Vec::new(), raster_width: 0, raster_height: 0, images: Vec::new() });
            }
            if let Some((pixels, original_width, original_height)) = image_from_data_uri_attr(line) {
                let id = attr(line, "data-image-id").and_then(|v| v.parse::<u64>().ok()).unwrap_or(max_image_id + 1);
                max_image_id = max_image_id.max(id);
                let name = attr(line, "data-image-name").map(|v| unescape_xml(&v)).unwrap_or_else(|| "Image".to_string());
                let x = attr(line, "x").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
                let y = attr(line, "y").and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
                let w = attr(line, "width").and_then(|v| v.parse::<f32>().ok()).unwrap_or(original_width as f32).max(1.0);
                let h = attr(line, "height").and_then(|v| v.parse::<f32>().ok()).unwrap_or(original_height as f32).max(1.0);
                if let Some(layer) = &mut current_layer {
                    layer.images.push(RasterImage { id, name, pixels_rgba: pixels, width: original_width, height: original_height, pos: Vec2::new(x, y), size: Vec2::new(w, h), rotation: 0.0 });
                }
            }
        }

        if line.contains("data-raster-stroke=\"true\"") {
            let points_text = attr(line, "points").unwrap_or_default();
            let stroke = RasterStroke {
                points: parse_points(&points_text),
                width: attr(line, "data-raster-width").or_else(|| attr(line, "stroke-width")).and_then(|v| v.parse::<f32>().ok()).unwrap_or(8.0).max(0.1),
                color: attr(line, "data-raster-color").and_then(|v| parse_color(&v)).or_else(|| attr(line, "stroke").and_then(|v| parse_hex_color(&v))).unwrap_or([0, 0, 0, 180]),
            };
            if current_layer.is_none() {
                current_layer = Some(Layer { name: format!("Raster Layer {}", layers.len() + 1), visible: true, locked: false, kind: LayerKind::Raster, paths: Vec::new(), raster_strokes: Vec::new(), raster_pixels_rgba: Vec::new(), raster_width: 0, raster_height: 0, images: Vec::new() });
            }
            if let Some(layer) = &mut current_layer { layer.kind = LayerKind::Raster; layer.raster_strokes.push(stroke); }
        }

        if line.contains("data-ppw=\"true\"") {
            let cp_text = attr(line, "data-ppw-control-points").or_else(|| attr(line, "points")).ok_or_else(|| "SVG path has no control point data.".to_string())?;
            let mut path = PPWPath {
                is_closed: attr(line, "data-ppw-closed").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false),
                control_points: parse_points(&cp_text),
                weights: attr(line, "data-ppw-weights").map(|v| parse_numbers(&v)).unwrap_or_default(),
                phis: attr(line, "data-ppw-phis").map(|v| parse_numbers(&v)).unwrap_or_default(),
                psis: attr(line, "data-ppw-psis").map(|v| parse_numbers(&v)).unwrap_or_default(),
                fill_enabled: attr(line, "data-ppw-fill-enabled").and_then(|v| v.parse::<bool>().ok()).unwrap_or_else(|| attr(line, "fill").map(|v| v != "none").unwrap_or(false)),
                fill_color: attr(line, "data-ppw-fill-color").and_then(|v| parse_color(&v)).unwrap_or([120, 170, 255, 70]),
                stroke_width: attr(line, "data-ppw-stroke-width").or_else(|| attr(line, "stroke-width")).and_then(|v| v.parse::<f32>().ok()).unwrap_or(2.5).max(0.1),
                stroke_color: attr(line, "data-ppw-stroke-color").and_then(|v| parse_color(&v)).or_else(|| attr(line, "stroke").and_then(|v| parse_hex_color(&v))).unwrap_or([0, 0, 0, 255]),
            };
            path.rebuild_open_segment_params();
            if current_layer.is_none() { current_layer = Some(Layer { name: format!("Layer {}", layers.len() + 1), visible: true, locked: false, kind: LayerKind::Vector, paths: Vec::new(), raster_strokes: Vec::new(), raster_pixels_rgba: Vec::new(), raster_width: 0, raster_height: 0, images: Vec::new() }); }
            if let Some(layer) = &mut current_layer { layer.paths.push(path); }
        }

        if line.starts_with("</g") { if let Some(layer) = current_layer.take() { layers.push(layer); } }
    }

    if let Some(layer) = current_layer.take() { layers.push(layer); }

    if layers.is_empty() { return Err("No compatible vector, raster, or image layers were found in the SVG.".to_string()); }

    for layer in &mut layers {
        if layer.kind == LayerKind::Vector && layer.paths.is_empty() { layer.paths.push(PPWPath::empty()); }
        if layer.kind == LayerKind::Raster { layer.paths.clear(); }
    }

    let mut document = Document { layers, active_layer: 0, canvas_width, canvas_height, next_image_id: max_image_id.saturating_add(1).max(1) };
    document.normalize();
    Ok(document)
}

fn rgba_to_png_data_uri(pixels: &[u8], width: u32, height: u32) -> Option<String> {
    if width == 0 || height == 0 { return None; }
    let expected = (width as usize).checked_mul(height as usize)?.checked_mul(4)?;
    if pixels.len() != expected { return None; }
    let mut png = Vec::new();
    let encoder = PngEncoder::new(Cursor::new(&mut png));
    encoder.write_image(pixels, width, height, ColorType::Rgba8.into()).ok()?;
    Some(format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(png)))
}

fn image_from_data_uri_attr(line: &str) -> Option<(Vec<u8>, u32, u32)> {
    let href = attr(line, "href").or_else(|| attr(line, "xlink:href"))?;
    let data = href.strip_prefix("data:image/png;base64,").or_else(|| href.strip_prefix("data:image/*;base64,"))?;
    let bytes = general_purpose::STANDARD.decode(data).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some((img.into_raw(), width, height))
}

fn svg_attr(text: &str, name: &str) -> Option<String> {
    let svg_start = text.find("<svg")?;
    let svg_end = text[svg_start..].find('>')? + svg_start;
    attr(&text[svg_start..=svg_end], name)
}

fn parse_dimension(text: &str) -> Option<f32> {
    let numeric = text.trim().trim_end_matches("px").parse::<f32>().ok()?;
    Some(numeric.max(1.0))
}

fn attr(line: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn format_points(points: &[Vec2]) -> String {
    points.iter().map(|p| format!("{:.4},{:.4}", p.x, p.y)).collect::<Vec<_>>().join(" ")
}

fn parse_points(text: &str) -> Vec<Vec2> {
    text.split_whitespace().filter_map(|pair| {
        let mut parts = pair.split(',');
        let x = parts.next()?.parse::<f32>().ok()?;
        let y = parts.next()?.parse::<f32>().ok()?;
        Some(Vec2::new(x, y))
    }).collect()
}

fn format_numbers(values: &[f32]) -> String {
    values.iter().map(|v| format!("{:.4}", v)).collect::<Vec<_>>().join(" ")
}

fn parse_numbers(text: &str) -> Vec<f32> {
    text.split_whitespace().filter_map(|v| v.parse::<f32>().ok()).collect()
}

fn format_color(color: [u8; 4]) -> String { format!("{},{},{},{}", color[0], color[1], color[2], color[3]) }

fn parse_color(text: &str) -> Option<[u8; 4]> {
    let parts = text.split(',').filter_map(|v| v.trim().parse::<u8>().ok()).collect::<Vec<_>>();
    if parts.len() == 4 { Some([parts[0], parts[1], parts[2], parts[3]]) } else { None }
}

fn parse_hex_color(text: &str) -> Option<[u8; 4]> {
    let s = text.trim();
    if !s.starts_with('#') { return None; }
    let hex = &s[1..];
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b, 255])
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}

fn unescape_xml(s: &str) -> String {
    s.replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}
