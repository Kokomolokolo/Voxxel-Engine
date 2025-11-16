
mod camera;
mod chunk;
mod hud;
mod player;

use camera::*;
use chunk::*;
use hud::*;
use player::*;
use bevy::prelude::*;
// TODO / BUGS
// Wenn ein in der luft schwebendes teil mit nichts verbunden ist verschidet es(vorallem bei chunk grenzen)

fn main() {
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        canvas: Some("#glcanvas".to_string()),
        fit_canvas_to_parent: true,
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
        .insert_resource(ChunkManager::new())
        .add_plugins(HudPlugin)
        .add_systems(Startup, (
            setup,
            setup_camera,
            setup_player,
        ))
        .add_systems(Update, 
            (
                camera_look,
                exit_on_esc, 
                update_chunks, 
                lock_cursor_on_click,
                // camera_follow_player,
                player_movement,
                // player_physics,
                player_mine_place,
            ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    // Himmel
    commands.insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.92))); // Hellblau



    // commands.spawn((
    //     Mesh3d(meshes.add(mesh)),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color: Color::WHITE,
    //         cull_mode: None,  // <- Das ist wichtig!
    //         ..default()
    //     })),
    //     Transform::default(),
    // ));
     // let my_mesh = create_cube_mesh(Vec3::ZERO);  // Deine Funktion
    // let mesh_handle = meshes.add(my_mesh);  // In Assets einfügen
    
    // commands.spawn((
    //     Mesh3d(mesh_handle),  // Mesh spawnen
    //     MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),  // Material (Farbe)
    //     Transform::from_xyz(0.0, 0.5, 0.0),  // Position
    // ));
}
fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
