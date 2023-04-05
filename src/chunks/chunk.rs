use std::sync::mpsc;

use bevy::prelude::{
  Component,
  Entity,
  IVec3,
  Mesh,
  UVec3,
  Vec3,
};
use bevy::render::render_resource::PrimitiveTopology;
use bracket_noise::prelude::FastNoise;
use rayon::prelude::*;

use crate::chunks::block::{
  Block,
  Face,
};
use crate::mesh_data::{
  MeshData,
  MeshDataInsert,
};

const WORLD_TO_NOISE: f32 = 0.027;

#[derive(Component)]
pub struct ChunkUpdate;

#[derive(Component, Default, Debug, Clone)]
pub struct ChunkNeighborsInfo {
  pub front: Option<Entity>,
  pub back: Option<Entity>,
  pub right: Option<Entity>,
  pub left: Option<Entity>,
  pub top: Option<Entity>,
  pub bottom: Option<Entity>,
}

#[derive(Component, Default, Debug, Clone)]
pub struct ChunkNeighbors {
  pub front: Option<Chunk>,
  pub back: Option<Chunk>,
  pub right: Option<Chunk>,
  pub left: Option<Chunk>,
  pub top: Option<Chunk>,
  pub bottom: Option<Chunk>,
}

#[derive(Component, Debug, Clone)]
pub struct Chunk {
  pub size: UVec3,
  pub block_size: Vec3,
  pub wireframe: bool,
  pub blocks: Vec<Block>,
  pub activated: bool,
}

impl Chunk {
  pub fn new(
    noise: &FastNoise,
    noise_offset: IVec3,
    max_y: u32,
    size: UVec3,
    block_size: Vec3,
  ) -> Chunk {
    let blocks = (0..size.x * size.y * size.z)
      .into_par_iter()
      .map(|i| {
        let x = (i / (size.y * size.z)) % size.x;
        let y = (i / size.z) % size.y;
        let z = i % size.z;

        let enabled = y as i32 + noise_offset.y
          <= (noise.get_noise(
            (x as f32 + noise_offset.x as f32) * WORLD_TO_NOISE,
            (z as f32 + noise_offset.z as f32) * WORLD_TO_NOISE,
          ) * max_y as f32)
            .round() as u32 as i32;
        // let enabled = noise.get_noise3d(
        // (x as f32 + noise_offset.x as f32) * WORLD_TO_NOISE,
        // (y as f32 + noise_offset.y as f32) * WORLD_TO_NOISE,
        // (z as f32 + noise_offset.z as f32) * WORLD_TO_NOISE,
        // ) < 0.1;

        let mut block = Block::new(block_size, Face::all());
        if !enabled {
          block.activated = false;
        }
        block
      })
      .collect::<Vec<Block>>();

    Chunk { size, block_size, wireframe: false, blocks, activated: true }
  }

  fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
    if x >= self.size.x as usize || y >= self.size.y as usize || z >= self.size.z as usize {
      return None;
    }
    Some(
      self.blocks[x * self.size.y as usize * self.size.z as usize + y * self.size.z as usize + z],
    )
  }

  pub fn update(&mut self, chunk_neighbors: &ChunkNeighbors) {
    let faces = (0..self.size.x as usize * self.size.y as usize * self.size.z as usize)
      .into_par_iter()
      .map(|i| {
        let x = (i / (self.size.y as usize * self.size.z as usize)) % self.size.x as usize;
        let y = (i / self.size.z as usize) % self.size.y as usize;
        let z = i % self.size.z as usize;

        let mut activated_faces = Face::empty();
        if self.get_block(x, y, z + 1).map_or(
          chunk_neighbors
            .front
            .as_ref()
            .map_or(true, |chunk| chunk.get_block(x, y, 0).map_or(true, |block| !block.activated)),
          |block| !block.activated,
        ) {
          activated_faces.set(Face::Front, true);
        }
        if self.get_block(x, y, z.wrapping_sub(1)).map_or(
          chunk_neighbors.back.as_ref().map_or(true, |chunk| {
            chunk.get_block(x, y, chunk.size.z as usize - 1).map_or(true, |block| !block.activated)
          }),
          |block| !block.activated,
        ) {
          activated_faces.set(Face::Back, true);
        }
        if self.get_block(x + 1, y, z).map_or(
          chunk_neighbors
            .right
            .as_ref()
            .map_or(true, |chunk| chunk.get_block(0, y, z).map_or(true, |block| !block.activated)),
          |block| !block.activated,
        ) {
          activated_faces.set(Face::Right, true);
        }
        if self.get_block(x.wrapping_sub(1), y, z).map_or(
          chunk_neighbors.left.as_ref().map_or(true, |chunk| {
            chunk.get_block(chunk.size.x as usize - 1, y, z).map_or(true, |block| !block.activated)
          }),
          |block| !block.activated,
        ) {
          activated_faces.set(Face::Left, true);
        }
        if self.get_block(x, y + 1, z).map_or(
          chunk_neighbors
            .top
            .as_ref()
            .map_or(true, |chunk| chunk.get_block(x, 0, z).map_or(true, |block| !block.activated)),
          |block| !block.activated,
        ) {
          activated_faces.set(Face::Top, true);
        }
        if self.get_block(x, y.wrapping_sub(1), z).map_or(
          chunk_neighbors.bottom.as_ref().map_or(true, |chunk| {
            chunk.get_block(x, chunk.size.y as usize - 1, z).map_or(true, |block| !block.activated)
          }),
          |block| !block.activated,
        ) {
          activated_faces.set(Face::Bottom, true);
        }

        activated_faces
      })
      .collect::<Vec<Face>>();

    self.blocks.par_iter_mut().zip(faces).for_each(|(block, activated_faces)| {
      block.wireframe = self.wireframe;
      block.activated_faces = activated_faces;
    });
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
      let (sender, receiver) = mpsc::channel();
      self.blocks.par_iter().enumerate().for_each_with(sender, |sender, (i, block)| {
        let x = ((i / (self.size.y as usize * self.size.z as usize)) % self.size.x as usize) as f32;
        let y = ((i / self.size.z as usize) % self.size.y as usize) as f32;
        let z = (i % self.size.z as usize) as f32;

        let block_mesh_data = block.mesh_data();
        sender
          .send((
            block_mesh_data
              .vertex_positions
              .into_par_iter()
              .zip(block_mesh_data.vertex_normals)
              .zip(block_mesh_data.vertex_uvs)
              .map(|((p, n), u)| {
                (
                  [
                    p[0] + x * self.block_size.x,
                    p[1] + y * self.block_size.y,
                    p[2] + z * self.block_size.z,
                  ],
                  n,
                  u,
                )
              })
              .collect::<Vec<([f32; 3], [f32; 3], [f32; 2])>>(),
            block_mesh_data.indices,
          ))
          .unwrap();
      });

      for (vertices, indices) in receiver.iter() {
        mesh_data.insert(vertices, indices, MeshDataInsert::INDICES_OFFSET);
      }
    }

    mesh_data
  }
}

impl From<Chunk> for Mesh {
  fn from(chunk: Chunk) -> Mesh { chunk.mesh_data().into() }
}

impl From<&Chunk> for Mesh {
  fn from(chunk: &Chunk) -> Mesh { chunk.mesh_data().into() }
}
