use super::vec2::Vec2;

#[derive(Debug, Clone, Default)]
pub struct TriangulateResult {
    pub vertices: Vec<Vec2>,
    pub indices: Vec<usize>,
}

pub struct ShapeUtil;

impl ShapeUtil {
    pub fn triangulate_polygon(polygon: &[Vec2]) -> TriangulateResult {
        let vertices = clean_polygon(polygon);
        if vertices.len() < 3 {
            return TriangulateResult::default();
        }

        let mut order: Vec<usize> = (0..vertices.len()).collect();
        if signed_area(&vertices) < 0.0 {
            order.reverse();
        }

        let mut indices = Vec::new();
        let mut guard = 0usize;

        while order.len() > 3 && guard < vertices.len() * vertices.len() {
            guard += 1;
            let n = order.len();
            let mut ear_found = false;

            for i in 0..n {
                let i0 = order[(i + n - 1) % n];
                let i1 = order[i];
                let i2 = order[(i + 1) % n];

                if !is_convex(vertices[i0], vertices[i1], vertices[i2]) {
                    continue;
                }

                let mut contains_other = false;
                for &j in &order {
                    if j == i0 || j == i1 || j == i2 {
                        continue;
                    }
                    if point_in_triangle(vertices[j], vertices[i0], vertices[i1], vertices[i2]) {
                        contains_other = true;
                        break;
                    }
                }

                if contains_other {
                    continue;
                }

                indices.extend_from_slice(&[i0, i1, i2]);
                order.remove(i);
                ear_found = true;
                break;
            }

            if !ear_found {
                indices.clear();
                for i in 1..vertices.len() - 1 {
                    indices.extend_from_slice(&[0, i, i + 1]);
                }
                return TriangulateResult { vertices, indices };
            }
        }

        if order.len() == 3 {
            indices.extend_from_slice(&[order[0], order[1], order[2]]);
        }

        TriangulateResult { vertices, indices }
    }

    pub fn distance_segment_and_point(segment: &[Vec2], point: Vec2) -> f32 {
        if segment.len() < 2 {
            return f32::MAX;
        }

        let mut distance = f32::MAX;

        for pair in segment.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let ab = b - a;
            let ap = point - a;
            let bp = point - b;

            if ab.is_zero_approx() {
                distance = distance.min(ap.length());
            } else if ap.dot(ab) < 0.0 {
                distance = distance.min(ap.length());
            } else if bp.dot(ab) > 0.0 {
                distance = distance.min(bp.length());
            } else {
                distance = distance.min(ap.cross(ab).abs() / ab.length());
            }
        }

        distance
    }
}

fn clean_polygon(points: &[Vec2]) -> Vec<Vec2> {
    let mut out = Vec::new();
    for &p in points {
        if out.last().map(|last: &Vec2| (*last - p).length_squared() > 0.0001).unwrap_or(true) {
            out.push(p);
        }
    }
    if out.len() >= 2 && (out[0] - *out.last().unwrap()).length_squared() <= 0.0001 {
        out.pop();
    }
    out
}

fn signed_area(points: &[Vec2]) -> f32 {
    let mut area = 0.0;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        area += a.x * b.y - b.x * a.y;
    }
    area * 0.5
}

fn is_convex(a: Vec2, b: Vec2, c: Vec2) -> bool {
    (b - a).cross(c - b) > 0.00001
}

fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let c1 = (b - a).cross(p - a);
    let c2 = (c - b).cross(p - b);
    let c3 = (a - c).cross(p - c);
    c1 >= -0.00001 && c2 >= -0.00001 && c3 >= -0.00001
}
