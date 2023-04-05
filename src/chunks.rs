mod block;
mod chunk;

use bevy::math::{
  ivec3,
  Vec3A,
};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::utils::HashMap;

use bracket_noise::prelude::FastNoise;
use chunk::{
  Chunk,
  ChunkNeighbors,
  ChunkNeighborsInfo,
  ChunkUpdate,
};

#[derive(Resource)]
pub struct Noise(pub FastNoise);

#[derive(Resource)]
pub struct ChunkWorldConfig {
  pub y_chunks: i32,
  pub chunk_size: UVec3,
  pub block_size: Vec3,
}

#[derive(Resource)]
pub struct ChunkWorld {
  pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Component)]
pub struct ChunkLoadingPoint {
  pub radius: f32,
}

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(ChunkWorld { chunks: HashMap::new() })
      .add_system(toggle_wireframes_for_chunks)
      .add_system(spawn_chunks_within_radius)
      .add_system(load_chunks_within_radius)
      .add_system(update_chunks.in_base_set(CoreSet::First));
  }
}

fn toggle_wireframes_for_chunks(
  mut commands: Commands,
  mut chunks: Query<(Entity, &mut Chunk)>,
  key: Res<Input<KeyCode>>,
) {
  if key.just_pressed(KeyCode::L) {
    for (chunk_entity, mut chunk) in &mut chunks {
      chunk.wireframe = !chunk.wireframe;
      commands.entity(chunk_entity).insert(ChunkUpdate);
    }
  }
}

fn spawn_chunks_within_radius(
  mut commands: Commands,
  clps: Query<(&Transform, &ChunkLoadingPoint)>,
  noise: Res<Noise>,
  mut mesh_assets: ResMut<Assets<Mesh>>,
  mut standard_material_assets: ResMut<Assets<StandardMaterial>>,
  chunk_world_config: Res<ChunkWorldConfig>,
  mut chunk_world: ResMut<ChunkWorld>,
) {
  for (clp_transform, clp) in &clps {
    let clp_aabb =
      Aabb { center: clp_transform.translation.into(), half_extents: Vec3A::splat(clp.radius) };
    let clp_min = (clp_aabb.min().round()
      / (chunk_world_config.chunk_size.x as f32 * chunk_world_config.block_size.x))
      .as_ivec3();
    let clp_max = (clp_aabb.max().round()
      / (chunk_world_config.chunk_size.x as f32 * chunk_world_config.block_size.x))
      .as_ivec3();

    let mut update_neighbors = false;
    for x in clp_min.x..clp_max.x {
      for y in 0..chunk_world_config.y_chunks {
        for z in clp_min.z..clp_max.z {
          let key = ivec3(x, y, z);
          if !chunk_world.chunks.contains_key(&key) {
            update_neighbors = true;
            let chunk = Chunk::new(
              &noise.0,
              key * chunk_world_config.chunk_size.as_ivec3(),
              chunk_world_config.y_chunks as u32 * chunk_world_config.chunk_size.y,
              chunk_world_config.chunk_size,
              chunk_world_config.block_size,
            );
            let chunk = commands
              .spawn(PbrBundle {
                mesh: mesh_assets.add((&chunk).into()),
                material: standard_material_assets
                  .add(StandardMaterial { base_color: Color::ORANGE_RED, ..Default::default() }),
                transform: Transform::from_translation(
                  key.as_vec3()
                    * chunk_world_config.chunk_size.as_vec3()
                    * chunk_world_config.block_size,
                ),
                ..Default::default()
              })
              .insert(chunk)
              .insert(ChunkUpdate)
              .id();

            chunk_world.chunks.insert(key, chunk);
          }
        }
      }
    }

    if update_neighbors {
      for (key, chunk_entity) in &chunk_world.chunks {
        let chunk_neighbors_info = ChunkNeighborsInfo {
          front: chunk_world.chunks.get(&(*key + ivec3(0, 0, 1))).copied(),
          back: chunk_world.chunks.get(&(*key - ivec3(0, 0, 1))).copied(),
          right: chunk_world.chunks.get(&(*key + ivec3(1, 0, 0))).copied(),
          left: chunk_world.chunks.get(&(*key - ivec3(1, 0, 0))).copied(),
          top: chunk_world.chunks.get(&(*key + ivec3(0, 1, 0))).copied(),
          bottom: chunk_world.chunks.get(&(*key - ivec3(0, 1, 0))).copied(),
        };
        commands.entity(*chunk_entity).insert(chunk_neighbors_info);
      }
    }
  }
}

fn load_chunks_within_radius(
  mut commands: Commands,
  clps: Query<(&Transform, &ChunkLoadingPoint), Changed<Transform>>,
  mut chunks: Query<(Entity, &Transform, &mut Chunk)>,
) {
  for (clp_transform, clp) in &clps {
    let clp_aabb =
      Aabb { center: clp_transform.translation.into(), half_extents: Vec3A::splat(clp.radius) };

    for (chunk_entity, chunk_transform, mut chunk) in &mut chunks {
      let chunk_pos: Vec3A = chunk_transform.translation.into();
      if (chunk_pos.x >= clp_aabb.min().x && chunk_pos.x <= clp_aabb.max().x)
        && (chunk_pos.y >= clp_aabb.min().y && chunk_pos.y <= clp_aabb.max().y)
        && (chunk_pos.z >= clp_aabb.min().z && chunk_pos.z <= clp_aabb.max().z)
      {
        if !chunk.activated {
          chunk.activated = true;
          commands.entity(chunk_entity).insert(ChunkUpdate);
        }
      } else if chunk.activated {
        chunk.activated = false;
        commands.entity(chunk_entity).insert(ChunkUpdate);
      }
    }
  }
}

fn update_chunks(
  mut commands: Commands,
  mut chunks: Query<(Entity, &mut Chunk, &ChunkNeighborsInfo, &Handle<Mesh>), With<ChunkUpdate>>,
  mut mesh_assets: ResMut<Assets<Mesh>>,
) {
  let mut chunks_neighbors = HashMap::new();
  for (chunk_entity, _, chunk_neighbors_info, _) in &chunks {
    let chunk_neighbors = ChunkNeighbors {
      front: chunk_neighbors_info
        .front
        .and_then(|entity| chunks.get(entity).ok())
        .map(|(_, chunk, _, _)| chunk.clone()),
      back: chunk_neighbors_info
        .back
        .and_then(|entity| chunks.get(entity).ok())
        .map(|(_, chunk, _, _)| chunk.clone()),
      right: chunk_neighbors_info
        .right
        .and_then(|entity| chunks.get(entity).ok())
        .map(|(_, chunk, _, _)| chunk.clone()),
      left: chunk_neighbors_info
        .left
        .and_then(|entity| chunks.get(entity).ok())
        .map(|(_, chunk, _, _)| chunk.clone()),
      top: chunk_neighbors_info
        .top
        .and_then(|entity| chunks.get(entity).ok())
        .map(|(_, chunk, _, _)| chunk.clone()),
      bottom: chunk_neighbors_info
        .bottom
        .and_then(|entity| chunks.get(entity).ok())
        .map(|(_, chunk, _, _)| chunk.clone()),
    };
    chunks_neighbors.insert(chunk_entity, chunk_neighbors);
  }

  for (chunk_entity, mut chunk, _, chunk_mesh_handle) in &mut chunks {
    chunk.update(&chunks_neighbors[&chunk_entity]);
    commands.entity(chunk_entity).remove::<ChunkUpdate>();
    mesh_assets.set_untracked(chunk_mesh_handle, chunk.as_ref().into());
  }
}
