extern crate glium;
extern crate std;

use glium::vertex::{Vertex, VertexBufferAny, PerInstance};
use glium::index::NoIndices;
use glium::VertexBuffer;
use glium::backend::Facade;
use glium::buffer::Mapping;
use std::iter::Iterator;

pub struct InstancedObjects<T>
    where T: Vertex + Copy + Send + 'static {
    vertices: VertexBufferAny,
    indices: NoIndices,
    pub per_instance: VertexBuffer<T>,
}

impl<T> InstancedObjects<T>
    where T: Vertex + Copy + Send + 'static {

    pub fn new<F: Facade> (
        facade: &F,
        vertex_info: (VertexBufferAny, NoIndices),
        per_instance_data: Vec<T>
    ) -> InstancedObjects<T> {
        InstancedObjects {
            vertices: vertex_info.0,
            indices: vertex_info.1,
            per_instance: VertexBuffer::dynamic(facade, &*per_instance_data.into_boxed_slice()).unwrap()
        }
    }

    pub fn get_vertices_data(&self) -> (&VertexBufferAny, PerInstance) {
        (&self.vertices, self.per_instance.per_instance().unwrap())
    }

    pub fn get_indices_data(&self) -> &NoIndices {
        &self.indices
    }

    pub fn update_per_instance_buffer<F>(&mut self, upd_func: F)
        where F: FnOnce(&mut Mapping<[T]>) {
        upd_func(&mut self.per_instance.map());
    }
}
