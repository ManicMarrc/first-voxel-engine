use bevy::prelude::{
  Component,
  Mesh,
  Vec3,
};
use bevy::render::render_resource::PrimitiveTopology;

use crate::mesh_data::{
  MeshData,
  MeshDataInsert,
};

bitflags::bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct Face: u8 {
    const Front  = 0b000001;
    const Back   = 0b000010;
    const Right  = 0b000100;
    const Left   = 0b001000;
    const Top    = 0b010000;
    const Bottom = 0b100000;
  }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Block {
  pub size: Vec3,
  pub wireframe: bool,
  pub activated: bool,
  pub activated_faces: Face,
}

impl Block {
  pub fn new(size: Vec3, activated_faces: Face) -> Block {
    Block { size, wireframe: false, activated: true, activated_faces }
  }

  pub fn mesh_data(&self) -> MeshData {
    let mut mesh_data = MeshData {
      primitive_topology: if self.wireframe {
        PrimitiveTopology::LineList
      } else {
        PrimitiveTopology::TriangleList
      },
      ..Default::default()
    };

    if self.activated {
      if self.activated_faces.contains(Face::Front) {
        #[rustfmt::skip]
      mesh_data.insert(
        vec![
          ([-self.size.x / 2.0, -self.size.y / 2.0,  self.size.z / 2.0], [ 0.0,  0.0,  1.0], [0.0, 0.0]),
          ([ self.size.x / 2.0, -self.size.y / 2.0,  self.size.z / 2.0], [ 0.0,  0.0,  1.0], [1.0, 0.0]),
          ([ self.size.x / 2.0,  self.size.y / 2.0,  self.size.z / 2.0], [ 0.0,  0.0,  1.0], [1.0, 1.0]),
          ([-self.size.x / 2.0,  self.size.y / 2.0,  self.size.z / 2.0], [ 0.0,  0.0,  1.0], [0.0, 1.0]),
        ],
        vec![0, 1, 2, 3],
        MeshDataInsert::all()
      );
      }
      if self.activated_faces.contains(Face::Back) {
        #[rustfmt::skip]
      mesh_data.insert(
        vec![
          ([-self.size.x / 2.0,  self.size.y / 2.0, -self.size.z / 2.0], [ 0.0,  0.0, -1.0], [1.0, 0.0]),
          ([ self.size.x / 2.0,  self.size.y / 2.0, -self.size.z / 2.0], [ 0.0,  0.0, -1.0], [0.0, 0.0]),
          ([ self.size.x / 2.0, -self.size.y / 2.0, -self.size.z / 2.0], [ 0.0,  0.0, -1.0], [0.0, 1.0]),
          ([-self.size.x / 2.0, -self.size.y / 2.0, -self.size.z / 2.0], [ 0.0,  0.0, -1.0], [1.0, 1.0]),
        ],
        vec![0, 1, 2, 3],
        MeshDataInsert::all()
      );
      }
      if self.activated_faces.contains(Face::Right) {
        #[rustfmt::skip]
      mesh_data.insert(
        vec![
          ([ self.size.x / 2.0, -self.size.y / 2.0, -self.size.z / 2.0], [ 1.0,  0.0,  0.0], [0.0, 0.0]),
          ([ self.size.x / 2.0,  self.size.y / 2.0, -self.size.z / 2.0], [ 1.0,  0.0,  0.0], [1.0, 0.0]),
          ([ self.size.x / 2.0,  self.size.y / 2.0,  self.size.z / 2.0], [ 1.0,  0.0,  0.0], [1.0, 1.0]),
          ([ self.size.x / 2.0, -self.size.y / 2.0,  self.size.z / 2.0], [ 1.0,  0.0,  0.0], [0.0, 1.0]),
        ],
        vec![0, 1, 2, 3],
        MeshDataInsert::all()
      );
      }
      if self.activated_faces.contains(Face::Left) {
        #[rustfmt::skip]
      mesh_data.insert(
        vec![
          ([-self.size.x / 2.0, -self.size.y / 2.0,  self.size.z / 2.0], [-1.0,  0.0,  0.0], [1.0, 0.0]),
          ([-self.size.x / 2.0,  self.size.y / 2.0,  self.size.z / 2.0], [-1.0,  0.0,  0.0], [0.0, 0.0]),
          ([-self.size.x / 2.0,  self.size.y / 2.0, -self.size.z / 2.0], [-1.0,  0.0,  0.0], [0.0, 1.0]),
          ([-self.size.x / 2.0, -self.size.y / 2.0, -self.size.z / 2.0], [-1.0,  0.0,  0.0], [1.0, 1.0]),
        ],
        vec![0, 1, 2, 3],
        MeshDataInsert::all()
      );
      }
      if self.activated_faces.contains(Face::Top) {
        #[rustfmt::skip]
      mesh_data.insert(
        vec![
          ([ self.size.x / 2.0,  self.size.y / 2.0, -self.size.z / 2.0], [ 0.0,  1.0,  0.0], [1.0, 0.0]),
          ([-self.size.x / 2.0,  self.size.y / 2.0, -self.size.z / 2.0], [ 0.0,  1.0,  0.0], [0.0, 0.0]),
          ([-self.size.x / 2.0,  self.size.y / 2.0,  self.size.z / 2.0], [ 0.0,  1.0,  0.0], [0.0, 1.0]),
          ([ self.size.x / 2.0,  self.size.y / 2.0,  self.size.z / 2.0], [ 0.0,  1.0,  0.0], [1.0, 1.0]),
        ],
        vec![0, 1, 2, 3],
        MeshDataInsert::all()
      );
      }
      if self.activated_faces.contains(Face::Bottom) {
        #[rustfmt::skip]
      mesh_data.insert(
        vec![
          ([ self.size.x / 2.0, -self.size.y / 2.0,  self.size.z / 2.0], [ 0.0, -0.0,  0.0], [0.0, 0.0]),
          ([-self.size.x / 2.0, -self.size.y / 2.0,  self.size.z / 2.0], [ 0.0, -0.0,  0.0], [1.0, 0.0]),
          ([-self.size.x / 2.0, -self.size.y / 2.0, -self.size.z / 2.0], [ 0.0, -0.0,  0.0], [1.0, 1.0]),
          ([ self.size.x / 2.0, -self.size.y / 2.0, -self.size.z / 2.0], [ 0.0, -0.0,  0.0], [0.0, 1.0]),
        ],
        vec![0, 1, 2, 3],
        MeshDataInsert::all()
      );
      }
    }

    mesh_data
  }
}

impl From<Block> for Mesh {
  fn from(block: Block) -> Mesh { block.mesh_data().into() }
}

impl From<&Block> for Mesh {
  fn from(block: &Block) -> Mesh { block.mesh_data().into() }
}
