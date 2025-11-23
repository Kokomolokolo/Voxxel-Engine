
mod camera;
mod chunk;
mod hud;
mod player;
mod chunk_manager;
//mod world_gen;
mod world;

use camera::CameraPlugin;
use chunk_manager::ChunkManagerPlugin;
use hud::HudPlugin;
use player::PlayerPlugin;

use bevy::prelude::*;
// TODO / BUGS
// Wenn ein in der luft schwebendes teil mit nichts verbunden ist verschidet es(vorallem bei chunk grenzen)

fn main() {
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        canvas: Some("#glcanvas".to_string()),
        resolution: (1200., 800.).into(),
        // fit_canvas_to_parent: true,
        ..default()
    };
    
    #[cfg(not(target_arch = "wasm32"))]
    let window = Window {
        title: "Voxxel Engine".to_string(),
        resolution: (1280., 720.).into(),
        ..default()
    };
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_plugins((
            PlayerPlugin,
            CameraPlugin,
            ChunkManagerPlugin,
            HudPlugin,
        ))
        .add_systems(Startup, setup,)
        .add_systems(Update, exit_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    // light
    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,  // Helligkeit
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,  // 45° nach unten
            std::f32::consts::FRAC_PI_4,   // 45° zur Seite
            0.0
        )),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 900.0, 
        ..default()
    });
    // // Himmel
    commands.insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.92))); // Hellblau




}
fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
