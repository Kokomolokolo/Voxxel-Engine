use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

use noise::{NoiseFn, Perlin};
//use noise::{Fbm, Perlin};

use std::collections::HashMap;


pub const CHUNK_WIDTH: usize = 16;
const CHUNK_HEIGHT: usize = 64; // falls ich das später noch ändern will

#[cfg(not(target_arch = "wasm32"))]
const RENDER_DISTACE: i32 = 12;
#[cfg(target_arch = "wasm32")]
const RENDER_DISTACE: i32 = 6;

#[derive(Component)]
pub struct Chunk {
    pos: IVec2,
    blocks: [BlockType; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT] // 1d Array: [type; size]
}
// Weitere Blöcke hier hinzufügen
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockType {
    Air,
    Grass,
    Stone
}
#[derive(Resource)]
pub struct ChunkManager {
    chunks: HashMap<IVec2, Entity>,
    noise: Perlin,
} // Hier weiter machen. Die Positionen der Chunks müssen gespeichert werden, und in pos gespeichert werden.
// Sie werden dann über Bevy transform an den richtigen Ort platziert, wenn das das einfachste ist.

impl Chunk {
    pub fn new(pos: IVec2, noise: &Perlin) -> Self {
        let mut blocks = [BlockType::Air; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT];
        
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let world_x = pos.x * CHUNK_WIDTH as i32 + x as i32;
                let world_z = pos.y * CHUNK_WIDTH as i32 + z as i32;
                
                let scale = 0.05;
                let noise_value = noise.get([world_x as f64 * scale, world_z as f64 * scale]);
                let base_height = 20;
                //let height = ((noise_value + 1.0) / 2.0 * 30.0) as i32;
                let height = base_height + (noise_value / 2.0 * 30.0) as i32;

                if x == 0 && z == 0 {
                    println!("Chunk {:?}: noise={}, height={}", pos, noise_value, height);
                }

                for y in 0..height {
                    let block_type = if y < height - 2 { // die obersten 2 Blöcke sind immer Gras
                        BlockType::Stone
                    } else {
                        BlockType::Grass
                    };
                    let idx = Self::index(x, y as usize, z);
                    blocks[idx] = block_type
                }
            }
        }
        Self { pos, blocks }
    }
    fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_WIDTH + y * CHUNK_WIDTH * CHUNK_WIDTH
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
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
    pub fn build_mesh(&self) -> Mesh {
        // Verticies, normale und indices werden hier gespeichert. Jeder Block schreibt seine werte hier rein.
        // Die gesammten Werte werden in einem gesammten Chunk mesh zurück gegeben.
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in 0..CHUNK_HEIGHT {
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
                        block,
                        &mut vertices, 
                        &mut normals, 
                        &mut indices,
                        &mut colors, 
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    
    mesh
    }
}


fn add_cube_faces(
    pos: Vec3,
    block_type: BlockType, 
    vertices: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    colors: &mut Vec<[f32; 4]>,
    render_top: bool,
    render_bottom: bool,
    render_right: bool,
    render_left: bool,
    render_back: bool,
    render_front: bool,
) {
    let color = match block_type {
        BlockType::Grass => [0.3, 0.8, 0.3, 1.0],
        BlockType::Stone => [1.0, 1., 1., 1.0],
        BlockType::Air => [1.0, 1.0, 1.0, 1.0],
    };
    
    // Top face (+y)
    if render_top {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x, pos.y + 1.0, pos.z],
            [pos.x + 1.0, pos.y + 1.0, pos.z],
            [pos.x + 1.0, pos.y + 1.0, pos.z + 1.0],
            [pos.x, pos.y + 1.0, pos.z + 1.0],
        ]);
        normals.extend_from_slice(&[
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]);
        colors.extend_from_slice(&[
            color,
            color,
            color,
            color,
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
            [pos.x, pos.y, pos.z],
            [pos.x + 1.0, pos.y, pos.z],
            [pos.x + 1.0, pos.y, pos.z + 1.0],
            [pos.x, pos.y, pos.z + 1.0],
        ]);
        normals.extend_from_slice(&[
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ]);
        colors.extend_from_slice(&[
            color,
            color,
            color,
            color,
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
            [pos.x + 1.0, pos.y, pos.z],
            [pos.x + 1.0, pos.y, pos.z + 1.0],
            [pos.x + 1.0, pos.y + 1.0, pos.z + 1.0],
            [pos.x + 1.0, pos.y + 1.0, pos.z],
        ]);
        normals.extend_from_slice(&[
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        ]);
        colors.extend_from_slice(&[
            color,
            color,
            color,
            color,
        ]);
        indices.extend_from_slice(&[
            start, start+1, start+3,
            start+1, start+2, start+3,
        ]);
    }
    
    // Left face (-x)
    if render_left {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x, pos.y, pos.z],
            [pos.x, pos.y, pos.z + 1.0],
            [pos.x, pos.y + 1.0, pos.z + 1.0],
            [pos.x, pos.y + 1.0, pos.z],
        ]);
        normals.extend_from_slice(&[
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        ]);
        colors.extend_from_slice(&[
            color,
            color,
            color,
            color,
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
            [pos.x, pos.y, pos.z + 1.0],
            [pos.x, pos.y + 1.0, pos.z + 1.0],
            [pos.x + 1.0, pos.y + 1.0, pos.z + 1.0],
            [pos.x + 1.0, pos.y, pos.z + 1.0],
        ]);
        normals.extend_from_slice(&[
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ]);
        colors.extend_from_slice(&[
            color,
            color,
            color,
            color,
        ]);
        indices.extend_from_slice(&[
            start, start+1, start+3,
            start+1, start+2, start+3,
        ]);
    }
    
    // Front face (-z)
    if render_front {
        let start = vertices.len() as u32;
        vertices.extend_from_slice(&[
            [pos.x, pos.y, pos.z],
            [pos.x, pos.y + 1.0, pos.z],
            [pos.x + 1.0, pos.y + 1.0, pos.z],
            [pos.x + 1.0, pos.y, pos.z],
        ]);
        normals.extend_from_slice(&[
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ]);
        colors.extend_from_slice(&[
            color,
            color,
            color,
            color,
        ]);
        indices.extend_from_slice(&[
            start, start+1, start+3,
            start+1, start+2, start+3,
        ]);
    }
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            noise: Perlin::new(12345)
        }
    }
    pub fn spawn_chunk(
        &mut self,
        pos: IVec2,
        commands: &mut Commands,           // Zum Entities erstellen
        meshes: &mut ResMut<Assets<Mesh>>, // Zum Mesh speichern
        materials: &mut ResMut<Assets<StandardMaterial>>, // Zum Material speichern
    ) {
        if self.chunks.contains_key(&pos) { // wenn der Chunk bereits existiert, dann fertig
            return
        }

        let chunk = Chunk::new(pos, &self.noise);
        let mesh = chunk.build_mesh();

        // let mesh_handle = meshes.add(mesh); // Damit verfügbar in Res<Mesh>

        let entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                cull_mode: None,  // <- Das ist wichtig!
                ..default()
            })),
            Transform::from_xyz(
                pos.x as f32 * CHUNK_WIDTH as f32,
                0.0,
                pos.y as f32 * CHUNK_WIDTH as f32
            ),
            chunk,
        )).id();
        self.chunks.insert(pos, entity);
    }

    pub fn get_world_block(&self, world_pos: Vec3, chunk_query: &Query<&Chunk>) -> Option<BlockType> { // da möglicherweise der Block nicht geladen ist etc
        let chunk_pos = IVec2::new(
            (world_pos.x / CHUNK_WIDTH as f32).floor() as i32,
            (world_pos.z / CHUNK_WIDTH as f32).floor() as i32,
        );
        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as f32) as usize; // rem_euclid ist ähnlich wie %, funktioniert mit neg Zahlen
        let local_y = world_pos.y as usize;
        let local_z = world_pos.z.rem_euclid(CHUNK_WIDTH as f32) as usize;

        let entity = *self.chunks.get(&chunk_pos)?; // da habe ich keine ahnung was das alles macht. mit * wert der poiters entnommen
        let chunk = chunk_query.get(entity).ok()?;
        
        Some(chunk.get_block(local_x, local_y, local_z))
    }

    pub fn get_world_block_mut(&self, world_pos: Vec3, chunk_query: &Query<(&mut Chunk, &Mesh3d)>) -> Option<BlockType> { // da möglicherweise der Block nicht geladen ist etc
        let chunk_pos = IVec2::new(
            (world_pos.x / CHUNK_WIDTH as f32).floor() as i32,
            (world_pos.z / CHUNK_WIDTH as f32).floor() as i32,
        );
        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as f32) as usize; // rem_euclid ist ähnlich wie %, funktioniert mit neg Zahlen
        let local_y = world_pos.y as usize;
        let local_z = world_pos.z.rem_euclid(CHUNK_WIDTH as f32) as usize;

        let entity = *self.chunks.get(&chunk_pos)?; // da habe ich keine ahnung was das alles macht. mit * wert der poiters entnommen
        let (chunk, _) = chunk_query.get(entity).ok()?;
        
        Some(chunk.get_block(local_x, local_y, local_z))
    }
    pub fn set_world_block(
        &mut self,
        world_pos: Vec3,
        block_type: BlockType,
        chunk_query: &mut Query<(&mut Chunk, &Mesh3d)>,
        meshes: &mut ResMut<Assets<Mesh>>
    ) {
        let chunk_pos = IVec2::new(
            (world_pos.x / CHUNK_WIDTH as f32).floor() as i32,
            (world_pos.z / CHUNK_WIDTH as f32).floor() as i32,
        );
        let local_x = world_pos.x.rem_euclid(CHUNK_WIDTH as f32) as usize; // rem_euclid ist ähnlich wie %, funktioniert mit neg Zahlen
        let local_y = world_pos.y as usize;
        let local_z = world_pos.z.rem_euclid(CHUNK_WIDTH as f32) as usize;

        let Some(&entity) = self.chunks.get(&chunk_pos) else {
            return
        };

        let Ok((mut chunk, mesh_handle)) = chunk_query.get_mut(entity) else {
            return
        };

        chunk.set_block(local_x, local_y, local_z, block_type);

        let new_mesh = chunk.build_mesh();

        if let Some(mesh_asset) = meshes.get_mut(&mesh_handle.0) {
            *mesh_asset = new_mesh;
        }
    }
}

pub fn update_chunks(
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Transform, With<Camera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    // Camera position in chunk cords umrechnen
    let camera_chunk = IVec2::new(
        (camera_transform.translation.x / CHUNK_WIDTH as f32).floor() as i32,
        (camera_transform.translation.z / CHUNK_WIDTH as f32).floor() as i32,
    );

    for x in -RENDER_DISTACE..RENDER_DISTACE {
        for z in -RENDER_DISTACE..RENDER_DISTACE {
            let chunk_pos = camera_chunk + ivec2(x, z);
            chunk_manager.spawn_chunk(chunk_pos, &mut commands, &mut meshes, &mut materials);
        }
    }
    despawn_chunks(camera_query, chunk_manager, commands);
}

pub fn despawn_chunks(
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut commands: Commands
) {
    let mut to_remove = Vec::new();
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    
    let camera_chunk = IVec2::new(
        (camera_transform.translation.x / CHUNK_WIDTH as f32).floor() as i32,
        (camera_transform.translation.z / CHUNK_WIDTH as f32).floor() as i32,
    );
    for chunk in &chunk_manager.chunks {
        let chunk_pos = chunk.0;
        if (chunk_pos.x - camera_chunk.x).abs() > RENDER_DISTACE + 2 
            || (chunk_pos.y - camera_chunk.y).abs() > RENDER_DISTACE + 2 { // +2 für einen buffer, falls bewegung zwischen chunks
            to_remove.push(*chunk_pos); // da chunk.0 keine referenz ist
        }
    }
    for pos in to_remove {
        if let Some(entity) = chunk_manager.chunks.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}