
mod camera;
mod chunk;

use camera::*;
use chunk::*;
use bevy::prelude::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (
            setup,
            setup_camera
        ))
        .add_systems(Update, 
            (camera_movment, camera_look, exit_on_esc))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            range: 200.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0 , 30.0, 1.0),
    ));

    let chunk = Chunk::new(IVec2::ZERO);
    let mesh = chunk.build_mesh();

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.8, 0.3),
            cull_mode: None,  // <- Das ist wichtig!
            ..default()
        })),
        Transform::default(),
    ));
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
