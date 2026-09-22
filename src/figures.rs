//! `figures`: pictures of what records hold, drawn by the tool, so no script keeps a second copy
//! of the arithmetic.
//!
//! One figure for now, the block means: every run of an invocation as a line of its block means
//! against when each block ran, so the picture shows what the trimmed mean and its claim
//! summarize. A panel per bench and invocation, since two benches' scales flatten each other on
//! one axis and a run number repeats in every invocation.
//!
//! The SVG is written by hand, on a white ground. A `.png` output rasterizes it with `resvg`
//! and an embedded font, Liberation Sans, so a host with no fonts of its own draws the same
//! labels.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::record::{self, AnalyzedRun, Skipped};
use crate::series::{Trim, Trimmed};

/// Which runs of an invocation a panel draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Show {
    /// Every run.
    All,
    /// Every run, the ones the trim keeps in colour and the ones it drops in faint grey.
    Trim,
    /// The fastest and the slowest run by mean.
    Extremes,
    /// These runs by number, coloured in the order given.
    Runs(Vec<u64>),
}

impl Show {
    /// Parse `all`, `trim`, `extremes`, or a list of run numbers, `3,1,7`.
    pub fn parse(s: &str) -> Result<Show, String> {
        match s {
            "all" => Ok(Show::All),
            "trim" => Ok(Show::Trim),
            "extremes" => Ok(Show::Extremes),
            list => {
                let runs: Result<Vec<u64>, _> =
                    list.split(',').map(|r| r.trim().parse::<u64>()).collect();
                match runs {
                    Ok(runs) if !runs.is_empty() && !runs.contains(&0) => Ok(Show::Runs(runs)),
                    _ => Err(format!(
                        "'{s}' is not all, trim, extremes, or run numbers, as in 3,1,7"
                    )),
                }
            }
        }
    }
}

/// The x axis: when each block ran, or its number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XAxis {
    /// Seconds from the warm's start, the block sleeps included, so a drift in wall time shows.
    Time,
    /// The block's number, so runs line up block for block.
    Block,
}

impl XAxis {
    /// Parse `time` or `block`.
    pub fn parse(s: &str) -> Result<XAxis, String> {
        match s {
            "time" => Ok(XAxis::Time),
            "block" => Ok(XAxis::Block),
            _ => Err(format!("'{s}' is not time or block")),
        }
    }
}

/// What the `figures` command draws.
pub struct Plan {
    /// Benches to draw, all when empty.
    pub benches: Vec<String>,
    /// Which runs.
    pub show: Show,
    /// The x axis.
    pub x: XAxis,
    /// The one invocation to draw, every one when `None`.
    pub series: Option<String>,
    /// The trim that `Show::Trim` and the trimmed-mean line use.
    pub trim: Trim,
}

/// One line of a panel: a run, its colour, and its points.
struct Line {
    run: u64,
    colour: &'static str,
    /// Dashed, for a colour's second use past the palette.
    dashed: bool,
    points: Vec<(f64, f64)>,
}

/// One panel: a bench's invocation.
struct Panel {
    title: String,
    x_label: &'static str,
    lines: Vec<Line>,
    /// The invocation's trimmed mean, drawn dashed across the panel.
    trimmed: Option<f64>,
}

/// Ten colours distinguishable on white, Tableau's ten.
const PALETTE: [&str; 10] = [
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948", "#b07aa1", "#ff9da7",
    "#9c755f", "#79706e",
];

/// A run the trim drops.
const DROPPED: &str = "#d0d0d0";

/// The run's points: each block mean against its seam's time in seconds, or against its number
/// when the times cannot pair with the means, the clock unread or the blocks merged past the
/// point cap.
fn points(run: &AnalyzedRun, x: XAxis) -> (Vec<(f64, f64)>, bool) {
    let timed =
        x == XAxis::Time && run.block_agg == 1 && run.clock_t_ns.len() == run.block_mean_ns.len();
    let pts = run
        .block_mean_ns
        .iter()
        .enumerate()
        .map(|(i, &m)| {
            let x = if timed {
                run.clock_t_ns[i] as f64 / 1e9
            } else {
                (i + 1) as f64
            };
            (x, m)
        })
        .collect();
    (pts, timed)
}

/// The runs of one invocation that `show` picks, in the order it draws them, each with its
/// colour, the trim's dropped runs grey under `Show::Trim`.
fn lines(runs: &[AnalyzedRun], plan: &Plan) -> (Vec<Line>, bool) {
    let means: Vec<f64> = runs.iter().map(|r| r.mean_ns).collect();
    let dropped: Vec<usize> = match Trimmed::of(&means, plan.trim) {
        Some(t) => t.dropped_low.into_iter().chain(t.dropped_high).collect(),
        None => Vec::new(),
    };
    let picked: Vec<usize> = match &plan.show {
        Show::All | Show::Trim => (0..runs.len()).collect(),
        Show::Extremes => {
            let mut order: Vec<usize> = (0..runs.len()).collect();
            order.sort_by(|&a, &b| means[a].total_cmp(&means[b]));
            match (order.first(), order.last()) {
                (Some(&lo), Some(&hi)) if lo != hi => vec![lo, hi],
                (Some(&lo), _) => vec![lo],
                _ => Vec::new(),
            }
        }
        Show::Runs(list) => list
            .iter()
            .filter_map(|n| runs.iter().position(|r| r.run == *n))
            .collect(),
    };
    let mut all_timed = true;
    let mut out = Vec::new();
    for (k, &i) in picked.iter().enumerate() {
        let (pts, timed) = points(&runs[i], plan.x);
        all_timed &= timed;
        let grey = plan.show == Show::Trim && dropped.contains(&i);
        out.push(Line {
            run: runs[i].run,
            colour: if grey {
                DROPPED
            } else {
                PALETTE[k % PALETTE.len()]
            },
            dashed: !grey && k >= PALETTE.len(),
            points: pts,
        });
    }
    // Grey first, so the kept runs draw over them.
    out.sort_by_key(|l| l.colour != DROPPED);
    (out, all_timed)
}

/// A tick step for a span, 1, 2, or 5 times a power of ten, giving about five ticks.
fn tick_step(span: f64) -> f64 {
    if span <= 0.0 || !span.is_finite() {
        return 1.0;
    }
    let raw = span / 5.0;
    let pow = 10f64.powf(raw.log10().floor());
    let unit = raw / pow;
    let nice = if unit < 1.5 {
        1.0
    } else if unit < 3.5 {
        2.0
    } else if unit < 7.5 {
        5.0
    } else {
        10.0
    };
    nice * pow
}

/// The ticks from `lo` to `hi`, each a multiple of the step.
fn ticks(lo: f64, hi: f64) -> Vec<f64> {
    let step = tick_step(hi - lo);
    let mut t = (lo / step).ceil() * step;
    let mut out = Vec::new();
    while t <= hi + step * 1e-9 {
        out.push(t);
        t += step;
    }
    out
}

/// A tick label: as few decimals as the step needs.
fn tick_label(v: f64, step: f64) -> String {
    let decimals = if step >= 1.0 {
        0
    } else {
        (-step.log10().floor()) as usize
    };
    format!("{v:.decimals$}")
}

/// Text made safe for SVG.
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Panel size and margins, px.
const W: f64 = 900.0;
const H: f64 = 320.0;
const LEFT: f64 = 70.0;
const RIGHT: f64 = 150.0;
const TOP: f64 = 36.0;
const BOTTOM: f64 = 46.0;

/// The panels as one SVG, stacked.
fn svg(panels: &[Panel]) -> String {
    let total_h = H * panels.len() as f64;
    let mut out = String::new();
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{total_h}" viewBox="0 0 {W} {total_h}" font-family="sans-serif" font-size="12">"#
    );
    let _ = writeln!(out, r#"<rect width="100%" height="100%" fill="white"/>"#);
    for (p, panel) in panels.iter().enumerate() {
        let y0 = H * p as f64;
        let pts: Vec<(f64, f64)> = panel.lines.iter().flat_map(|l| l.points.clone()).collect();
        if pts.is_empty() {
            continue;
        }
        let (mut xlo, mut xhi) = (f64::INFINITY, f64::NEG_INFINITY);
        let (mut ylo, mut yhi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &(x, y) in &pts {
            xlo = xlo.min(x);
            xhi = xhi.max(x);
            ylo = ylo.min(y);
            yhi = yhi.max(y);
        }
        if let Some(t) = panel.trimmed {
            ylo = ylo.min(t);
            yhi = yhi.max(t);
        }
        let pad = ((yhi - ylo) * 0.05).max(yhi.abs() * 1e-4);
        let (ylo, yhi) = (ylo - pad, yhi + pad);
        let xhi = if xhi > xlo { xhi } else { xlo + 1.0 };
        let (pw, ph) = (W - LEFT - RIGHT, H - TOP - BOTTOM);
        let sx = |x: f64| LEFT + (x - xlo) / (xhi - xlo) * pw;
        let sy = |y: f64| y0 + TOP + (1.0 - (y - ylo) / (yhi - ylo)) * ph;
        let _ = writeln!(
            out,
            r#"<text x="{LEFT}" y="{}" font-size="14" font-weight="bold">{}</text>"#,
            y0 + 22.0,
            escape(&panel.title)
        );
        let _ = writeln!(
            out,
            r##"<rect x="{LEFT}" y="{}" width="{pw}" height="{ph}" fill="none" stroke="#888"/>"##,
            y0 + TOP
        );
        let xstep = tick_step(xhi - xlo);
        for t in ticks(xlo, xhi) {
            let _ = writeln!(
                out,
                r##"<line x1="{x}" y1="{a}" x2="{x}" y2="{b}" stroke="#eee"/><text x="{x}" y="{c}" text-anchor="middle" fill="#444">{l}</text>"##,
                x = sx(t),
                a = y0 + TOP,
                b = y0 + TOP + ph,
                c = y0 + TOP + ph + 16.0,
                l = tick_label(t, xstep)
            );
        }
        let ystep = tick_step(yhi - ylo);
        for t in ticks(ylo, yhi) {
            let _ = writeln!(
                out,
                r##"<line x1="{LEFT}" y1="{y}" x2="{r}" y2="{y}" stroke="#eee"/><text x="{l}" y="{ty}" text-anchor="end" fill="#444">{s}</text>"##,
                y = sy(t),
                r = LEFT + pw,
                l = LEFT - 6.0,
                ty = sy(t) + 4.0,
                s = tick_label(t, ystep)
            );
        }
        let _ = writeln!(
            out,
            r##"<text x="{}" y="{}" text-anchor="middle" fill="#444">{}</text>"##,
            LEFT + pw / 2.0,
            y0 + H - 8.0,
            panel.x_label
        );
        let _ = writeln!(
            out,
            r##"<text x="16" y="{y}" text-anchor="middle" fill="#444" transform="rotate(-90 16 {y})">block mean, ns</text>"##,
            y = y0 + TOP + ph / 2.0
        );
        for line in &panel.lines {
            let path: Vec<String> = line
                .points
                .iter()
                .map(|&(x, y)| format!("{:.1},{:.1}", sx(x), sy(y)))
                .collect();
            let dash = if line.dashed {
                r#" stroke-dasharray="6 3""#
            } else {
                ""
            };
            let _ = writeln!(
                out,
                r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="1.5"{dash}/>"#,
                path.join(" "),
                line.colour
            );
        }
        if let Some(t) = panel.trimmed {
            let _ = writeln!(
                out,
                r##"<line x1="{LEFT}" y1="{y}" x2="{r}" y2="{y}" stroke="#222" stroke-dasharray="4 4"/><text x="{tx}" y="{ty}" fill="#222">trimmed {t:.3}</text>"##,
                y = sy(t),
                r = LEFT + pw,
                tx = LEFT + 6.0,
                ty = sy(t) - 5.0
            );
        }
        // The legend: each run's colour, top to bottom in the order drawn, grey last.
        let mut legend: Vec<&Line> = panel.lines.iter().collect();
        legend.sort_by_key(|l| l.colour == DROPPED);
        for (k, line) in legend.iter().enumerate().take(18) {
            let ly = y0 + TOP + 8.0 + 14.0 * k as f64;
            let label = if line.colour == DROPPED {
                format!("run {} (dropped)", line.run)
            } else {
                format!("run {}", line.run)
            };
            let _ = writeln!(
                out,
                r##"<line x1="{a}" y1="{ly}" x2="{b}" y2="{ly}" stroke="{c}" stroke-width="2"/><text x="{t}" y="{ty}" fill="#444">{label}</text>"##,
                a = LEFT + pw + 6.0,
                b = LEFT + pw + 22.0,
                c = line.colour,
                t = LEFT + pw + 26.0,
                ty = ly + 4.0
            );
        }
    }
    out.push_str("</svg>\n");
    out
}

/// The records of `paths` as panels, a panel per bench and invocation, in bench then session
/// order.
fn panels(runs: Vec<AnalyzedRun>, plan: &Plan) -> Vec<Panel> {
    let mut by: BTreeMap<(String, String), Vec<AnalyzedRun>> = BTreeMap::new();
    for run in runs {
        if !plan.benches.is_empty() && !plan.benches.contains(&run.bench) {
            continue;
        }
        if plan.series.as_ref().is_some_and(|s| *s != run.series) {
            continue;
        }
        by.entry((run.bench.clone(), run.series.clone()))
            .or_default()
            .push(run);
    }
    by.into_iter()
        .map(|((bench, series), mut runs)| {
            runs.sort_by_key(|r| r.run);
            let host = runs[0].host.clone();
            let means: Vec<f64> = runs.iter().map(|r| r.mean_ns).collect();
            let trimmed = Trimmed::of(&means, plan.trim).map(|t| t.mean);
            let (lines, timed) = lines(&runs, plan);
            Panel {
                title: format!("{bench}  {series}  {host}, {} runs", runs.len()),
                x_label: if timed {
                    "seconds from the warm's start"
                } else {
                    "block"
                },
                lines,
                trimmed,
            }
        })
        .collect()
}

/// The font every figure's text is drawn in, embedded so a PNG reads the same on every host. SIL
/// Open Font License 1.1, its text beside it in `assets/fonts`.
const FONT: &[u8] = include_bytes!("../assets/fonts/LiberationSans-Regular.ttf");

/// The format a figure is written in, by its path's extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Svg,
    Png,
}

/// The format `out` names, `.svg` or `.png`.
fn format_of(out: &std::path::Path) -> Result<Format, String> {
    match out.extension().and_then(|e| e.to_str()) {
        Some("svg") => Ok(Format::Svg),
        Some("png") => Ok(Format::Png),
        _ => Err(format!(
            "{}: name a .png or a .svg, the format following the extension",
            out.display()
        )),
    }
}

/// The SVG rasterized to PNG at its own size, its text in [`FONT`].
fn png(svg: &str) -> Result<Vec<u8>, String> {
    let mut opt = resvg::usvg::Options::default();
    let db = opt.fontdb_mut();
    db.load_font_data(FONT.to_vec());
    db.set_sans_serif_family("Liberation Sans");
    let tree = resvg::usvg::Tree::from_str(svg, &opt).map_err(|e| format!("the SVG: {e}"))?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| format!("a {}x{} image", size.width(), size.height()))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap
        .encode_png()
        .map_err(|e| format!("encoding the PNG: {e}"))
}

/// The `figures` command: draw the block means of the records under `paths` into `out`.
/// Returns the exit code.
pub fn run(paths: &[PathBuf], plan: &Plan, out: &std::path::Path) -> i32 {
    if paths.is_empty() {
        eprintln!("error: figures: name the record files or directories to draw");
        return 2;
    }
    let format = match format_of(out) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: figures: --out {e}");
            return 2;
        }
    };
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let mut found: Vec<PathBuf> = match std::fs::read_dir(path) {
                Ok(listing) => listing
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().is_some_and(|x| x == "jsonl"))
                    .collect(),
                Err(e) => {
                    eprintln!("error: figures: reading {}: {e}", path.display());
                    return 1;
                }
            };
            found.sort();
            files.extend(found);
        } else {
            files.push(path.clone());
        }
    }
    let mut skipped = Skipped::default();
    let mut runs = Vec::new();
    for file in &files {
        match record::read_analyzed(file, &mut skipped) {
            Ok(r) => runs.extend(r),
            Err(e) => {
                eprintln!("error: figures: {e}");
                return 1;
            }
        }
    }
    let panels = panels(runs, plan);
    if panels.is_empty() {
        eprintln!("error: figures: no records match the benches and series asked for");
        return 1;
    }
    let text = svg(&panels);
    let bytes = match format {
        Format::Svg => text.into_bytes(),
        Format::Png => match png(&text) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("error: figures: {e}");
                return 1;
            }
        },
    };
    if let Err(e) = std::fs::write(out, bytes) {
        eprintln!("error: figures: writing {}: {e}", out.display());
        return 1;
    }
    println!(
        "figures: wrote {}, {} panel{}",
        out.display(),
        panels.len(),
        if panels.len() == 1 { "" } else { "s" }
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run `n` of `bench` in series `s` whose blocks are `blocks`, clocked a second apart.
    fn run(n: u64, mean: f64, blocks: &[f64]) -> AnalyzedRun {
        AnalyzedRun {
            series: "s".to_string(),
            run: n,
            bench: "b".to_string(),
            host: "h".to_string(),
            tags: BTreeMap::new(),
            t_start: String::new(),
            params: BTreeMap::new(),
            mean_ns: mean,
            block_mean_ns: blocks.to_vec(),
            clock_khz: Vec::new(),
            clock_t_ns: (0..blocks.len() as u64)
                .map(|i| i * 1_000_000_000)
                .collect(),
            block_agg: 1,
        }
    }

    fn plan(show: Show, x: XAxis) -> Plan {
        Plan {
            benches: Vec::new(),
            show,
            x,
            series: None,
            trim: Trim::DEFAULT,
        }
    }

    #[test]
    fn show_parses_its_four_forms() {
        assert_eq!(Show::parse("all"), Ok(Show::All));
        assert_eq!(Show::parse("trim"), Ok(Show::Trim));
        assert_eq!(Show::parse("extremes"), Ok(Show::Extremes));
        assert_eq!(Show::parse("3,1,7"), Ok(Show::Runs(vec![3, 1, 7])));
        assert!(Show::parse("0").is_err() && Show::parse("some").is_err());
        assert_eq!(XAxis::parse("block"), Ok(XAxis::Block));
        assert!(XAxis::parse("t").is_err());
    }

    #[test]
    fn the_runs_shown_follow_the_choice() {
        let runs: Vec<AnalyzedRun> = (1..=10)
            .map(|n| run(n, 100.0 + n as f64, &[1.0, 2.0]))
            .collect();
        let (l, timed) = lines(&runs, &plan(Show::All, XAxis::Time));
        assert_eq!(l.len(), 10);
        assert!(timed);
        assert_eq!(l[1].points[1], (1.0, 2.0), "block two at one second");
        // The 10-50 trim drops run 1 low and runs 6 to 10 high, drawn grey and first.
        let (l, _) = lines(&runs, &plan(Show::Trim, XAxis::Time));
        let grey: Vec<u64> = l
            .iter()
            .filter(|x| x.colour == DROPPED)
            .map(|x| x.run)
            .collect();
        assert_eq!(grey, [1, 6, 7, 8, 9, 10]);
        assert_eq!(l[0].colour, DROPPED);
        let (l, _) = lines(&runs, &plan(Show::Extremes, XAxis::Block));
        assert_eq!(l.iter().map(|x| x.run).collect::<Vec<_>>(), [1, 10]);
        assert_eq!(l[0].points[1], (2.0, 2.0), "block two is x = 2");
        let (l, _) = lines(&runs, &plan(Show::Runs(vec![7, 3]), XAxis::Time));
        assert_eq!(l.iter().map(|x| x.run).collect::<Vec<_>>(), [7, 3]);
    }

    #[test]
    fn blocks_without_seam_times_fall_back_to_their_numbers() {
        let mut r = run(1, 1.0, &[1.0, 2.0, 3.0]);
        r.clock_t_ns.clear();
        assert_eq!(
            points(&r, XAxis::Time),
            (vec![(1.0, 1.0), (2.0, 2.0), (3.0, 3.0)], false)
        );
        let mut r = run(1, 1.0, &[1.0, 2.0]);
        r.block_agg = 2;
        assert!(!points(&r, XAxis::Time).1);
    }

    #[test]
    fn ticks_are_round_and_cover_the_span() {
        assert_eq!(tick_step(10.0), 2.0);
        assert_eq!(tick_step(0.3), 0.05);
        assert_eq!(ticks(0.0, 10.0), [0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(tick_label(0.25, 0.05), "0.25");
        assert_eq!(tick_label(4.0, 2.0), "4");
    }

    #[test]
    fn the_tracked_baseline_draws_a_panel_per_invocation() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("records/knobs.jsonl");
        let mut skipped = Skipped::default();
        let runs = record::read_analyzed(&path, &mut skipped).unwrap();
        let panels = panels(runs, &plan(Show::Trim, XAxis::Time));
        assert_eq!(panels.len(), 26, "one per invocation");
        let text = svg(&panels);
        assert!(text.starts_with("<svg") && text.ends_with("</svg>\n"));
        assert_eq!(text.matches("<polyline").count(), 350, "one line per run");
        assert!(text.contains("trimmed 94."));
    }

    #[test]
    fn the_format_follows_the_extension_and_a_png_is_one() {
        assert_eq!(format_of(std::path::Path::new("a.png")), Ok(Format::Png));
        assert_eq!(format_of(std::path::Path::new("a.svg")), Ok(Format::Svg));
        assert!(format_of(std::path::Path::new("a.jpg")).is_err());
        let runs = vec![run(1, 1.0, &[1.0, 2.0, 1.5])];
        let bytes = png(&svg(&panels(runs, &plan(Show::All, XAxis::Time)))).unwrap();
        assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    }
}
