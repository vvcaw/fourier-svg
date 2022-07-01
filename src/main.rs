use nannou::prelude::*;
use num::Complex;

struct Model {
    fourier: Vec<FourierCoefficients>,
    dt: f32,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    let points = vec![
        (-40.0, 40.0),
        (0.0, 40.0),
        (40.0, 40.0),
        (40.0, 0.0),
        (40.0, -40.0),
        (0.0, -40.0),
        (-40.0, -40.0),
        (-40.0, 0.0),
    ];

    let mut series = dft(&points);
    series.sort_by(|c0, c1| c1.amplitude.partial_cmp(&c0.amplitude).unwrap());

    Model {
        fourier: series,
        dt: (2.0 * PI) / (points.len() as f32),
    }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // #373F51 -> Bg
    // #E9E6FF -> Circles
    // #58A4B0 -> Trail
    draw.background().color(rgb(0.22, 0.25, 0.32));

    // Calculate time in multiples of `dt`, as we should only render the line at the sampled points to get the correct path back out
    let t = ((app.elapsed_frames() as f32) * model.dt) % (2.0 * PI);

    // Draw epicycles
    draw_epicycles(&draw, &model.fourier, t);

    draw.to_frame(app, &frame).unwrap();
}

fn draw_epicycles(draw: &Draw, fourier: &Vec<FourierCoefficients>, time: f32) {
    let mut x = 0.0;
    let mut y = 0.0;

    for FourierCoefficients {
        frequency,
        amplitude,
        phase,
    } in fourier
    {
        let angle = frequency * time + phase;

        let x_with_offset = x + angle.cos() * amplitude;
        let y_with_offset = y + angle.sin() * amplitude;

        // Draw circle
        draw.ellipse()
            .x_y(x, y)
            .no_fill()
            .stroke_weight(8.0)
            .radius(*amplitude)
            .stroke_color(rgb(0.91f32, 0.90f32, 1.0f32));

        // Draw line to moving dot
        draw.line()
            .points(Point2::new(x, y), Point2::new(x_with_offset, y_with_offset))
            .start_cap_round()
            .end_cap_round()
            .stroke_weight(8.0)
            .color(rgb(0.91f32, 0.90f32, 1.0f32));

        x = x_with_offset;
        y = y_with_offset;
    }
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

            FourierCoefficients {
                frequency: k as f32,
                phase: sum.im.atan2(sum.re),
                amplitude: (sum.re * sum.re + sum.im * sum.im).sqrt(),
            }
        })
        .collect()
}
