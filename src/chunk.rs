use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use std::collections::HashMap;

//use noise::{Fbm, Perlin};
use crate::world_gen::*;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 128; // falls ich das später noch ändern will


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
    Stone,
    Wood,
    Leaves,
    Water,
    Dirt,
    Sand,
}

impl Chunk {
    pub fn new(pos: IVec2, generator: &WorldGenerator) -> Self {
        let mut blocks = [BlockType::Air; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT];

        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                let world_x = pos.x * CHUNK_WIDTH as i32 + x as i32;
                let world_z = pos.y * CHUNK_WIDTH as i32 + z as i32;
                
                for y in 0..CHUNK_HEIGHT {
                    let idx = Self::index(x, y, z);
                    blocks[idx] = generator.get_block_at(world_x, y as i32, world_z)
                }
            }
        }
        generator.generate_trees(&mut blocks, pos);
        Self { pos, blocks }
    }
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        x + z * CHUNK_WIDTH + y * CHUNK_WIDTH * CHUNK_WIDTH
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        let idx = Self::index(x, y, z); // groß geschrieben, da eine Funktion von self
        self.blocks[idx]
    }
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let idx = Self::index(x, y, z);
        self.blocks[idx] = block;
    }
    pub fn is_solid(&self, x: i32, y: i32, z: i32 ) -> bool {
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
    
    // Über chunkgrenzen hinaus checken. Habe darauf aber gerade keine Lust mehr.
    pub fn is_solid_global(&self, 
        x: i32, 
        y: i32, 
        z: i32,
        neighbors: &HashMap<IVec2, &Chunk>  // Alle Nachbar-Chunks
    ) -> bool {
        // wenn y außerhalb: Keine Nachbarn noch oben oder unten
        if y < 0 || y >= CHUNK_HEIGHT as i32 {
            return false;
        }
        // Innerhalb dieses Chunks?
        if x >= 0 && x < CHUNK_WIDTH as i32 && z >= 0 && z < CHUNK_WIDTH as i32 {
            return self.is_solid(x, y, z);
        }
        let chunk_offset = IVec2::new(
            x.div_euclid(CHUNK_WIDTH as i32), 
            z.div_euclid(CHUNK_WIDTH as i32)
        );
        let local_x = x.rem_euclid(CHUNK_WIDTH as i32);
        let local_z = z.rem_euclid(CHUNK_WIDTH as i32);
        let neighbor_pos = self.pos + chunk_offset;
        if let Some(neighbor) = neighbors.get(&neighbor_pos) {
            neighbor.is_solid(local_x, y, local_z)
        } else {
            // Chunk nicht geladen -> als solid behandeln (keine Fläche rendern)
            true
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
        RenderAssetUsages::default(),
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
        BlockType::Wood => [0.6, 0.4, 0.2, 1.0],
        BlockType::Air => [1.0, 1.0, 1.0, 1.0],
        BlockType::Leaves => [0.15, 0.6, 0.2, 1.0],
        BlockType::Water => [0.2, 0.4, 0.8, 0.6],
        BlockType::Dirt => [0.5, 0.35, 0.2, 1.0],
        BlockType::Sand => [0.9, 0.8, 0.6, 1.0],
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
