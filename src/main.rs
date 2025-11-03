
mod camera;

use camera::*;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::{Mesh, Indices};


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

fn exit_on_esc(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 16; // falls ich das später noch ändern will
#[derive(Component)]
struct Chunk {
    pos: IVec2,
    blocks: [BlockType; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT] // 1d Array: [type; size]
}

impl Chunk {
    fn new(pos: IVec2) -> Self {
        let mut blocks = [BlockType::Air; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT];
        
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..10 {
                    let block_type = if y < 8 {
                        BlockType::Stone
                    } else {
                        BlockType::Grass
                    };
                    let idx = Self::index(x, y, z);
                    blocks[idx] = block_type
                }
            }
        }
        Self { pos, blocks }
    }
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
    fn build_mesh(&self) {
        // Verticies, normale und indices werden hier gespeichert. Jeder Block schreibt seine werte hier rein.
        // Die gesammten Werte werden in einem gesammten Chunk mesh zurück gegeben.
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..10 {
                    let block: BlockType = self.get_block(x, y, z);
                    if block == BlockType::Air {
                        continue;
                    }
                    // hier weiter mit: Cube rendern: Die create_cube funktion anpassen, dass sie Koordinaten annimmt, wo sie den Zentrum des Cubes malt.
                    // Am Ende: Entweder alle Meshes zu einem Mesh zusammenfassen via PrimitivTopology::TriangleList 
                    // ein neues Chunk mesh machen, dass gesammt zurück gegeben wird. Das ist glaube ich besser.
                    // dann müssten alle verticies, indices und normals in einem gesammt vec gespeichert werden, und dann alle Zusammen eingefügt werden.
                }
            }
         }
    }
}
// Weitere Blöcke hier hinzufügen
#[derive(Clone, Copy, PartialEq)]
enum BlockType {
    Air,
    Grass,
    Stone
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
            range: 100.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(3.0 , 2.0, 1.0),
    ));

    let my_mesh = create_cube_mesh();  // Deine Funktion
    let mesh_handle = meshes.add(my_mesh);  // In Assets einfügen
    
    commands.spawn((
        Mesh3d(mesh_handle),  // Mesh spawnen
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),  // Material (Farbe)
        Transform::from_xyz(0.0, 0.5, 0.0),  // Position
    ));
}

fn create_cube_mesh() -> Mesh {
     let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default()
    );

    let vertices: Vec<[f32; 3]> = vec![
        // top
        [-0.5, 0.5, -0.5], // vertex with index 0
        [0.5, 0.5, -0.5], // vertex with index 1
        [0.5, 0.5, 0.5], // etc. until 23
        [-0.5, 0.5, 0.5],
        // bottom   (-y)
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        // right    (+x)
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
        // left     (-x)
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        // back     (+z)
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
        // forward  (-z)
        [-0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
    ];
    let normals: Vec<[f32; 3]> = vec![
        // Normals for the top side (towards +y)
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // Normals for the bottom side (towards -y)
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        // Normals for the right side (towards +x)
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Normals for the left side (towards -x)
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // Normals for the back side (towards +z)
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Normals for the forward side (towards -z)
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
    ];
    let indices: Vec<u32> = vec![
        0,3,1 , 1,3,2, // triangles making up the top (+y) facing side.
        4,5,7 , 5,6,7, // bottom (-y)
        8,11,9 , 9,11,10, // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
}