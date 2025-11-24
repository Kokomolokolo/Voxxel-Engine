use bevy::prelude::*;
use crate::chunk::*;
use super::TarrainGenerator;
use super::Biome;
use bevy::math::IVec2;
use noise::{NoiseFn, Perlin, Fbm};
pub struct FeatureGenerator {
    tree_noise: Perlin
}

impl FeatureGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            tree_noise: Perlin::new(seed + 300),
        }
    }
    pub fn generate_trees(
        &self, 
        blocks: &mut [BlockType; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT],
        tarrain: &TarrainGenerator,
        chunk_pos: IVec2
    ) {
    let mut tree_positons: Vec<(usize, usize)> = Vec::new();
        for x in 2..CHUNK_WIDTH -2 { // 2 und -2 da sonst die Blätter chunk übergreifend währen, ein kleiner hack
            for z in 2..CHUNK_WIDTH-2 {
                let world_x = chunk_pos.x * CHUNK_WIDTH as i32 + x as i32;
                let world_z = chunk_pos.y * CHUNK_WIDTH as i32 + z as i32;

                let height = tarrain.get_height(world_x, world_z);
                // Height könnte zu hoch sein
                if height <= 0 || height >= CHUNK_HEIGHT as i32 {
                    continue;
                }
                let ground_idx = Chunk::index(x, height as usize - 1, z);
                if blocks[ground_idx] != BlockType::Grass {
                    continue;
                }

                let tree_noise = self.tree_noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]);

                // Höhere Wahrscheinlichkeit in einem Wald bei biomnoise unter -0.3
                let biom = tarrain.get_biom(world_x, world_z);
                let noise_threshold = 0.7; //biom.get_tree_density();
                
                let min_dist = 4;

                if tree_noise > noise_threshold && height < 50 && height > 23 {
                    let mut too_close = false;
                    for pos in &tree_positons {
                        let dx = (pos.0 as i32 - x as i32).abs();
                        let dz = (pos.1 as i32 - z as i32).abs();

                        if dx < min_dist && dz < min_dist {
                            too_close = true;
                        }
                    }
                    if !too_close {
                        tree_positons.push((x, z));
                        // Baum stamm platzieren
                        for y in 0..5 {
                            let block_y = height as usize + y;
                            if block_y >= CHUNK_HEIGHT {
                                break;  // Nicht über Chunk-Grenze hinaus
                            }
                            let idx = Chunk::index(x, block_y, z);
                            blocks[idx] = BlockType::Wood;
                        }
                        let leaves = self.generate_leave_structure();
                        for leave in leaves {
                            let leaf_y = (height + 5) + leave.1;
                            if leaf_y < 0 || leaf_y >= CHUNK_HEIGHT as i32 {
                                continue;
                            }
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
    // Hilfsfunktionen
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