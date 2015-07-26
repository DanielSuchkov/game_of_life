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
use nalgebra::Vec3;
use glium::backend::glutin_backend::GlutinFacade;

pub enum Action {
    Stop,
    Continue,
}

pub struct PerObjectState {
    pos: Vec3<f32>,
    scale_factor: f32,
    show: bool,
    color: Vec3<f32>,
}

impl PerObjectState {
    pub fn to_attr(&self) -> PerObjectAttr {
        PerObjectAttr {
            pos: self.pos.as_array().clone(),
            scale_factor: self.scale_factor,
            color: [
                self.color.x,
                self.color.y,
                self.color.z,
                if self.show { 1.0 } else { 0.0 }
            ],
        }
    }
}

#[derive(Copy, Clone)]
pub struct PerObjectAttr {
    pub pos: [f32; 3],
    pub color: [f32; 4],
    pub scale_factor: f32,
}

implement_vertex!(PerObjectAttr, pos, color, scale_factor);

type Range3 = (std::ops::Range<usize>, std::ops::Range<usize>, std::ops::Range<usize>);

fn get_line() -> std::io::Result<String> {
    let mut result = String::new();
    match std::io::stdin().read_line(&mut result) {
        Ok(_) => Ok(result),
        Err(e) => Err(e),
    }
}

struct State {
    nbs: (usize, usize, usize),
    world: Vec<Vec<Vec<bool>>>,
    birth: Vec<u32>,
    stay: Vec<u32>,
}

impl State {
    pub fn new(x_size: usize, y_size: usize, z_size: usize) -> State {
        let mut w1 = Vec::with_capacity(x_size);
        for x in 0..(x_size + 2) {
            let mut w2 = Vec::with_capacity(y_size);
            for y in 0..(y_size + 2) {
                let mut w3 = Vec::with_capacity(z_size);
                for z in 0..(z_size + 2) {
                    let val =
                        if x == 0 || y == 0 || z == 0 || x == x_size + 1 || y == y_size + 1 || z == z_size + 1 {
                            false
                        } else {
                            if x_size / 8 <= x && x <= x_size - x_size / 8
                                && y_size / 8 <= y && y <= y_size - y_size / 8
                                && z_size / 8 <= z && z <= z_size - z_size / 8 {
                                rand::random()
                            } else {
                                false
                            }
                        };
                    w3.push(val);
                }
                w2.push(w3);
            }
            w1.push(w2);
        }

        println!("birth:");
        let b = get_line();
        println!("stay:");
        let s = get_line();

        State {
            nbs: (x_size, y_size, z_size),
            world: w1,
            birth: b.unwrap().split_whitespace().map(|s| s.parse::<u32>().unwrap()).collect(),
            stay: s.unwrap().split_whitespace().map(|s| s.parse::<u32>().unwrap()).collect(),
        }
    }

    pub fn iter_over_dims(&self) -> Box<Iterator<Item = (usize, usize, usize)>> {
        let (d1, d2, d3) = self.nbs;
        Box::new(
            (1..(d1 + 1)).flat_map(move |x| {
                (1..(d2 + 1)).flat_map(move |y| {
                    (1..(d3 + 1)).map(move |z| {
                        (x, y, z)
                    })
                })
            })
        )
    }

    pub fn get_initial_state(&self) -> Box<Iterator<Item = PerObjectState>> {
        let (xmax, ymax, zmax) = (self.nbs.0 + 2, self.nbs.1 + 2, self.nbs.2 + 2);
        let world = self.world.clone();
        Box::new(
            self.iter_over_dims().map(move |(x, y, z)|
                PerObjectState {
                    pos: Vec3::new(
                        (x as f32 - (xmax - 1) as f32 / 2.0) * 15.0,
                        (y as f32 - (ymax - 1) as f32 / 2.0) * 15.0,
                        (z as f32 - (zmax - 1) as f32 / 2.0) * 15.0,
                    ),
                    scale_factor: 5.0,
                    show: world[x][y][z],
                    color: rand::random(),
                }
            )
        )
    }

    pub fn up_to_actual_state(&self, state: &mut Vec<PerObjectState>) {
        let dims_iter = self.iter_over_dims();
        for ((x, y, z), obj_st) in dims_iter.zip(state.iter_mut()) {
            obj_st.show = self.world[x][y][z];
        }
    }

    fn rules(&self, alive: bool, neighbours: u32) -> bool {
        match alive {
            false => self.birth.iter().any(|x| *x == neighbours),
            true => self.stay.iter().any(|x| *x == neighbours),
        }
    }

    pub fn step_forward(&mut self) {
        let old_world = self.world.clone();
        for (x, y, z) in self.iter_over_dims() {
            let neighbours = {
                let mut result = 0u32;
                for mut dz in (z - 1)..(z + 2) {
                    for mut dy in (y - 1)..(y + 2) {
                        for mut dx in (x - 1)..(x + 2) {
                            if self.nbs.0 > 1 {
                                if dx == 0 { dx = self.nbs.0; }
                                if dx == (self.nbs.0 + 1) { dx = 1; }
                            }

                            if self.nbs.1 > 1 {
                                if dy == 0 { dy = self.nbs.1; }
                                if dy == (self.nbs.1 + 1) { dy = 1; }
                            }

                            if self.nbs.2 > 1 {
                                 if dz == 0 { dz = self.nbs.2; }
                                 if dz == (self.nbs.2 + 1) { dz = 1; }
                            }

                            if !(dx == x && dy == y && dz == z) {
                                result += old_world[dx][dy][dz] as u32;
                            }
                        }
                    }
                }
                result
            };
            self.world[x][y][z] = self.rules(old_world[x][y][z], neighbours);
        }
    }
}

struct Sterek {
    display: GlutinFacade,
    main_group: objects::InstancedObjects<PerObjectAttr>,
    main_program: glium::Program,
    camera: camera::PerspectiveCamera,
    state: State,
    angle: f64,
    r: f32,
}

impl Sterek {
    fn new(state: State) -> Sterek {
        use glium::DisplayBuild;

        let display = glutin::WindowBuilder::new()
            .with_depth_buffer(24)
            .build_glium()
            .unwrap();

        let object_group = objects::InstancedObjects::new(
            &display,
            support::read_from_obj(&display, "support/cube.obj", true).unwrap(),
            state.get_initial_state().map(|s| s.to_attr()).collect::<Vec<_>>()
        );

        let program = glium::Program::from_source(
            &display,
            &support::read_file_content("shaders/vertex.glsl").unwrap(),
            &support::read_file_content("shaders/fragment.glsl").unwrap(),
            None
        ).unwrap();
        let r = 1500.0;
        Sterek {
            display: display,
            main_group: object_group,
            main_program: program,
            state: state,
            angle: 0.0,
            r: r,
            camera: camera::PerspectiveCamera::new()
                .with_fov(60)
                .with_position(Vec3::new(0.0, 0.0, r)),
        }
    }

    fn main_loop(&mut self) {
        let params = glium::DrawParameters {
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut transforms = self.state.get_initial_state().collect::<Vec<_>>();
        let mut last_step_time = clock_ticks::precise_time_ns();
        'main_loop: loop {
            const FIXED_TIME_STAMP: u64 = 16666667;
            const STEP_INTERVAL: u64 = 200000000;
            let t1_ns = clock_ticks::precise_time_ns();

            self.state.up_to_actual_state(&mut transforms);
            self.update_state_buffer(transforms.iter());
            self.redraw_scene(self.display.draw(), &params);

            if let Action::Stop = self.process_events() {
                break 'main_loop;
            }
            if clock_ticks::precise_time_ns() - last_step_time > STEP_INTERVAL {
                self.state.step_forward();
                last_step_time = clock_ticks::precise_time_ns();
            }

            let dt_ns = clock_ticks::precise_time_ns() - t1_ns;
            if dt_ns < FIXED_TIME_STAMP {
                thread::sleep_ms(((FIXED_TIME_STAMP - dt_ns) / 1000000) as u32);
            }
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
        let recalc_cam_pos = |cam: &mut camera::PerspectiveCamera, angle: f64, r: f32| {
                let new_pos = Vec3::new(angle.to_radians().sin() as f32, 0.0, angle.to_radians().cos() as f32) * r;
                cam.with_position_mut(new_pos)
                    .with_rotation_mut(Vec3::new(0.0, -angle.to_radians() as f32, 0.0));
        };

        for event in self.display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Add)) => {
                    self.state.step_forward();
                }

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Up)) => {
                    self.r -= 7.5;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Down)) => {
                    self.r += 7.5;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::A)) => {
                    self.camera.add_position(Vec3::new(-1.0, 0.0, 0.0));
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::D)) => {
                    self.camera.add_position(Vec3::new(1.0, 0.0, 0.0));
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::W)) => {
                    self.camera.add_position(Vec3::new(0.0, -1.0, 0.0));
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::S)) => {
                    self.camera.add_position(Vec3::new(0.0, 1.0, 0.0));
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Left)) => {
                    self.angle -= 1.0;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Right)) => {
                    self.angle += 1.0;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                glutin::Event::Resized(x, y) => {
                    self.camera.with_view_dimensions_mut(x, y);
                },
                _ => {}
            }
        }
        Action::Continue
    }
}

fn main() {
    let mut sterek = Sterek::new(State::new(50, 50, 50));
    sterek.main_loop();
}
