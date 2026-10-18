//! Animated diagonal stripes stay inside the completed part of AI job progress.
use egui::{Pos2, Response, Stroke, Ui, pos2};

/// Show real progress, retaining a small animated cap while waiting for the first result.
/// Pause retains the last stripe phase; completion removes stripes and redraw requests.
pub(crate) fn ai_progress(ui: &mut Ui, progress: f32, width: f32, active: bool) -> Response {
    let progress = if progress.is_finite() { progress.clamp(0.0, 1.0) } else { 0.0 };
    let width = if width.is_finite() { width.clamp(32.0, 2048.0) } else { 300.0 };
    let tokens = crate::theme::Tokens::get(ui.ctx());
    let response = ui.add(egui::ProgressBar::new(progress).desired_width(width).fill(tokens.accent));
    if progress >= 1.0 || !ui.is_rect_visible(response.rect) {
        return response;
    }
    let phase_id = response.id.with("ai-stripe-phase");
    let phase = if active {
        let phase = ui.input(|i| (i.time / 1.1).rem_euclid(1.0)) as f32;
        ui.data_mut(|d| d.insert_temp(phase_id, phase));
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(40));
        phase
    } else {
        ui.data(|d| d.get_temp::<f32>(phase_id)).unwrap_or(0.0)
    };
    let rect = response.rect;
    let radius = rect.height() / 2.0;
    let filled_width = (rect.width() * progress).max(rect.height()).min(rect.width());
    let right = rect.left() + filled_width;
    let mut pill = Vec::with_capacity(26);
    for (cx, start) in [(right - radius, -std::f32::consts::FRAC_PI_2), (rect.left() + radius, std::f32::consts::FRAC_PI_2)] {
        for i in 0..=12 {
            let angle = start + i as f32 * std::f32::consts::PI / 12.0;
            pill.push(pos2(cx + radius * angle.cos(), rect.center().y + radius * angle.sin()));
        }
    }
    // Geometry avoids repeat-sampler differences between normal and predictable GPU filtering.
    let period = 24.0;
    let start = rect.left() - rect.height() - period + phase * period;
    for i in 0..128 {
        let x = start + i as f32 * period;
        if x > right + rect.height() {
            break;
        }
        let stripe = vec![
            pos2(x, rect.top()),
            pos2(x + period * 0.45, rect.top()),
            pos2(x + period * 0.45 - rect.height(), rect.bottom()),
            pos2(x - rect.height(), rect.bottom()),
        ];
        let stripe = clip(stripe, &pill);
        if stripe.len() >= 3 {
            ui.painter().add(egui::Shape::convex_polygon(stripe, tokens.text.linear_multiply(0.24), Stroke::NONE));
        }
    }
    response
}

/// Intersect two convex polygons; the rounded fill outline has positive winding.
fn clip(mut polygon: Vec<Pos2>, outline: &[Pos2]) -> Vec<Pos2> {
    for (a, b) in outline.iter().zip(outline.iter().cycle().skip(1)).take(outline.len()) {
        let Some(mut previous) = polygon.last().copied() else { break };
        let side = |p: Pos2| (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
        let mut previous_side = side(previous);
        let mut next = Vec::with_capacity(polygon.len() + 1);
        for current in polygon.iter().copied() {
            let current_side = side(current);
            if (previous_side >= 0.0) != (current_side >= 0.0) {
                let denominator = previous_side - current_side;
                if denominator.abs() > f32::EPSILON {
                    next.push(previous + (current - previous) * (previous_side / denominator).clamp(0.0, 1.0));
                }
            }
            if current_side >= 0.0 {
                next.push(current);
            }
            previous = current;
            previous_side = current_side;
        }
        polygon = next;
    }
    polygon
}
