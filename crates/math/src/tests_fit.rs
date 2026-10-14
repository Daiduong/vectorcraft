//! Curve fitting: golden error bounds against the functions, and where curves split.

use super::eval::parse_expr;
use super::fit::*;
use std::f64::consts::PI;
use vectorcraft_geom::{CubicBez, ParamCurve, Point};

fn graph(src: &str, range: (f64, f64), n: usize, clip: Option<(f64, f64)>) -> Runs {
    let f = parse_expr(src, &["x"]).unwrap_or_else(|e| panic!("{src}: {e}"));
    fit_function(&f, "x", range, n, FitOptions { clip })
}

/// The largest |y − f(x)| over points along the cubics (`steps` per cubic).
fn max_error(runs: &Runs, f: impl Fn(f64) -> f64, steps: usize) -> f64 {
    let mut worst = 0.0f64;
    for c in runs.iter().flatten() {
        for k in 0..=steps {
            let p = c.eval(k as f64 / steps as f64);
            worst = worst.max((p.y - f(p.x)).abs());
        }
    }
    worst
}

fn connected(run: &[CubicBez]) -> bool {
    run.windows(2).all(|w| w[0].p3 == w[1].p0)
}

fn x_extent(run: &[CubicBez]) -> (f64, f64) {
    let xs = run.iter().flat_map(|c| [c.p0.x, c.p3.x]);
    (xs.clone().fold(f64::INFINITY, f64::min), xs.fold(f64::NEG_INFINITY, f64::max))
}

/// A cubic polynomial is drawn exactly (Hermite cubics with exact slopes).
#[test]
fn a_cubic_polynomial_is_exact() {
    let runs = graph("x^3-x", (-1.4, 1.4), 20, None);
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].len(), 20);
    assert!(connected(&runs[0]));
    assert_eq!(runs[0][0].p0, Point::new(-1.4, -1.4f64.powi(3) + 1.4));
    assert_eq!(runs[0][19].p3.x, 1.4);
    let err = max_error(&runs, |x| x * x * x - x, 50);
    assert!(err < 1e-12, "error {err}");
    // The control points are a third of the step in, along the tangent.
    let c = runs[0][0];
    let h = 2.8 / 20.0;
    assert!((c.p1.x - (c.p0.x + h / 3.0)).abs() < 1e-12);
    let slope = 3.0 * 1.4f64.powi(2) - 1.0;
    assert!((c.p1.y - (c.p0.y + slope * h / 3.0)).abs() < 1e-12);
}

/// The guide's precision rule (about four steps per extremum) draws a sine closely.
#[test]
fn a_sine_at_the_recommended_precision_is_close() {
    // sin on [0, 5π/4] has one extremum: four steps.
    let runs = graph(r"\sin x", (0.0, 5.0 * PI / 4.0), 4, None);
    assert_eq!((runs.len(), runs[0].len()), (1, 4));
    let err = max_error(&runs, f64::sin, 100);
    assert!(err < 5e-3, "error {err}");
    let finer = graph(r"\sin x", (0.0, 5.0 * PI / 4.0), 16, None);
    let err16 = max_error(&finer, f64::sin, 100);
    assert!(err16 < 2e-5 && err16 < err / 50.0, "error {err16}");
}

/// 1/sin x has poles at −π, 0 and π in (−5, 5): four pieces, none across a pole, all inside the
/// band. 0 is a sample (not a number there); π is not one (a pole between samples).
#[test]
fn a_cosecant_splits_at_its_asymptotes() {
    for clip in [Some((-4.0, 4.0)), None] {
        let runs = graph(r"\frac{1}{\sin x}", (-5.0, 5.0), 40, clip);
        assert_eq!(runs.len(), 4, "{clip:?}: {:?}", runs.iter().map(|r| x_extent(r)).collect::<Vec<_>>());
        for run in &runs {
            assert!(connected(run));
            let (lo, hi) = x_extent(run);
            for pole in [-PI, 0.0, PI] {
                assert!(!(lo < pole && pole < hi), "{clip:?}: a run over [{lo}, {hi}] crosses the pole at {pole}");
            }
        }
        let ends: Vec<f64> = runs.iter().flat_map(|r| [r[0].p0.y, r[r.len() - 1].p3.y]).collect();
        if let Some((lo, hi)) = clip {
            assert!(runs.iter().flatten().all(|c| [c.p0, c.p3].iter().all(|p| p.y >= lo - 1e-9 && p.y <= hi + 1e-9)));
            // The pieces reach the band's edges at the poles.
            assert!(ends.iter().filter(|y| (y.abs() - 4.0).abs() < 1e-6).count() >= 6, "{ends:?}");
            let err = max_error(&runs, |x| 1.0 / x.sin(), 40);
            assert!(err < 0.05, "error {err}");
        } else {
            // Unclipped, the pieces still end far up the asymptotes, not at a jump across them.
            assert!(ends.iter().filter(|y| y.abs() > 100.0).count() >= 6, "{ends:?}");
        }
    }
}

#[test]
fn a_tangent_clipped_to_a_band() {
    let runs = graph(r"\tan x", (0.0, 3.0), 12, Some((-5.0, 5.0)));
    assert_eq!(runs.len(), 2);
    let (_, end0) = x_extent(&runs[0]);
    let (start1, _) = x_extent(&runs[1]);
    assert!((end0 - 5f64.atan()).abs() < 1e-9, "{end0}");
    assert!((start1 - (PI - 5f64.atan())).abs() < 1e-9, "{start1}");
    // Steep beside the pole: the precision shows there.
    let coarse = max_error(&runs, f64::tan, 40);
    let fine = max_error(&graph(r"\tan x", (0.0, 3.0), 48, Some((-5.0, 5.0))), f64::tan, 40);
    assert!(coarse < 0.1 && fine < 0.01, "{coarse} {fine}");
}

/// A curve that leaves the band and comes back is cut at both crossings.
#[test]
fn a_parabola_leaving_the_band_is_cut_where_it_crosses() {
    let runs = graph("x^2", (-3.0, 3.0), 6, Some((-1.0, 4.0)));
    assert_eq!(runs.len(), 1);
    let (lo, hi) = x_extent(&runs[0]);
    assert!((lo + 2.0).abs() < 1e-9 && (hi - 2.0).abs() < 1e-9, "[{lo}, {hi}]");
    assert!(max_error(&runs, |x| x * x, 40) < 1e-9);
    // Its top dips into a band above it between samples… and out of one below it.
    let runs = graph("4-x^2", (-3.0, 3.0), 3, Some((-10.0, 3.0)));
    assert_eq!(runs.len(), 2, "{:?}", runs.iter().map(|r| x_extent(r)).collect::<Vec<_>>());
}

/// Where the function stops being a number, the curve stops at the edge of its domain; a
/// vertical tangent there comes out vertical.
#[test]
fn domains_end_curves_and_vertical_tangents_stay_vertical() {
    let runs = graph(r"\sqrt x", (-1.0, 4.0), 5, None);
    assert_eq!(runs.len(), 1);
    let first = runs[0][0];
    assert!(first.p0.x.abs() < 1e-9 && first.p0.y.abs() < 1e-4, "{first:?}");
    let runs = graph(r"\sqrt x", (0.0, 4.0), 4, None);
    let first = runs[0][0];
    assert_eq!(first.p0, Point::ZERO);
    assert!(first.p1.x.abs() < 1e-12 && first.p1.y > 0.0, "vertical tangent: {first:?}");
    assert!(max_error(&runs, f64::sqrt, 50) < 0.05);
    // ln x at 0 is −∞: the curve starts inside the band.
    let runs = graph(r"\ln x", (0.0, 3.0), 6, Some((-3.0, 3.0)));
    assert_eq!(runs.len(), 1);
    assert!((runs[0][0].p0.y + 3.0).abs() < 1e-6 && runs[0][0].p0.x > 0.0);
    // Nowhere a number: nothing.
    assert!(graph(r"\sqrt{-1-x^2}", (-2.0, 2.0), 8, None).is_empty());
}

#[test]
fn bad_ranges_and_counts_are_tolerated() {
    assert!(graph("x", (f64::NAN, 1.0), 4, None).is_empty());
    assert!(graph("x", (0.0, f64::INFINITY), 4, None).is_empty());
    assert!(graph("x", (1.0, 1.0), 4, None).is_empty());
    assert_eq!(graph("x", (0.0, 1.0), 0, None)[0].len(), 1);
    assert_eq!(graph("x", (1.0, 0.0), 2, None)[0][0].p0.x, 0.0, "a reversed range is put in order");
    assert_eq!(graph("x", (0.0, 1.0), usize::MAX, None)[0].len(), MAX_SEGMENTS);
    let flat = graph("x", (0.0, 1.0), 2, Some((f64::NAN, 3.0)));
    assert_eq!(flat[0].len(), 2, "a clip that isn't numbers is ignored");
}

#[test]
fn a_parametric_circle_is_round_and_closed() {
    let x = parse_expr(r"\cos t", &["t"]).unwrap();
    let y = parse_expr(r"\sin t", &["t"]).unwrap();
    let runs = fit_parametric(&x, &y, "t", (0.0, 2.0 * PI), 16);
    assert_eq!((runs.len(), runs[0].len()), (1, 16));
    let mut worst = 0.0f64;
    for c in &runs[0] {
        for k in 0..=20 {
            worst = worst.max((c.eval(k as f64 / 20.0).to_vec2().hypot() - 1.0).abs());
        }
    }
    assert!(worst < 1e-4, "radius error {worst}");
    assert!((runs[0][15].p3 - runs[0][0].p0).hypot() < 1e-12);
}

#[test]
fn a_parametric_curve_breaks_at_poles_and_gaps() {
    let x = parse_expr("t", &["t"]).unwrap();
    let y = parse_expr(r"\frac{1}{t}", &["t"]).unwrap();
    for n in [10, 9] {
        let runs = fit_parametric(&x, &y, "t", (-1.0, 1.0), n);
        assert_eq!(runs.len(), 2, "n = {n}");
        assert!(runs.iter().all(|r| {
            let (lo, hi) = x_extent(r);
            !(lo < 0.0 && 0.0 < hi)
        }));
    }
}

#[test]
fn samples_are_joined_smoothly_and_split_at_gaps() {
    let pts: Vec<(f64, f64)> = (0..=10).map(|i| (i as f64 / 10.0, (i as f64 / 10.0).powi(2))).collect();
    let runs = fit_samples(&pts);
    assert_eq!((runs.len(), runs[0].len()), (1, 10));
    assert!(connected(&runs[0]));
    assert!(max_error(&runs, |x| x * x, 20) < 1e-12, "a parabola sampled evenly is exact");
    let mut gappy = pts.clone();
    gappy[5].1 = f64::NAN;
    let runs = fit_samples(&gappy);
    assert_eq!(runs.iter().map(Vec::len).collect::<Vec<_>>(), vec![4, 4]);
    let line = fit_samples(&[(0.0, 0.0), (1.0, 1.0)]);
    let c = line[0][0];
    assert_eq!((line.len(), line[0].len(), c.p0, c.p3), (1, 1, Point::ZERO, Point::new(1.0, 1.0)));
    assert!((c.p1 - Point::new(1.0 / 3.0, 1.0 / 3.0)).hypot() < 1e-12 && (c.p2 - Point::new(2.0 / 3.0, 2.0 / 3.0)).hypot() < 1e-12);
    assert!(fit_samples(&[(0.0, 0.0)]).is_empty());
    assert!(fit_samples(&[]).is_empty());
}

#[test]
fn runs_become_one_path_with_a_subpath_each() {
    let runs = graph(r"\frac{1}{x}", (-1.0, 1.0), 8, Some((-5.0, 5.0)));
    assert_eq!(runs.len(), 2);
    let path = to_bezpath(&runs);
    let moves = path.elements().iter().filter(|e| matches!(e, vectorcraft_geom::PathEl::MoveTo(_))).count();
    assert_eq!(moves, 2);
}
