#[macro_use]
extern crate glium;

use crate::glutin::dpi::PhysicalSize;
use crate::glutin::platform::unix::x11::ffi::Complex;
use glium::{glutin, Surface};
use num::Complex;
use std::f64::consts::PI;
use std::ops::{Add, Mul};
use std::time::Instant;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut start_time = Instant::now();

    let rect = vec![
        Vertex {
            position: [-1.0, -1.0],
        },
        Vertex {
            position: [1.0, -1.0],
        },
        Vertex {
            position: [1.0, 1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
        },
    ];

    let vertex_buffer = glium::VertexBuffer::new(&display, &rect).unwrap();
    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &([0, 1, 2, 2, 3, 0] as [u16; 6]),
    )
    .unwrap();

    let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        out vec2 pos;
        
        uniform float time;
        uniform vec2 mouse;
        
        void main() {
            pos = position;
            gl_Position = vec4(pos, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec2 pos;
        out vec4 color;
        
        uniform float time;
        uniform vec2 mouse;
        uniform float width;
        uniform float height;
        //uniform CircleData { FourierCircle[8] circles; };
        //uniform int circles_length;
        
        struct FourierCircle {
            float frequency;
            float phase;
            float amplitude;
        };
        
        struct CircleDotReturn {
            vec2 coords;
            vec4 color;        
        };
        
        CircleDotReturn circle_dot(vec2 uv, vec2 last_offset, FourierCircle f_circle, vec4 color, float time_des) {
            #define PI 3.1415926538

            float thickness = 0.02;
        
            float angle = (time_des * f_circle.frequency + f_circle.phase);
        
            float circle = abs(length(uv - last_offset) - f_circle.amplitude) - thickness;
            circle = circle / fwidth(circle);

            //vec2 uv_rot = vec2(uv.x + f_circle.amplitude * cos(angle), uv.y + f_circle.amplitude * sin(angle));
            vec2 uv_rot = vec2((uv.x - last_offset.x) * cos(angle) - (uv.y - last_offset.y) * sin(angle), (uv.x - last_offset.x) * sin(angle) + (uv.y - last_offset.y) * cos(angle));
            
            float rot_circle = thickness - abs(length(uv_rot - vec2(f_circle.amplitude, 0.0)));
            rot_circle = rot_circle / fwidth(rot_circle);
            
            vec3 fg = vec3(1.0, 0.0, 0.0);
            vec3 fg2 = vec3(1.0, 1.0, 1.0);
            vec3 bg = vec3(0.0, 0.0, 0.0);
            
            vec4 col = vec4(0);
            
            if (rot_circle >= circle) {
                col = vec4(rot_circle * fg, 1.0);
            } else {
                col = vec4(circle * fg2, 1.0);
            }
            
            return CircleDotReturn(uv_rot, col);
        }
        
        void main() {            
            vec2 uv = pos / normalize(vec2(height, width));
            
            FourierCircle[8] circles;
            
            circles[0] = FourierCircle(3.0, 2.356194490192345, 0.10355339059327376);
            circles[1] = FourierCircle(7.0, 2.3561944901923457, 0.6035533905932737);
            circles[2] = FourierCircle(0, 0, 0.0);
            circles[3] = FourierCircle(1.0, 0.0, 6.938893903907228e-18);
            circles[4] = FourierCircle(2.0, 0.6101377437853474, 2.4220355306086514e-17);
            circles[5] = FourierCircle(4.0, 0.8525869032260585, 5.2724156532840057e-17);
            circles[6] = FourierCircle(5.0, 1.5707963267948966, 2.8449465006019636e-16);
            circles[7] = FourierCircle(6.0, 1.3280935455684066, 2.4304557997560485e-16);

            int size = 8;
            
            vec4 c;
            CircleDotReturn r;
            
            // Plot all circles with center dots
            for (int i = 0; i < size-5; ++i) {
                if (i == 0) {
                    r = circle_dot(uv, vec2(0, 0), circles[0], color, time);
                    c = r.color;                    
                } else {
                    r = circle_dot(r.coords, vec2(circles[i - 1].amplitude, 0), circles[i], color, time);
                    
                    // Check if color is red
                    if (r.color.r > 0 && r.color.g == 0 && r.color.b == 0) {
                        c = r.color;
                    } else {
                        c = vec4(min(c.r, r.color.r), min(c.g, r.color.g), min(c.b, r.color.b), min(c.a, r.color.a));                    
                    }
                }
            }
            
            // Plot the last dot for a given amount of frames
            float time_des = time;
            int samples = 10;
            
            float dt = ((2 * PI) / size);
            
            while (time_des >= dt) {
                for (int i = 0; i < size-5; ++i) {
                    if (i == 0) {
                        r = circle_dot(uv, vec2(0, 0), circles[0], color, time_des);                    
                    } else {
                        r = circle_dot(r.coords, vec2(circles[i - 1].amplitude, 0), circles[i], color, time_des);
                    }
                    
                    // Last one
                    if (i == size - 1-5) {
                        if (r.color.r > 0 && r.color.g == 0 && r.color.b == 0) {
                            c = r.color;
                        }
                    }
                }
                
                time_des -= dt;
            }
            
            color = c;
        }
    "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let mut display_size = PhysicalSize {
        width: display.get_framebuffer_dimensions().0,
        height: display.get_framebuffer_dimensions().1,
    };
    let mut mouse_position = [0.0, 0.0];

    let points = vec![
        (-0.5, 0.5),
        (0.0, 0.5),
        (0.5, 0.5),
        (0.5, 0.0),
        (0.5, -0.5),
        (0.0, -0.5),
        (-0.5, -0.5),
        (-0.5, 0.0),
    ];

    let fourier = dft(&points);
    println!("{:#?}", &fourier);

    let uniforms = Uniforms {
        circle_count: fourier.len() as i32,
        circles: fourier,
    };

    let mut t: f32 = 0.0;
    let dt: f32 = (2.0 * (PI as f32)) / (points.len() as f32);

    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();

        t += dt;

        let next_frame_time = Instant::now() + std::time::Duration::from_nanos(16_666_666);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glutin::event::WindowEvent::Resized(size) => {
                    display_size = size;
                    return;
                }
                glutin::event::WindowEvent::CursorMoved {
                    device_id: _device_id,
                    position,
                    ..
                } => {
                    mouse_position[0] = position.x as f32;
                    mouse_position[1] = (display_size.height as f32) - (position.y as f32);
                    return;
                }
                _ => return,
            },
            glutin::event::Event::MainEventsCleared => {
                t += dt;

                if t > 2.0 * (PI as f32) {
                    t = 0.0;
                }

                let mut time_delta =
                    ((now.duration_since(start_time).as_secs_f32()) / dt).round() * dt;

                if time_delta >= (2.0 * PI) as f32 {
                    time_delta = 0.0f32;
                    start_time = Instant::now();
                }

                println!("{}", time_delta);

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 1.0, 1.0);
                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniform! {
                            time: time_delta,
                            mouse: mouse_position,
                            width: display_size.width as f32,
                            height: display_size.height as f32
                        },
                        &Default::default(),
                    )
                    .unwrap();
                target.finish().unwrap();
            }
            _ => return,
        }
    });
}

#[derive(Debug, Copy, Clone)]
struct FourierCircle {
    frequency: f64,
    phase: f64,
    amplitude: f64,
}

struct Uniforms {
    circle_count: i32,
    circles: [FourierCircle; 24 as usize],
}

fn dft(points: &Vec<(f64, f64)>) -> [FourierCircle; 24] {
    let mut X: [FourierCircle; 24] = [FourierCircle {
        amplitude: 0.0,
        frequency: 0.0,
        phase: 0.0,
    }; 24];
    let N = points.len();

    for k in 0..N {
        let mut sum: Complex<f64> = Complex::new(0.0, 0.0);

        for n in 0..N {
            let (x, y) = points[n];
            let argument = (2.0 * PI * (k as f64) * (n as f64)) / (N as f64);

            //println!("{} PHI for  k {} +  n {}", argument, k, n);

            let res = Complex::new(x, y).mul(Complex::new(argument.cos(), -argument.sin()));

            //println!("{:?}", res);

            sum = Complex::new(sum.re + res.re, sum.im + res.im);
        }

        sum.re = sum.re / (N as f64);
        sum.im = sum.im / (N as f64);

        println!("{:?}", sum);
        println!("Real {:?}", sum.re);
        println!("Img {:?}", sum.im);
        println!("Atan2 {:?}", sum.re.atan2(sum.im));

        X[k] = FourierCircle {
            frequency: k as f64,
            phase: sum.im.atan2(sum.re),
            amplitude: (sum.re * sum.re + sum.im * sum.im).sqrt(),
        };
    }

    X
}
