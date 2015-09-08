#[macro_use]
extern crate glium;
extern crate rand;
extern crate nalgebra;
extern crate clock_ticks;
extern crate image;

mod support;
mod camera;
mod transform;
mod objects;

use glium::{Surface, glutin};
use nalgebra::Vec3;
use glium::backend::glutin_backend::GlutinFacade;
use glium::glutin::{Event, ElementState, VirtualKeyCode};

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

fn get_line() -> std::io::Result<String> {
    let mut result = String::new();
    match std::io::stdin().read_line(&mut result) {
        Ok(n) => {
            if n > 1 {
                Ok(result)
            }
            else {
                Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Empty line"))
            }
        },
        Err(e) => Err(e),
    }
}

type U3d = (usize, usize, usize);

#[inline]
fn i2p((_, ys, zs): U3d, idx: usize) -> U3d {
    (idx / (zs * ys),  (idx / zs) % ys, idx % zs)
}

#[inline]
fn p2i((_, ys, zs): U3d, (x, y, z): U3d) -> usize {
    x * zs * ys + y * zs + z
}

struct State {
    dim: U3d,
    world: Vec<bool>,
    old_world: Vec<bool>,
    birth: Vec<u32>,
    stay: Vec<u32>,
}

impl State {
    pub fn new((xs, ys, zs): U3d) -> State {
        let mut world = Vec::with_capacity(xs * ys * zs);

        for x in 0..xs {
            for y in 0..ys {
                for z in 0..zs {
                    let state =
                        if xs / 8 <= x && x <= xs - xs / 8
                            && ys / 8 <= y && y <= ys - ys / 8
                            && zs / 8 <= z && z <= zs - zs / 8
                            && x % 5 != 0 && y % 5 != 0 && z % 5 != 0 {
                            rand::random()
                        } else {
                            false
                        };
                    world.push(state);
                }
            }
        }

        println!("birth:");
        let b = get_line().unwrap_or(String::from("5"));
        println!("stay:");
        let s = get_line().unwrap_or(String::from("4 5"));
        println!("{} | {}", b, s);
        State {
            dim: (xs, ys, zs),
            old_world: world.clone(),
            world: world,
            birth: b.split_whitespace().map(|s| s.parse::<u32>().unwrap()).collect(),
            stay: s.split_whitespace().map(|s| s.parse::<u32>().unwrap()).collect(),
        }
    }

    pub fn get_initial_state(&self) -> Box<Iterator<Item = PerObjectState>> {
        let (xs, ys, zs) = self.dim;
        let world = self.world.clone();
        Box::new(
            (0..(xs * ys * zs)).map(move |i| {
                let (x, y, z) = i2p((xs, ys, zs), i);
                PerObjectState {
                    pos: Vec3::new(
                        (x as f32 - (xs - 1) as f32 / 2.0) * 15.0,
                        (y as f32 - (ys - 1) as f32 / 2.0) * 15.0,
                        (z as f32 - (zs - 1) as f32 / 2.0) * 15.0,
                    ),
                    scale_factor: 5.0,
                    show: world[i],
                    color: Vec3::new(0.9, 0.9, 0.9),
                        // if rand::random() {
                        //     Vec3::new(0.3 * rand::random::<f32>(), 0.05, 0.1)
                        // } else {
                        //     Vec3::new(0.1, 0.05, 0.3 * rand::random::<f32>())
                        // },
                }
            })
        )
    }

    pub fn up_to_actual_state(&self, state: &mut Vec<PerObjectState>) {
        for (mut st, wld) in state.iter_mut().zip(self.world.iter()) {
            st.show = *wld;
        }
    }

    #[inline]
    fn rules(&self, alive: bool, neighbours: u32) -> bool {
        match alive {
            false => self.birth.iter().any(|x| *x == neighbours),
            true => self.stay.iter().any(|x| *x == neighbours),
        }
    }

    pub fn step_forward(&mut self) {
        std::mem::swap(&mut self.world, &mut self.old_world);
        let (xs, ys, zs) = self.dim;
        unsafe {
            for i in 0..(xs * ys * zs) {
                let (x, y, z) = i2p(self.dim, i);
                let (x, y, z) = (x + 1, y + 1, z + 1);
                let neighbours = {
                    let mut neib = 0;
                    for mut dx in (x - 1)..(x + 2) {
                        for mut dy in (y - 1)..(y + 2) {
                            for mut dz in (z - 1)..(z + 2) {
                                if xs > 2 {
                                    if dx == 0 { dx = xs; }
                                    if dx == xs + 1 { dx = 1; }
                                }

                                if ys > 2 {
                                    if dy == 0 { dy = ys; }
                                    if dy == ys + 1 { dy = 1; }
                                }

                                if zs > 2 {
                                     if dz == 0 { dz = zs; }
                                     if dz == zs + 1 { dz = 1; }
                                }

                                if !(dx == x && dy == y && dz == z) {
                                    neib += *self.old_world.get_unchecked(p2i(self.dim, (dx - 1, dy - 1, dz - 1))) as u32;
                                }
                            }
                        }
                    }
                    neib
                };
                *self.world.get_unchecked_mut(i) = self.rules(*self.old_world.get_unchecked(i), neighbours);
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

struct Applicaton {
    display: GlutinFacade,
    main_group: objects::InstancedObjects<PerObjectAttr>,
    main_program: glium::Program,
    background_program: glium::Program,
    background_vb: glium::VertexBuffer<Vertex>,
    background_ib: glium::IndexBuffer<u16>,
    camera: camera::PerspectiveCamera,
    state: State,
    time_from_start: f32,
    angle: f64,
    r: f32,
}

impl Applicaton {
    fn new(state: State) -> Applicaton {
        use glium::DisplayBuild;

        let display = glutin::WindowBuilder::new()
            .with_depth_buffer(24)
            .build_glium()
            .unwrap();

        let object_group = objects::InstancedObjects::new(
            &display,
            support::read_from_obj(&display, "support/cube.obj", true).unwrap(),
            state.get_initial_state()
                .map(|s| s.to_attr())
                .collect()
        );

        let program = glium::Program::from_source(
            &display,
            &support::read_file_content("../shaders/main.vs").unwrap(),
            &support::read_file_content("../shaders/main.fs").unwrap(),
            None
        ).unwrap();

        let bg_program = glium::Program::from_source(
            &display,
            &support::read_file_content("../shaders/proc_tex.vs").unwrap(),
            &support::read_file_content("../shaders/proc_tex.fs").unwrap(),
            None
        ).unwrap();

        let background_vb = glium::VertexBuffer::new(
            &display,
            &[
                Vertex { position: [-1.0, -1.0]},
                Vertex { position: [-1.0,  1.0]},
                Vertex { position: [ 1.0,  1.0]},
                Vertex { position: [ 1.0, -1.0]}
            ]
        ).unwrap();

        let background_ib = glium::IndexBuffer::new(
            &display, glium::index::PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]
        ).unwrap();

        let r = 1500.0;
        Applicaton {
            display: display,
            main_group: object_group,
            main_program: program,
            background_program: bg_program,
            background_vb: background_vb,
            background_ib: background_ib,
            time_from_start: 0.0,
            state: state,
            angle: 0.0,
            r: r,
            camera: camera::PerspectiveCamera::new()
                .with_fov(60)
                .with_position(Vec3::new(0.0, 0.0, r))
                .with_zfar(5000.0),
        }
    }

    fn main_loop(&mut self) {
        let params = glium::DrawParameters {
            depth_test: glium::DepthTest::IfLess,
            depth_write: true,
            .. Default::default()
        };

        let mut transforms = self.state.get_initial_state().collect();
        let mut last_step_time = clock_ticks::precise_time_ms();
        let mut last_up_time = last_step_time;
        let mut last_frame_time = last_step_time;
        let mut frames = 0;
        'main_loop: loop {
            frames += 1;
            const STEP_INTERVAL: u64 = 500;

            let current_time = clock_ticks::precise_time_ms();
            let dt = current_time - last_step_time;
            let frame_dt = current_time - last_frame_time;
            last_frame_time = current_time;
            self.time_from_start += frame_dt as f32 / 1000.0;

            if dt > STEP_INTERVAL {
                self.state.step_forward();
                self.state.up_to_actual_state(&mut transforms);
                self.update_state_buffer(transforms.iter());
                last_step_time = clock_ticks::precise_time_ms();
            }

            self.redraw_scene(self.display.draw(), &params);

            if let Action::Stop = self.process_events() {
                break 'main_loop;
            }
            let up_dt = clock_ticks::precise_time_ms() - last_up_time;
            if up_dt > 1000 {
                println!("{ft}: {fps}", ft=(up_dt as f32 / frames as f32), fps=(frames as f32 / up_dt as f32 * 1000.0));
                last_up_time = clock_ticks::precise_time_ms();
                frames = 0;
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
        let (x_size, y_size) = target.get_dimensions();
        let resolution = nalgebra::Vec2::new(x_size as f32, y_size as f32);
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(
            &self.background_vb,
            &self.background_ib,
            &self.background_program,
            &uniform!{
                u_time: self.time_from_start,
                u_resolution: resolution,
            },
            &glium::DrawParameters {
                depth_test: glium::DepthTest::Overwrite,
                depth_write: false,
                .. Default::default()
            }
        ).unwrap();

        target.draw(
            self.main_group.get_vertices_data(),
            self.main_group.get_indices_data(),
            &self.main_program,
            &uniform!{ mvp: self.camera.to_vp_array(), u_time: self.time_from_start, },
            &params
        ).unwrap();

        target.finish().unwrap();
    }

    fn process_events(&mut self) -> Action {
        let recalc_cam_pos = |cam: &mut camera::PerspectiveCamera, angle: f64, r: f32| {
                let new_pos = Vec3::new(angle.to_radians().sin() as f32, 0.0, angle.to_radians().cos() as f32) * r;
                cam.with_position_mut(new_pos)
                    .with_rotation_mut(Vec3::new(0.0, -angle.to_radians() as f32, 0.0));
        };

        for event in self.display.poll_events() {
            match event {
                Event::Closed => return Action::Stop,

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Add)) => {
                    self.state.step_forward();
                }

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) => {
                    self.r -= 7.5;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) => {
                    self.r += 7.5;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::A)) => {
                    self.camera.add_position(Vec3::new(-1.0, 0.0, 0.0));
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::D)) => {
                    self.camera.add_position(Vec3::new(1.0, 0.0, 0.0));
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::W)) => {
                    self.camera.add_position(Vec3::new(0.0, -1.0, 0.0));
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                    self.camera.add_position(Vec3::new(0.0, 1.0, 0.0));
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Left)) => {
                    self.angle -= 1.0;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Right)) => {
                    self.angle += 1.0;
                    recalc_cam_pos(&mut self.camera, self.angle, self.r);
                },

                Event::Resized(x, y) => {
                    self.camera.with_view_dimensions_mut(x, y);
                },
                _ => {}
            }
        }
        Action::Continue
    }
}

fn main() {
    let mut sterek = Applicaton::new(State::new((50, 50, 50)));
    sterek.main_loop();
}
