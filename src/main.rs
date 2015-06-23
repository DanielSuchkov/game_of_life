#[macro_use]
extern crate glium;
extern crate rand;
extern crate nalgebra;
extern crate clock_ticks;

mod support;
mod camera;
mod transform;
mod objects;

use std::thread;
use glium::Surface;
use glium::glutin;
use std::borrow::Borrow;
use transform::{Factor, Transform};
use nalgebra::Vec3;
use glium::backend::glutin_backend::{GlutinFacade};

pub enum Action {
    Stop,
    Continue,
}

struct StateManager {
    angle: f64
}

#[derive(Copy, Clone)]
pub struct Mat4Attr {
    pub row0: [f32; 4],
    pub row1: [f32; 4],
    pub row2: [f32; 4],
    pub row3: [f32; 4],
}

impl Default for Mat4Attr {
    fn default() -> Mat4Attr {
        Mat4Attr{
            row0: [0.0, 0.0, 0.0, 0.0],
            row1: [0.0, 0.0, 0.0, 0.0],
            row2: [0.0, 0.0, 0.0, 0.0],
            row3: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

implement_vertex!(Mat4Attr, row0, row1, row2, row3);

impl StateManager {
    fn new() -> StateManager {
        StateManager { angle: 0.0 }
    }

    fn collect_initial_state<F, T>(mut func: F) -> Vec<T>
        where F: FnMut(Transform) -> T {
        (0 .. 100).map(|n|
            func(Transform::new()
                .with_rotation(Vec3::new(0.0, 0.0f32, 0.0))
                .with_translation(Vec3::new(((n / 10) as f32 - 4.5) * 15.0, ((n % 10) as f32 - 4.5) * 15.0, 200.0))
            )
        ).collect::<Vec<_>>()
    }

    fn update_state_storage<'a>(&self, iter_over_stg: std::slice::IterMut<'a, Transform>) {
        for t in iter_over_stg {
            t.with_rotation_mut(Vec3 {
                x: self.angle.to_radians() as f32,
                y: self.angle.to_radians() as f32,
                z: -self.angle.to_radians() as f32
            });
        }
    }

    fn step_state_fwd(&mut self) {
        self.angle += 5.0;
    }

    fn step_state_bwd(&mut self) {
        self.angle -= 5.0;
    }
}

struct Sterek {
    display: GlutinFacade,
    main_group: objects::InstancedObjects<Mat4Attr>,
    main_program: glium::Program,
    camera: camera::PerspectiveCamera,
    state_mgr: StateManager,
}

fn transform_to_mat4attr(t: &Transform) -> Mat4Attr {
    let model = t.to_array();
    Mat4Attr {
        row0: model[0],
        row1: model[1],
        row2: model[2],
        row3: model[3],
    }
}

impl Sterek {
    fn new() -> Sterek {
        use glium::DisplayBuild;

        let display = glutin::WindowBuilder::new()
            .with_depth_buffer(24)
            .build_glium()
            .unwrap();

        let object_group = objects::InstancedObjects::new(
            &display,
            support::read_from_obj(&display, "support/cube.obj").unwrap(),
            StateManager::collect_initial_state(|_| Default::default())
        );

        let program = glium::Program::from_source(
            &display,
            support::read_file_content("shaders/vertex.glsl").unwrap().borrow(),
            support::read_file_content("shaders/fragment.glsl").unwrap().borrow(),
            None
        ).unwrap();

        Sterek {
            display: display,
            main_group: object_group,
            main_program: program,
            state_mgr: StateManager::new(),
            camera: camera::PerspectiveCamera::new().with_fov(60),
        }
    }

    fn main_loop(&mut self) {
        let params = glium::DrawParameters {
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut transforms = StateManager::collect_initial_state(|t| t.with_scale(Factor::Scalar(5.0)));
        let mut accumulator = 0;
        let mut previous_clock = clock_ticks::precise_time_ns();

        'main_loop: loop {
            self.state_mgr.update_state_storage(transforms.iter_mut());
            self.update_state_buffer(transforms.iter());
            self.redraw_scene(self.display.draw(), &params);

            if let Action::Stop = self.process_events() {
                break 'main_loop;
            }

            let now = clock_ticks::precise_time_ns();
            accumulator += now - previous_clock;
            previous_clock = now;

            const FIXED_TIME_STAMP: u64 = 16666667;
            while accumulator >= FIXED_TIME_STAMP {
                accumulator -= FIXED_TIME_STAMP;
                // if you have a game, update the state here
            }

            thread::sleep_ms(((FIXED_TIME_STAMP - accumulator) / 1000000) as u32);
        }
    }

    fn update_state_buffer<'a, I>(&mut self, storage_iter: I)
        where I: Iterator<Item = &'a Transform> {
        self.main_group.update_per_instance_buffer(|ref mut m|
            for (transf, mat) in storage_iter.zip(m.iter_mut()) {
                *mat = transform_to_mat4attr(&transf);
            }
        );
    }

    fn redraw_scene(&self, mut target: glium::Frame, params: &glium::DrawParameters) {
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(
            self.main_group.get_vertices_data(),
            self.main_group.get_indices_data(),
            &self.main_program,
            &uniform!{ mvp: self.camera.to_vp_array() },
            &params
        ).unwrap();

        target.finish();
    }

    fn process_events(&mut self) -> Action {
        for event in self.display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Up)) => {
                    self.camera.add_position(Vec3::new(0.0, 0.0, -1.0));
                    self.state_mgr.step_state_fwd();
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Down)) => {
                    self.camera.add_position(Vec3::new(0.0, 0.0, 1.0));
                    self.state_mgr.step_state_bwd();
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Left)) => {
                    self.camera.add_rotation(Vec3::new(0.0, -1.0f64.to_radians() as f32, 0.0));
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Right)) => {
                    self.camera.add_rotation(Vec3::new(0.0, 1.0f64.to_radians() as f32, 0.0));
                },

                glutin::Event::Resized(x, y) => {
                    self.camera.with_view_dimensions(x, y);
                },
                _ => {}
            }
        }
        Action::Continue
    }
}

fn main() {
    let mut sterek = Sterek::new();
    sterek.main_loop();
}
