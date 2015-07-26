extern crate clock_ticks;
extern crate obj;
extern crate nalgebra;
extern crate genmesh;
extern crate std;

use std::fs::File;
use std::io::Read;
use self::genmesh::EmitTriangles;
use glium::{self, Display};
use glium::vertex::VertexBufferAny;


#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    texture: [f32; 2],
}

fn max_by<F>(arr: &[f32], mut cmp: F) -> f32
    where F: FnMut(f32, f32) -> bool {
    let mut max_val = arr[0];
    for val in arr.iter() {
        max_val = if cmp(*val, max_val) { max_val } else { *val };
    }
    max_val
}

/// Returns a vertex buffer that should be rendered as `TrianglesList`.
pub fn load_wavefront(display: &Display, data: &[u8], normalize_coords: bool) -> VertexBufferAny {
    implement_vertex!(Vertex, position, normal, texture);

    let mut data = ::std::io::BufReader::new(data);
    let data = obj::Obj::load(&mut data);

    let mut vertex_data = Vec::new();
    let mut max_coord = 0.0f32;
    for shape in data.object_iter().next().unwrap().group_iter().flat_map(|g| g.indices().iter()) {
        shape.emit_triangles(|tri| {
            for v in [tri.x, tri.y, tri.z].iter() {
                let position = data.position()[v.0];
                let texture = v.1.map(|index| data.texture()[index]);
                let normal = v.2.map(|index| data.normal()[index]);
                max_coord = max_coord.max(max_by(&position, |a, b| a.abs() < b.abs()));
                let texture = texture.unwrap_or([0.0, 0.0]);
                let normal = normal.unwrap_or([0.0, 0.0, 0.0]);

                vertex_data.push(Vertex {
                    position: position,
                    normal: normal,
                    texture: texture,
                })
            }
        })
    }

    if normalize_coords {
        for v in vertex_data.iter_mut() {
            for c in v.position.iter_mut() {
                *c /= max_coord;
            }
        }
    }

    glium::vertex::VertexBuffer::new(display, &*vertex_data.into_boxed_slice()).unwrap().into_vertex_buffer_any()
}

pub fn read_from_obj<'a>(display: &glium::Display, path: &'a str, normalize_coords: bool)
    -> std::io::Result<(glium::vertex::VertexBufferAny, glium::index::NoIndices)> {
    let mut buf = Vec::new();
    match File::open(path) {
        Ok(mut f) => f.read_to_end(&mut buf).and_then(
            |_| Ok((
                load_wavefront(display, &buf, normalize_coords),
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList)
            ))
        ),
        Err(e) => { println!("cannot open file: {}", e); Err(e) }
    }
}

pub fn read_file_content<'a>(path: &'a str) -> std::io::Result<String> {
    let mut content = String::new();
    match File::open(path) {
        Ok(mut f) => f.read_to_string(&mut content).and_then(|_| Ok(content)),
        Err(e) => { println!("cannot open file: {}", e); Err(e) }
    }
}
