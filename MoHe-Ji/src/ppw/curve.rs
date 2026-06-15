use super::{path::PPWPath, polygon::PPWPolygon, vec2::Vec2};

pub struct PPWCurve;

impl PPWCurve {
    const POINTS_PER_SEGMENT: usize = 50;
    const PSI_INFINITY: f32 = 2.0;
    const NEWTON_EPSILON: f64 = 0.001;
    const MAX_NEWTON_ITERATION: usize = 300;

    fn calc_rational_bezier_point(p0: Vec2, p1: Vec2, p2: Vec2, w: f32, t: f32) -> Vec2 {
        ((1.0 - t) * (1.0 - t) * p0 + 2.0 * w * t * (1.0 - t) * p1 + t * t * p2)
            / ((1.0 - t) * (1.0 - t) + 2.0 * w * t * (1.0 - t) + t * t)
    }

    fn calc_max_curvature_p1(cp0: Vec2, cp1: Vec2, cp2: Vec2, w: f32) -> (Vec2, f32) {
        if (cp0 - cp2).is_zero_approx() {
            return (cp0, 0.5);
        }
        if (cp1 - cp0).is_zero_approx() {
            return (cp1, 0.0);
        }
        if (cp1 - cp2).is_zero_approx() {
            return (cp1, 1.0);
        }

        let w64 = w as f64;
        let p = cp0 - cp1;
        let r = cp2 - cp1;
        let p_length = p.length() as f64;
        let r_length = r.length() as f64;
        let p_sqr_length = p_length * p_length;
        let r_sqr_length = r_length * r_length;
        let p_plus_r_sqr_length = (p + r).length_squared() as f64;
        let p_minus_r_sqr_length = (p - r).length_squared() as f64;
        let p_dot_r = p.dot(r) as f64;
        let ww = w64 * w64;

        let alpha_beta_to_q_t = |alpha: f64, beta: f64| -> (Vec2, f32) {
            let l0 = (alpha + beta) / 2.0;
            let l2 = (alpha - beta) / 2.0;
            let l1 = 1.0 - alpha;

            if l1.abs() < 1.0e-9 {
                return (cp1, 0.5);
            }

            let q = (cp1 - l0 as f32 * cp0 - l2 as f32 * cp2) / l1 as f32;
            let sqrt = if l0 * l2 < 0.0 { 0.0 } else { (l0 * l2).sqrt() };
            let denominator = 2.0 * w64 * (alpha + 2.0 * sqrt);
            if denominator.abs() < 1.0e-9 {
                return (q, 0.5);
            }

            let t = (2.0 * w64 * l2 + l1) / denominator;
            (q, t.clamp(0.0, 1.0) as f32)
        };

        if (p_sqr_length - r_sqr_length).abs() < 0.0001 {
            return alpha_beta_to_q_t(1.0 / (1.0 + w64), 0.0);
        }

        if (p_dot_r.abs() - p_length * r_length).abs() < 0.0001 {
            let m = (cp0 + cp2) / 2.0;
            let sign = (r_length - p_length).signum();
            let denom = (cp1 - m).length() as f64;
            if denom < 1.0e-9 {
                return (cp1, 0.5);
            }
            let k = sign * (cp0 - m).length() as f64 / denom;

            if p_dot_r < 0.0 {
                let inner = 1.0 - ww * (1.0 - k * k);
                if inner <= 0.0 {
                    return (cp1, 0.5);
                }
                let beta = sign / inner.sqrt();
                let alpha = (beta * (beta + k)) / (1.0 + k * beta);
                return alpha_beta_to_q_t(alpha, beta);
            } else {
                let inner = 1.0 - k * k;
                if inner < 0.0 {
                    return (cp1, 0.5);
                }
                let dby = 1.0 + w64 * inner.sqrt();
                let alpha = 1.0 / dby;
                let beta = k / dby;
                return alpha_beta_to_q_t(alpha, beta);
            }
        }

        let (mut alpha, mut beta) = if w64 <= 1.0 {
            let now_a = 1.0 / (1.0 + w64);
            let tmp_b = (p_sqr_length - r_sqr_length) * (1.0 - 2.0 * now_a)
                / 3.0
                / p_plus_r_sqr_length;
            (now_a, tmp_b.clamp(-now_a, now_a))
        } else {
            let chord = 2.0 * r_length / (p_length + r_length) - 1.0;
            let now_b = 2.0 / (1.0 + (-2.0 * chord / w64).exp()) - 1.0;
            let inner = ww / (1.0 - ww).powi(2) - ww * now_b * now_b / (1.0 - ww);
            let now_a = 1.0 / (1.0 - ww) + inner.abs().sqrt();
            (now_a, now_b)
        };

        let f = |a: f64, b: f64| (1.0 - ww) * a * a - 2.0 * a + 1.0 + ww * b * b;
        let g = |a: f64, b: f64| {
            p_plus_r_sqr_length * b * b * b
                - (p_sqr_length - r_sqr_length) * (1.0 - 2.0 * a) * b * b
                + (p_minus_r_sqr_length * a - 2.0 * (p_sqr_length + r_sqr_length)) * a * b
                - (p_sqr_length - r_sqr_length) * a * a
        };
        let fda = |a: f64| 2.0 * (1.0 - ww) * a - 2.0;
        let fdb = |b: f64| 2.0 * ww * b;
        let gda = |a: f64, b: f64| {
            2.0 * (p_sqr_length - r_sqr_length) * b * b
                + 2.0 * b * p_minus_r_sqr_length * a
                - 2.0 * b * (p_sqr_length + r_sqr_length)
                - 2.0 * (p_sqr_length - r_sqr_length) * a
        };
        let gdb = |a: f64, b: f64| {
            3.0 * p_plus_r_sqr_length * b * b
                - 2.0 * (p_sqr_length - r_sqr_length) * (1.0 - 2.0 * a) * b
                + (p_minus_r_sqr_length * a - 2.0 * (p_sqr_length + r_sqr_length)) * a
        };

        for _ in 0..Self::MAX_NEWTON_ITERATION {
            let rf = f(alpha, beta);
            let rg = g(alpha, beta);
            if rf.abs() < Self::NEWTON_EPSILON && rg.abs() < Self::NEWTON_EPSILON {
                break;
            }

            let det = fda(alpha) * gdb(alpha, beta) - fdb(beta) * gda(alpha, beta);
            if det.abs() < 1.0e-12 {
                break;
            }

            alpha -= (rf * gdb(alpha, beta) - rg * fdb(beta)) / det;
            beta -= (-rf * gda(alpha, beta) + rg * fda(alpha)) / det;

            if !alpha.is_finite() || !beta.is_finite() {
                return (cp1, 0.5);
            }
        }

        alpha_beta_to_q_t(alpha, beta)
    }

    fn blend_coefficient(t: f32, phi: f32, psi: f32) -> (f32, f32) {
        let phi = phi.max(0.0001);
        let ephi = phi.exp();
        let sigma = 2.0 / (1.0 + ephi);
        let inside = (ephi + 1.0).sqrt() * (ephi - 1.0).max(0.0).sqrt();
        let delta = -0.5 * (ephi - inside).max(0.000001).ln();

        let t2 = if psi <= -Self::PSI_INFINITY {
            0.0
        } else if psi >= Self::PSI_INFINITY {
            1.0
        } else {
            t / ((-psi).exp() * (1.0 - t) + t)
        };

        let t3 = delta * (2.0 * t2 - 1.0);
        let ht = t3.tanh() - sigma * t3;
        let ha = delta.tanh() - sigma * delta;

        if ha.abs() < 0.000001 {
            return (1.0 - t, t);
        }

        let b1 = (1.0 - ht / ha) / 2.0;
        let b2 = 1.0 - b1;
        (b1, b2)
    }

    fn calc_segment(
        cp0: Vec2,
        cp1: Vec2,
        cp2: Vec2,
        cp3: Vec2,
        w1: f32,
        w2: f32,
        phi: f32,
        psi: f32,
    ) -> Vec<Vec2> {
        let mut curve1 = [Vec2::ZERO; Self::POINTS_PER_SEGMENT];
        let (p1, tt) = Self::calc_max_curvature_p1(cp0, cp1, cp2, w1);
        for (i, out) in curve1.iter_mut().enumerate() {
            let t = tt + (1.0 - tt) * i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            *out = Self::calc_rational_bezier_point(cp0, p1, cp2, w1, t);
        }

        let mut curve2 = [Vec2::ZERO; Self::POINTS_PER_SEGMENT];
        let (p2, tt) = Self::calc_max_curvature_p1(cp1, cp2, cp3, w2);
        for (i, out) in curve2.iter_mut().enumerate() {
            let t = tt * i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            *out = Self::calc_rational_bezier_point(cp1, p2, cp3, w2, t);
        }

        let mut segment = Vec::with_capacity(Self::POINTS_PER_SEGMENT);
        for i in 0..Self::POINTS_PER_SEGMENT {
            let t = i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            let (b1, b2) = Self::blend_coefficient(t, phi, psi);
            segment.push(b1 * curve1[i] + b2 * curve2[i]);
        }
        segment
    }

    fn calc_start_segment(cp1: Vec2, cp2: Vec2, cp3: Vec2, w2: f32, phi: f32, psi: f32) -> Vec<Vec2> {
        let mut curve1 = [Vec2::ZERO; Self::POINTS_PER_SEGMENT];
        for (i, out) in curve1.iter_mut().enumerate() {
            let t = i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            *out = cp2 * t + (1.0 - t) * cp1;
        }

        let mut curve2 = [Vec2::ZERO; Self::POINTS_PER_SEGMENT];
        let (p2, tt) = Self::calc_max_curvature_p1(cp1, cp2, cp3, w2);
        for (i, out) in curve2.iter_mut().enumerate() {
            let t = tt * i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            *out = Self::calc_rational_bezier_point(cp1, p2, cp3, w2, t);
        }

        let mut segment = Vec::with_capacity(Self::POINTS_PER_SEGMENT);
        for i in 0..Self::POINTS_PER_SEGMENT {
            let t = i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            let (b1, b2) = Self::blend_coefficient(t, phi, psi);
            segment.push(b1 * curve1[i] + b2 * curve2[i]);
        }
        segment
    }

    fn calc_end_segment(cp0: Vec2, cp1: Vec2, cp2: Vec2, w1: f32, phi: f32, psi: f32) -> Vec<Vec2> {
        let mut curve1 = [Vec2::ZERO; Self::POINTS_PER_SEGMENT];
        let (p1, tt) = Self::calc_max_curvature_p1(cp0, cp1, cp2, w1);
        for (i, out) in curve1.iter_mut().enumerate() {
            let t = tt + (1.0 - tt) * i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            *out = Self::calc_rational_bezier_point(cp0, p1, cp2, w1, t);
        }

        let mut curve2 = [Vec2::ZERO; Self::POINTS_PER_SEGMENT];
        for (i, out) in curve2.iter_mut().enumerate() {
            let t = i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            *out = cp2 * t + (1.0 - t) * cp1;
        }

        let mut segment = Vec::with_capacity(Self::POINTS_PER_SEGMENT);
        for i in 0..Self::POINTS_PER_SEGMENT {
            let t = i as f32 / (Self::POINTS_PER_SEGMENT - 1) as f32;
            let (b1, b2) = Self::blend_coefficient(t, phi, psi);
            segment.push(b1 * curve1[i] + b2 * curve2[i]);
        }
        segment
    }

    pub fn convert(path: &PPWPath) -> PPWPolygon {
        if !path.validate() {
            eprintln!("Invalid PPWPath: {path:?}");
            return PPWPolygon::empty();
        }

        let cp_len = path.control_points.len();

        if cp_len == 1 {
            return PPWPolygon::empty();
        }

        if cp_len == 2 {
            let segment = vec![path.control_points[0], path.control_points[1]];
            return PPWPolygon {
                polygon: segment.clone(),
                segments: vec![segment],
            };
        }

        let mut segments = Vec::new();

        if path.is_closed {
            for i in 0..cp_len {
                let cp0 = path.control_points[(i + cp_len - 1) % cp_len];
                let cp1 = path.control_points[i];
                let cp2 = path.control_points[(i + 1) % cp_len];
                let cp3 = path.control_points[(i + 2) % cp_len];
                let w1 = path.weights[i % cp_len];
                let w2 = path.weights[(i + 1) % cp_len];
                let phi = path.phis[i];
                let psi = path.psis[i];
                segments.push(Self::calc_segment(cp0, cp1, cp2, cp3, w1, w2, phi, psi));
            }
        } else {
            segments.push(Self::calc_start_segment(
                path.control_points[0],
                path.control_points[1],
                path.control_points[2],
                path.weights[1],
                path.phis[0],
                path.psis[0],
            ));

            for i in 1..(cp_len - 2) {
                let cp0 = path.control_points[i - 1];
                let cp1 = path.control_points[i];
                let cp2 = path.control_points[i + 1];
                let cp3 = path.control_points[i + 2];
                let w1 = path.weights[i];
                let w2 = path.weights[i + 1];
                let phi = path.phis[i];
                let psi = path.psis[i];
                segments.push(Self::calc_segment(cp0, cp1, cp2, cp3, w1, w2, phi, psi));
            }

            segments.push(Self::calc_end_segment(
                path.control_points[cp_len - 3],
                path.control_points[cp_len - 2],
                path.control_points[cp_len - 1],
                path.weights[cp_len - 2],
                path.phis[cp_len - 2],
                path.psis[cp_len - 2],
            ));
        }

        let polygon = segments.iter().flat_map(|s| s.iter().copied()).collect();
        PPWPolygon { polygon, segments }
    }
}
