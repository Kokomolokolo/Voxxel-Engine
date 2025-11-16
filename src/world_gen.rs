use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::chunk::{BlockType, CHUNK_WIDTH, CHUNK_HEIGHT, Chunk};

pub struct WorldGenerator {
    noise: Perlin
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed)
        }
    }
    pub fn get_height(&self, world_x: i32, world_z: i32) -> i32 {
        let scale = 0.05;
        let noise_value = self.noise.get([world_x as f64 * scale, world_z as f64 * scale]);
        let base_height = 30;
        base_height + (noise_value / 2.0 * 30.0) as i32
    }

    pub fn get_block_at(&self, world_x: i32, world_y: i32, world_z: i32, ) -> BlockType {
        let height = self.get_height(world_x, world_z);
        
        if world_y >= height {
            // Über Terrain
            // if y < 25 { // WATER_LEVEL
            //     return BlockType::Water;
            // }
            return BlockType::Air;
        }

        // Unter Terrain
        if world_y < height - 2 {
            BlockType::Stone
        } else {
            BlockType::Grass
        }
    }

    pub fn generate_trees(&self, blocks: &mut [BlockType; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT], chunk_pos: IVec2) {
        let mut tree_positons: Vec<(usize, usize)> = Vec::new();
        for x in 2..CHUNK_WIDTH -2 { // 2 und -2 da sonst die Blätter chunk übergreifend währen, ein kleiner hack
            for z in 2..CHUNK_WIDTH-2 {
                let world_x = chunk_pos.x * CHUNK_WIDTH as i32 + x as i32;
                let world_z = chunk_pos.y * CHUNK_WIDTH as i32 + z as i32;

                let height = self.get_height(world_x, world_z);

                let ground_idx = Chunk::index(x, height as usize - 1, z);
                if blocks[ground_idx] != BlockType::Grass {
                    continue;
                }

                let tree_noise = self.noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]);

                if tree_noise > 0.9 && height < 50 {
                    let mut too_close = false;
                    for pos in &tree_positons {
                        let dx = (pos.0 as i32 - x as i32).abs();
                        let dz = (pos.1 as i32 - z as i32).abs();

                        if dx < 5 && dz < 5 {
                            too_close = true;
                        }
                    }
                    if !too_close {
                        tree_positons.push((x, z));
                        // Baum stamm platzieren
                        for y in 0..5 {
                            let block_y = height as usize + y;
                            let idx = Chunk::index(x, block_y, z);
                            blocks[idx] = BlockType::Wood
                        }
                    }
                }
            }
        }
    }
}