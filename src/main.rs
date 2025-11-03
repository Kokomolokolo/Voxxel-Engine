use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (
            setup
        ))
        .run();
}

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 16; // falls ich das später noch ändern will
#[derive(Component)]
struct Chunk {
    pos: IVec2,
    blocks: [BlockType; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT] // 1d Array: [type; size]
}

impl Chunk {
    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_HEIGHT + z * CHUNK_WIDTH * CHUNK_WIDTH
    }
    fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        let idx = Self::index(x, y, z); // groß geschrieben, da eine Funktion von self
        self.blocks[idx]
    }
    fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let idx = Self::index(x, y, z);
        self.blocks[idx] = block;
    }
}
// Weitere Blöcke hier hinzufügen
#[derive(Clone, Copy)]
enum BlockType {
    Air,
    Grass
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // kamera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));

    // light
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            range: 100.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0 , 2.0, 1.0),
    ));
}