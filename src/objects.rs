extern crate glium;

use glium::vertex::{Vertex, VertexBufferAny, PerInstance};
use glium::index::NoIndices;
use glium::VertexBuffer;
use glium::backend::Facade;
use glium::buffer::Mapping;

pub struct InstancedObjects<T>
    where T: Vertex + Copy + Send + 'static {
    vertices: VertexBufferAny,
    indices: NoIndices,
    per_instance: VertexBuffer<T>,
}

impl<T> InstancedObjects<T>
    where T: Vertex + Copy + Send + 'static {
    pub fn new<F: Facade, G: FnMut() -> Vec<T>>(
        facade: &F,
        verts: VertexBufferAny,
        indices: NoIndices,
        mut per_instance_generator: G
    ) -> InstancedObjects<T> {
        InstancedObjects {
            vertices: verts,
            indices: indices,
            per_instance: VertexBuffer::dynamic(facade, per_instance_generator())
        }
    }

    pub fn get_vertices_data(&self) -> (&VertexBufferAny, PerInstance) {
        (&self.vertices, self.per_instance.per_instance_if_supported().unwrap())
    }

    pub fn get_indices_data(&self) -> &NoIndices {
        &self.indices
    }

    pub fn update_per_instance_buffer<F>(&mut self, mut upd_func: F) where F: FnMut(&mut Mapping<T>) {
        upd_func(&mut self.per_instance.map());
    }
}
