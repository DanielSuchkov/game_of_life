#[macro_use]
extern crate glium;
extern crate rand;
extern crate nalgebra;

mod support;
#[allow(dead_code)]
mod transform;

use glium::Surface;
use glium::glutin;
use std::fs::File;
use std::io::Read;
use std::borrow::Borrow;
use transform::{Factor, Transform};
use nalgebra::Vec3;


pub fn read_from_obj<'a>(display: &glium::Display, path: &'a str)
    -> (glium::vertex::VertexBufferAny, glium::index::NoIndices) {
    let mut buf = Vec::new();
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();
    (support::load_wavefront(display, &buf), glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList))
}

pub fn read_file_content<'a>(path: &'a str) -> std::io::Result<String> {
    let mut content = String::new();
    match File::open(path) {
        Ok(mut f) => f.read_to_string(&mut content).and_then(|_| Ok(content)),
        Err(e) => { println!("cannot open file: {}", e); Err(e) }
    }
}

fn main() {
    use glium::DisplayBuild;

    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .build_glium()
        .unwrap();

    let vertex_buffer = support::load_wavefront(&display, include_bytes!("support/teapot.obj"));
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let mut per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            model0: [f32; 4],
            model1: [f32; 4],
            model2: [f32; 4],
            model3: [f32; 4],
        }

        implement_vertex!(Attr, model0, model1, model2, model3);

        let data = (0 .. 100).map(|n| {
            let model = Transform::new()
                .with_scale(Factor::Scalar(1.0))
                .with_rotation(Vec3::new(0.0, 0.0f64.to_radians() as f32, 0.0))
                .with_translation(Vec3::new( ((n / 10) as f32 - 4.5) * 180.0, ((n % 10) as f32 - 4.5) * 100.0, 1500.0))
                .to_array();

            Attr {
                model0: model[0],
                model1: model[1],
                model2: model[2],
                model3: model[3],
            }
        }).collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, data)
    };

    let program = glium::Program::from_source(&display,
        read_file_content("shaders/vertex.glsl").unwrap().borrow(),
        read_file_content("shaders/fragment.glsl").unwrap().borrow(),
        None)
        .unwrap();

    let mut perspective = nalgebra::PerspMat3::new(1024.0 / 768.0, 3.14159 / 3.0, 0.1, 2000.0);
    let mut projection = perspective.to_mat().as_array().clone();
    let mut angle = 0.0f64;

    let params = glium::DrawParameters {
        depth_test: glium::DepthTest::IfLess,
        depth_write: true,
        .. Default::default()
    };

    support::start_loop(|| {
        for (n, mat) in (0 .. 100).zip(per_instance.map().iter_mut()) {
            let model = Transform::new()
                .with_scale(Factor::Scalar(1.0))
                .with_rotation(Vec3::new(0.0, angle.to_radians() as f32, 0.0))
                .with_translation(Vec3::new(((n / 10) as f32 - 4.5) * 180.0, ((n % 10) as f32 - 4.5) * 100.0, 1500.0))
                .to_array();

            mat.model0 = model[0];
            mat.model1 = model[1];
            mat.model2 = model[2];
            mat.model3 = model[3];
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(
            (&vertex_buffer, per_instance.per_instance_if_supported().unwrap()),
            &indices,
            &program,
            &uniform!{ mvp: projection},
            &params
        ).unwrap();

        target.finish();

        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return support::Action::Stop,
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Up)) => {
                    angle += 1.0;
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Down)) => {
                    angle -= 1.0;
                },
                glutin::Event::Resized(x, y) => {
                    perspective.set_aspect(x as f32 / y as f32);
                    projection = perspective.to_mat().as_array().clone()
                },
                _ => ()
            }
        }

        support::Action::Continue
    });
}
