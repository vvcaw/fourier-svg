#[macro_use]
extern crate glium;

use crate::glutin::dpi::PhysicalSize;
use glium::{glutin, Surface};
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

    let start_time = Instant::now();

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
        
        struct FourierCircle {
            float angle;
            float radius;
        };
        
        struct CircleDotReturn {
            vec2 coords;
            vec4 color;        
        };
        
        CircleDotReturn circle_dot(vec2 uv, vec2 last_offset, FourierCircle f_circle, vec4 color) {
            #define PI 3.1415926538

            float thickness = 0.02;
        
            float angle = PI/4 * time;
        
            float circle = abs(length(uv - last_offset) - f_circle.radius) - thickness;
            circle = circle / fwidth(circle);

            vec2 uv_rot = vec2((uv.x - last_offset.x) * cos(angle) - (uv.y - last_offset.y) * sin(angle), (uv.x - last_offset.x) * sin(angle) + (uv.y - last_offset.y) * cos(angle));
            
            float rot_circle = thickness - abs(length(uv_rot - vec2(f_circle.radius, 0.0)));
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
            
            FourierCircle[3] circles;
            circles[0] = FourierCircle(0, 0.4);
            circles[1] = FourierCircle(0, 0.5);
            circles[2] = FourierCircle(0, 0.2);
            
            int size = 3;
            
            vec4 c;
            CircleDotReturn r;
            
            for (int i = 0; i < size; ++i) {
                if (i == 0) {
                    r = circle_dot(uv, vec2(0, 0), circles[0], color);
                    c = r.color;                    
                } else {
                    r = circle_dot(r.coords, vec2(circles[i - 1].radius, 0), circles[i], color);
                    
                    // Check if color is red
                    if (r.color.r > 0 && r.color.g == 0 && r.color.b == 0) {
                        c = r.color;
                    } else {
                        c = vec4(min(c.r, r.color.r), min(c.g, r.color.g), min(c.b, r.color.b), min(c.a, r.color.a));                    
                    }
                }
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

    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();

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
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 1.0, 1.0);
                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniform! {
                            time: now.duration_since(start_time).as_secs_f32(),
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
