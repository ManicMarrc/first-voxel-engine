mod chunks;
mod mesh_data;

use bevy::core_pipeline::tonemapping::{
  DebandDither,
  Tonemapping,
};
use bevy::diagnostic::{
  Diagnostics,
  FrameTimeDiagnosticsPlugin,
};
use bevy::math::{
  uvec3,
  vec3,
};
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bracket_noise::prelude::*;
use chunks::{
  ChunkLoadingPoint,
  ChunkWorldConfig,
  ChunksPlugin,
  Noise,
};
use rand::Rng;
use smooth_bevy_cameras::controllers::fps::{
  FpsCameraBundle,
  FpsCameraController,
  FpsCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

#[derive(Component)]
struct FpsText;

fn main() {
  let mut rng = rand::thread_rng();
  let mut noise = FastNoise::seeded(rng.gen());
  noise.set_noise_type(NoiseType::Perlin);
  noise.set_fractal_octaves(5);
  noise.set_fractal_gain(0.8);
  noise.set_fractal_lacunarity(0.2);
  noise.set_frequency(1.2);
  noise.set_gradient_perterb_amp(2.2);
  noise.set_interp(Interp::Hermite);

  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "First Voxel Engine".to_string(),
        resolution: (800.0, 600.0).into(),
        ..Default::default()
      }),
      ..Default::default()
    }))
    .add_plugin(LookTransformPlugin)
    .add_plugin(FpsCameraPlugin::default())
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(ChunksPlugin)
    .insert_resource(ClearColor(Color::AQUAMARINE))
    .insert_resource(AmbientLight { brightness: 0.1, ..Default::default() })
    .insert_resource(DirectionalLightShadowMap { size: 4096 })
    .insert_resource(Noise(noise))
    .insert_resource(ChunkWorldConfig {
      y_chunks: 2,
      chunk_size: uvec3(8, 8, 8),
      block_size: vec3(0.5, 0.5, 0.5),
    })
    .add_startup_system(setup)
    .add_system(grab_mouse)
    .add_system(draw_fps)
    .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  commands
    .spawn(Camera3dBundle {
      tonemapping: Tonemapping::AcesFitted,
      dither: DebandDither::Enabled,
      ..Default::default()
    })
    .insert(FpsCameraBundle::new(
      FpsCameraController { smoothing_weight: 0.1, ..Default::default() },
      vec3(0.0, 0.0, 5.0),
      vec3(0.0, 0.0, 0.0),
      Vec3::Y,
    ))
    .insert(ChunkLoadingPoint { radius: 40.0 });

  let font = asset_server.load("fonts/fff-forward.ttf");
  commands
    .spawn(
      TextBundle::from_sections([
        TextSection::new(
          "FPS: ",
          TextStyle { font: font.clone(), font_size: 30.0, color: Color::BLACK },
        ),
        TextSection::from_style(TextStyle { font, font_size: 30.0, color: Color::BLACK }),
      ])
      .with_text_alignment(TextAlignment::Left),
    )
    .insert(FpsText);

  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      illuminance: 8000.0,
      // shadows_enabled: true,
      ..Default::default()
    },
    transform: Transform::from_xyz(2.5, 2.5, 2.5).looking_at(Vec3::ZERO, Vec3::Y),
    ..Default::default()
  });
  commands.spawn(DirectionalLightBundle {
    directional_light: DirectionalLight {
      illuminance: 6000.0,
      // shadows_enabled: true,
      ..Default::default()
    },
    transform: Transform::from_xyz(-2.5, 2.5, -1.5).looking_at(Vec3::ZERO, Vec3::Y),
    ..Default::default()
  });
}

fn grab_mouse(mut windows: Query<&mut Window>, key: Res<Input<KeyCode>>) {
  let mut window = windows.single_mut();

  if key.just_pressed(KeyCode::M) {
    if window.cursor.visible {
      window.cursor.visible = false;
      window.cursor.grab_mode = CursorGrabMode::Confined;
    } else {
      window.cursor.visible = true;
      window.cursor.grab_mode = CursorGrabMode::None;
    }
  }
}

fn draw_fps(mut texts: Query<&mut Text, With<FpsText>>, diagnostics: ResMut<Diagnostics>) {
  for mut text in &mut texts {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).and_then(|d| d.smoothed()) {
      text.sections[1].value = fps.round().to_string();
    }
  }
}
