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
        
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140
        in vec2 pos;
        out vec4 color;
        
        uniform float time;
        uniform vec2 mouse;
        
        void main() {
            color = vec4(
                (sin((pos.x + time)) + 1) / 2,
                (sin((pos.y + time)) + 1) / 2,
                (cos((pos.y + pos.x + time)) + 1) / 2,
                1.0
            );
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
                    mouse_position[0] = (position.x as f32) / (display_size.width as f32);
                    mouse_position[1] = (position.y as f32) / (display_size.height as f32);
                    return;
                }
                _ => return,
            },
            glutin::event::Event::MainEventsCleared => {
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniform! {
                            time: now.duration_since(start_time).as_secs_f32(),
                            mouse: mouse_position
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
