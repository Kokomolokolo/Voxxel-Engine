
mod camera;

use camera::*;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};


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
const CHUNK_HEIGHT: usize = 64; // falls ich das später noch ändern will
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
                for y in 0..CHUNK_HEIGHT / 2 {
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
        x + y * CHUNK_WIDTH + z * CHUNK_WIDTH * CHUNK_HEIGHT
    }
    fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        let idx = Self::index(x, y, z); // groß geschrieben, da eine Funktion von self
        self.blocks[idx]
    }
    fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let idx = Self::index(x, y, z);
        self.blocks[idx] = block;
    }
    fn is_solid(&self, x: i32, y: i32, z: i32 ) -> bool {
        if x >= CHUNK_WIDTH as i32 || x < 0 || 
            y >= CHUNK_HEIGHT as i32 || y < 0 || 
            z >= CHUNK_WIDTH as i32 || z < 0 { // Chunk grenzen erkennen. Wenn nicht mehr in Chunk erstmal als "leer"
             return false
        }
        if self.get_block(x as usize, y as usize, z as usize) == BlockType::Air {
            return false
        } else {
            return true
        }
    }
    fn build_mesh(&self) -> Mesh {
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
                    // Am Ende: Entweder alle Meshes zu einem Mesh zusammenfassen via PrimitivTopology::TriangleList 
                    // ein neues Chunk mesh machen, dass gesammt zurück gegeben wird. Das ist glaube ich besser.
                    // dann müssten alle verticies, indices und normals in einem gesammt vec gespeichert werden, und dann alle Zusammen eingefügt werden.
                    let pos = Vec3::new(x as f32, y as f32, z as f32);
                    add_cube_faces(
                        pos, 
                        &mut vertices, 
                        &mut normals, 
                        &mut indices,
                        !self.is_solid(x as i32, y as i32 + 1, z as i32),
                        !self.is_solid(x as i32, y as i32 - 1, z as i32),
                        !self.is_solid(x as i32 + 1, y as i32, z as i32),
                        !self.is_solid(x as i32 - 1 ,y as i32, z as i32),
                        !self.is_solid(x as i32, y as i32, z as i32 + 1),
                        !self.is_solid(x as i32, y as i32, z as i32 - 1),
                    );

                }
            }
         }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, 
        RenderAssetUsages::default()
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    
    mesh
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

fn add_cube_faces(
    pos: Vec3,
    vertices: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    render_top: bool,
    render_bottom: bool,
    render_right: bool,
    render_left: bool,
    render_back: bool,
    render_front: bool,
) {
    // Top face (+y)
    if render_top {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x - 0.5, pos.y + 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y + 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y + 0.5, pos.z + 0.5],
            [pos.x - 0.5, pos.y + 0.5, pos.z + 0.5],
        ]);
        normals.extend_from_slice(&[
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        indices.extend_from_slice(&[
            start, start+3, start+1,
            start+1, start+3, start+2,
        ]);
    }
    
    // Bottom face (-y)
    if render_bottom {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x - 0.5, pos.y - 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y - 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y - 0.5, pos.z + 0.5],
            [pos.x - 0.5, pos.y - 0.5, pos.z + 0.5],
        ]);
        normals.extend_from_slice(&[
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ]);
        indices.extend_from_slice(&[
            start, start+1, start+3,
            start+1, start+2, start+3,
        ]);
    }
    
    // Right face (+x)
    if render_right {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x + 0.5, pos.y - 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y - 0.5, pos.z + 0.5],
            [pos.x + 0.5, pos.y + 0.5, pos.z + 0.5],
            [pos.x + 0.5, pos.y + 0.5, pos.z - 0.5],
        ]);
        normals.extend_from_slice(&[
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]);
        indices.extend_from_slice(&[
            start, start+3, start+1,
            start+1, start+3, start+2,
        ]);
    }
    
    // Left face (-x)
    if render_left {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x - 0.5, pos.y - 0.5, pos.z - 0.5],
            [pos.x - 0.5, pos.y - 0.5, pos.z + 0.5],
            [pos.x - 0.5, pos.y + 0.5, pos.z + 0.5],
            [pos.x - 0.5, pos.y + 0.5, pos.z - 0.5],
        ]);
        normals.extend_from_slice(&[
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ]);
        indices.extend_from_slice(&[
            start, start+1, start+3,
            start+1, start+2, start+3,
        ]);
    }
    
    // Back face (+z)
    if render_back {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x - 0.5, pos.y - 0.5, pos.z + 0.5],
            [pos.x - 0.5, pos.y + 0.5, pos.z + 0.5],
            [pos.x + 0.5, pos.y + 0.5, pos.z + 0.5],
            [pos.x + 0.5, pos.y - 0.5, pos.z + 0.5],
        ]);
        normals.extend_from_slice(&[
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ]);
        indices.extend_from_slice(&[
            start, start+3, start+1,
            start+1, start+3, start+2,
        ]);
    }
    
    // Front face (-z)
    if render_front {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x - 0.5, pos.y - 0.5, pos.z - 0.5],
            [pos.x - 0.5, pos.y + 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y + 0.5, pos.z - 0.5],
            [pos.x + 0.5, pos.y - 0.5, pos.z - 0.5],
        ]);
        normals.extend_from_slice(&[
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ]);
        indices.extend_from_slice(&[
            start, start+1, start+3,
            start+1, start+2, start+3,
        ]);
    }
}