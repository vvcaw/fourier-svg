use lazy_static::lazy_static;
use nannou::prelude::*;
use num::Complex;
use std::path::PathBuf;
use structopt::StructOpt;
use svg2pts_lib::get_path_from_file;

#[derive(StructOpt, Debug)]
#[structopt(name = "fourier-svg")]
pub struct Opt {
    /// Distance between points sampled from svg
    #[structopt(short, long, default_value = "5.0")]
    distance: f64,
    /// Input svg file
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,
}

lazy_static! {
    pub static ref OPT: Opt = Opt::from_args();
}

struct Model {
    fourier: Vec<FourierCoefficients>,
    draw: Draw,
    active_samples: Vec<(f32, f32)>,
    dt: f32,
}

fn parse_svg(filename: &str, point_distance: f64) -> Vec<(f32, f32)> {
    get_path_from_file(filename, 0, point_distance)
        .iter()
        .map(|(x, y)| (*x as f32, *y as f32))
        .collect()
}

fn main() {
    nannou::app(model)
        .update(update)
        .event(event)
        .simple_window(view)
        .run();
}

fn model(_app: &App) -> Model {
    let points = parse_svg(OPT.file.to_str().unwrap(), OPT.distance);
    let mut series = dft(&points);

    // Remove constant term (offset from (0, 0))
    series.remove(0);

    // Sort for better looks
    series.sort_by(|c0, c1| c1.amplitude.partial_cmp(&c0.amplitude).unwrap());

    Model {
        fourier: series,
        dt: (2.0 * PI) / (points.len() as f32),
        draw: Draw::new(),
        active_samples: vec![],
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Calculate time in multiples of `dt`, as we should only render the line at the sampled points to get the correct path back out
    let t = ((app.elapsed_frames() as f32) * model.dt) % (2.0 * PI);

    let win = app.window_rect();

    // Evaluate resolution from mouseX position
    let resolution = (app.mouse.x + win.right()) / win.w();

    // Draw epicycles
    let sample = draw_epicycles(&model.draw, &model.fourier, t, resolution);

    // Get number of drawn samples at current point in time
    let current_sample_count = (t / model.dt).ceil().to_usize().unwrap();

    // Reset the samples after one full iteration
    if current_sample_count == 0 {
        model.active_samples.clear();
    }

    model.active_samples.push(sample);

    // Draw all samples
    draw_samples(
        &model.draw,
        &model.active_samples,
        current_sample_count,
        model.fourier.len(),
    );
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = &model.draw;

    draw.background().color(rgb(0.22, 0.25, 0.32));

    draw.to_frame(app, &frame).unwrap();
}

fn draw_samples(
    draw: &Draw,
    points: &Vec<(f32, f32)>,
    current_sample_count: usize,
    max_sample_count: usize,
) {
    for i in 1..current_sample_count {
        let (last_x, last_y) = points[i - 1];

        // Use first vertex if path is filled entirely
        let (current_x, current_y) = if i == (max_sample_count - 1) {
            points[0]
        } else {
            points[i]
        };

        draw.line()
            .points(
                Point2::new(last_x, last_y),
                Point2::new(current_x, current_y),
            )
            .start_cap_round()
            .end_cap_round()
            .stroke_weight(4.0)
            .color(rgb(0.35f32, 0.64f32, 0.69f32));
    }
}

fn draw_epicycles(
    draw: &Draw,
    fourier: &Vec<FourierCoefficients>,
    time: f32,
    resolution: f32,
) -> (f32, f32) {
    let mut x = 0.0;
    let mut y = 0.0;

    // Render epicycles with given resolution
    for i in 0..((fourier.len() as f32) * resolution.clamp(0.0, 1.0))
        .ceil()
        .to_usize()
        .unwrap()
    {
        let FourierCoefficients {
            frequency,
            amplitude,
            phase,
        } = fourier[i];

        let angle = frequency * time + phase;

        let x_with_offset = x + angle.cos() * amplitude;
        let y_with_offset = y + angle.sin() * amplitude;

        // Draw circle
        draw.ellipse()
            .x_y(x, y)
            .no_fill()
            .stroke_weight(4.0)
            .radius(amplitude)
            .stroke_color(rgba(0.91f32, 0.90f32, 1.0f32, 0.2));

        // Draw line to moving dot
        draw.arrow()
            .points(Point2::new(x, y), Point2::new(x_with_offset, y_with_offset))
            .start_cap_round()
            .end_cap_round()
            .stroke_weight(4.0)
            .color(rgb(0.91f32, 0.90f32, 1.0f32));

        x = x_with_offset;
        y = y_with_offset;
    }

    (x, y)
}

/// Hold data for the calculated coefficients
struct FourierCoefficients {
    frequency: f32,
    amplitude: f32,
    phase: f32,
}

/// Calculate the discrete fourier transform for the given samples of a path
fn dft(points: &Vec<(f32, f32)>) -> Vec<FourierCoefficients> {
    // Map each point to it's coefficients
    points
        .iter()
        .enumerate()
        .map(|(k, _)| {
            let mut sum: Complex<f32> = Complex::new(0.0, 0.0);

            for n in 0..points.len() {
                let (x, y) = points[n];
                let angle = (2.0 * PI * (k as f32) * (n as f32)) / (points.len() as f32);

                let res = Complex::new(x, y) * Complex::new(angle.cos(), -angle.sin());

                sum = sum + res;
            }

            // This makes the circles have the correct relative size
            sum.im = sum.im / (points.len() as f32);
            sum.re = sum.re / (points.len() as f32);

            FourierCoefficients {
                frequency: k as f32,
                phase: sum.im.atan2(sum.re),
                amplitude: (sum.re * sum.re + sum.im * sum.im).sqrt(),
            }
        })
        .collect()
}
