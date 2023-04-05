use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bitflags::bitflags;

bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct MeshDataInsert: u8 {
    const INDICES_OFFSET             = 0b01;
    const INDICES_PRIMITIVE_TOPOLOGY = 0b10;
  }
}

#[derive(Default, Debug, Clone)]
pub struct MeshData {
  pub primitive_topology: PrimitiveTopology,
  pub vertex_positions: Vec<[f32; 3]>,
  pub vertex_normals: Vec<[f32; 3]>,
  pub vertex_uvs: Vec<[f32; 2]>,
  pub indices: Vec<u32>,
}

impl MeshData {
  fn is_vertices_in_sync(&self) -> bool {
    self.vertex_positions.len() == self.vertex_normals.len()
      && self.vertex_normals.len() == self.vertex_uvs.len()
  }

  pub fn insert(
    &mut self,
    vertices: Vec<([f32; 3], [f32; 3], [f32; 2])>,
    indices: Vec<u32>,
    insert: MeshDataInsert,
  ) {
    assert!(self.is_vertices_in_sync());
    let indices_offset = if insert.contains(MeshDataInsert::INDICES_OFFSET) {
      self.vertex_positions.len() as u32
    } else {
      0
    };
    #[rustfmt::skip]
    let indices = if insert.contains(MeshDataInsert::INDICES_PRIMITIVE_TOPOLOGY) {
      if self.primitive_topology == PrimitiveTopology::LineList {
        (0..indices.len() * 2).map(|i| {
          if i % 2 == 0 { indices[i / 2] + indices_offset }
          else { indices[(i / 2 + 1) % indices.len()] + indices_offset }
        }).collect()
      } else if self.primitive_topology == PrimitiveTopology::TriangleList && indices.len() == 4 {
        vec![
          indices[0] + indices_offset, indices[1] + indices_offset, indices[2] + indices_offset, 
          indices[2] + indices_offset, indices[3] + indices_offset, indices[0] + indices_offset, 
        ]
      } else {
        unimplemented!();
      }
    } else {
      indices.into_iter().map(|index| index + indices_offset).collect()
    };
    self.indices.extend(&indices);
    self.vertex_positions.extend(vertices.iter().map(|(vertex_positions, _, _)| vertex_positions));
    self.vertex_normals.extend(vertices.iter().map(|(_, vertex_normals, _)| vertex_normals));
    self.vertex_uvs.extend(vertices.iter().map(|(_, _, vertex_uvs)| vertex_uvs));
  }
}

impl From<MeshData> for Mesh {
  fn from(mesh_data: MeshData) -> Mesh {
    assert!(mesh_data.is_vertices_in_sync());
    let mut mesh = Mesh::new(mesh_data.primitive_topology);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.vertex_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.vertex_normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.vertex_uvs);
    mesh.set_indices(Some(Indices::U32(mesh_data.indices)));
    mesh
  }
}
