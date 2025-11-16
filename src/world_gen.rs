use bevy::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::chunk::{BlockType, CHUNK_WIDTH, CHUNK_HEIGHT, Chunk};

pub struct WorldGenerator {
    tarrain_noise: Perlin,
    detail_noise: Perlin
}

impl WorldGenerator {
    pub fn new(seed: u32, seed2: u32) -> Self {
        Self {
            tarrain_noise: Perlin::new(seed),
            detail_noise: Perlin::new(seed2)
        }
    }
    // pub fn get_height_v1(&self, world_x: i32, world_z: i32) -> i32 {
    //     let scale = 0.05;
    //     let noise_value = self.noise.get([world_x as f64 * scale, world_z as f64 * scale]);
    //     let base_height = 30;
    //     base_height + (noise_value / 2.0 * 30.0) as i32
    // }
    pub fn get_height(&self, world_x: i32, world_z: i32) -> i32 {
        // Basis-Hügel (große Formen)
        let base_height = self.tarrain_noise.get([world_x as f64 * 0.03, world_z as f64 * 0.03]) * 10.0;
        
        // Kleinere Noise für Rauheiten
        let detail = self.detail_noise.get([world_x as f64 * 0.12, world_z as f64 * 0.12]).abs() * 3.0;

        let combined = base_height + detail ;
        let dramatic = combined.abs().powf(1.3) * combined.signum(); 
        // Basis von 30
        30 + dramatic as i32
    }

    pub fn get_block_at(&self, world_x: i32, world_y: i32, world_z: i32, ) -> BlockType {
        let height = self.get_height(world_x, world_z);
        
        if world_y >= height {
            if world_y < 23 { // WATER_LEVEL
                return BlockType::Water;
            }
            return BlockType::Air;
        }

        // Unter Terrain
        if world_y < height - 4 {
            BlockType::Stone
        } else if world_y < height - 2 && world_y > height - 4 {
            BlockType::Dirt
        } else {
            if height <= 24 && height >= 20 {
                BlockType::Sand
            }
            else {
                BlockType::Grass
            }
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

                let tree_noise = self.tarrain_noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]);

                if tree_noise > 0.7 && height < 50 && height > 23 {
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
                            blocks[idx] = BlockType::Wood;
                        }
                        let leaves = self.generate_leave_structure();
                        for leave in leaves {
                            let idx = Chunk::index(
                                (x as i32 + leave.0)as usize ,
                                ((height + 5) + leave.1) as usize,
                                 (z as i32 + leave.2) as usize
                            );
                            blocks[idx] = BlockType::Leaves;
                        }
                    }
                }
            }
        }
    }
    fn generate_leave_structure(&self) -> Vec<(i32, i32, i32)> {
        vec! [
            // Layer -1, unter der Spize
            (-1, -2, 0), (1, -2, 0), (0, -2, 1), (0, -2, -1),
            // Layer 0, auf der ebene der Spitze
            (-1, -1, 1), (0, -1, 1), (1, -1, 1), 
            (1, -1, 0), (-1, -1, 0),
            (-1, -1, -1), (0, -1, -1), (1, -1, -1),
            // Über der Spitze
            (-1, 0, 0), (1, 0, 0), (0, 0, 1), (0, 0, -1),
            (0, 0, 0)
        ]
    }
}