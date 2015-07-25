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
use transform::{Factor, Transform};
use nalgebra::{Vec3, Norm};
use glium::backend::glutin_backend::{GlutinFacade};

pub enum Action {
    Stop,
    Continue,
}

pub struct PerObjectState {
    t: transform::Transform,
}

fn transform_to_mat4attr(t: &Transform) -> PerObjectAttr {
    let model = t.to_array();
    PerObjectAttr {
        row0: model[0],
        row1: model[1],
        row2: model[2],
        row3: model[3],
    }
}

impl PerObjectState {
    pub fn to_attr(&self) -> PerObjectAttr {
        transform_to_mat4attr(&self.t)
    }
}

#[derive(Copy, Clone)]
pub struct PerObjectAttr {
    pub row0: [f32; 4],
    pub row1: [f32; 4],
    pub row2: [f32; 4],
    pub row3: [f32; 4],
}

type Range3 = (std::ops::Range<i32>, std::ops::Range<i32>, std::ops::Range<i32>);

struct State {
    angle: f64,
    nbs: Range3,
}

impl State {
    pub fn new(el_rng: Range3) -> State {
        State {
            angle: 0.0,
            nbs: el_rng,
        }
    }

    pub fn iter_over_dims(&self) -> Box<Iterator<Item = (i32, i32, i32)>> {
        let (d1, d2, d3) = self.nbs.clone();
        Box::new(
            d1.flat_map(move |x| {
                let d3 = d3.clone();
                d2.clone().flat_map(move |y| {
                    d3.clone().map(move |z|{
                        (x, y, z)
                    })
                })
            })
        )
    }

    pub fn get_initial_state(&self) -> Box<Iterator<Item = PerObjectState>> {
        let (xmax, ymax) = (self.nbs.0.end, self.nbs.1.end);
        Box::new(
            self.iter_over_dims().map(move |(x, y, z)|
                PerObjectState {
                    t: Transform::new()
                        .with_rotation(Vec3::new(0.0, 0.0f32, 0.0))
                        .with_translation(Vec3::new(
                            (x as f32 - (xmax - 1) as f32 / 2.0) * 15.0,
                            (y as f32 - (ymax - 1) as f32 / 2.0) * 15.0,
                            350.0 + z as f32 * 15.0)
                        )
                        .with_scale(Factor::Scalar(5.0))
                }
            )
        )
    }

    pub fn up_to_actual_state(&self, state: &mut Vec<PerObjectState>) {
        for (_, obj_st) in self.iter_over_dims().zip(state.iter_mut()) {
            obj_st.t.with_rotation_mut(
                Vec3 {
                    x: self.angle.to_radians() as f32,
                    y: self.angle.to_radians() as f32,
                    z: self.angle.to_radians() as f32
                }
            );
        }
    }

    pub fn step_forward(&mut self) {
        self.angle += 5.0;
    }

    pub fn step_backward(&mut self) {
        self.angle -= 5.0;
    }
}

implement_vertex!(PerObjectAttr, row0, row1, row2, row3);

struct Sterek {
    display: GlutinFacade,
    main_group: objects::InstancedObjects<PerObjectAttr>,
    main_program: glium::Program,
    camera: camera::PerspectiveCamera,
    state: State,
}

impl Sterek {
    fn new() -> Sterek {
        use glium::DisplayBuild;

        let state = State::new((0..24, 0..24, 0..1));

        let display = glutin::WindowBuilder::new()
            .with_depth_buffer(24)
            .build_glium()
            .unwrap();

        let object_group = objects::InstancedObjects::new(
            &display,
            support::read_from_obj(&display, "support/cube.obj", true).unwrap(),
            state.get_initial_state().map(|s: PerObjectState| s.to_attr()).collect::<Vec<_>>()
        );

        let program = glium::Program::from_source(
            &display,
            &support::read_file_content("shaders/vertex.glsl").unwrap(),
            &support::read_file_content("shaders/fragment.glsl").unwrap(),
            None
        ).unwrap();

        Sterek {
            display: display,
            main_group: object_group,
            main_program: program,
            state: state,
            camera: camera::PerspectiveCamera::new().with_fov(60),
        }
    }

    fn main_loop(&mut self) {
        use glium::draw_parameters::{BlendingFunction, LinearBlendingFactor};

        let params = glium::DrawParameters {
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            /*blending_function: Some(BlendingFunction::Addition {
                source: LinearBlendingFactor::SourceAlpha,
                destination: LinearBlendingFactor::OneMinusSourceAlpha
            }),*/
            .. Default::default()
        };

        let mut transforms = self.state.get_initial_state().collect::<Vec<_>>();
        let mut accumulator = 0;
        let mut previous_clock = clock_ticks::precise_time_ns();

        'main_loop: loop {
            self.state.up_to_actual_state(&mut transforms);
            self.update_state_buffer(transforms.iter());
            self.redraw_scene(self.display.draw(), &params);

            if let Action::Stop = self.process_events() {
                break 'main_loop;
            }

            // let now = clock_ticks::precise_time_ns();
            // accumulator += now - previous_clock;
            // previous_clock = now;

            // const FIXED_TIME_STAMP: u64 = 16666667;
            // while accumulator >= FIXED_TIME_STAMP {
            //     accumulator -= FIXED_TIME_STAMP;
            //     // if you have a game, update the state here
            // }

            // thread::sleep_ms(((FIXED_TIME_STAMP - accumulator) / 1000000) as u32);
        }
    }

    fn update_state_buffer<'a, I>(&mut self, storage_iter: I)
        where I: Iterator<Item = &'a PerObjectState> {
        self.main_group.update_per_instance_buffer(|ref mut m|
            for (transf, mat) in storage_iter.zip(m.iter_mut()) {
                *mat = transf.to_attr();
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
                    self.state.step_forward();
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Down)) => {
                    self.camera.add_position(Vec3::new(0.0, 0.0, 1.0));
                    self.state.step_backward();
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
