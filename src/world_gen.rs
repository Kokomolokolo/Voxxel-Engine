use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Fbm, MultiFractal};

use crate::chunk::{BlockType, CHUNK_WIDTH, CHUNK_HEIGHT, Chunk};

pub struct WorldGenerator {
    tarrain_noise: Perlin,
    detail_noise: Perlin,
    pub biom_noise: Fbm<Perlin>, // pub damit ich im HUD anzeigen kann, welches Biom
    cave_noise: Perlin,
}

impl WorldGenerator {
    pub fn new(seed: u32, seed2: u32) -> Self {
        Self {
            tarrain_noise: Perlin::new(seed),
            detail_noise: Perlin::new(seed2),
            biom_noise: Fbm::<Perlin>::new(seed + 200)
                .set_octaves(1)           // Je geringer dest glatter
                .set_frequency(0.0025),
            cave_noise: Perlin::new(seed + 500)
        }
    }
    // pub fn get_height_v1(&self, world_x: i32, world_z: i32) -> i32 {
    //     let scale = 0.05;
    //     let noise_value = self.noise.get([world_x as f64 * scale, world_z as f64 * scale]);
    //     let base_height = 30;
    //     base_height + (noise_value / 2.0 * 30.0) as i32
    // }
    pub fn get_height(&self, world_x: i32, world_z: i32) -> i32 {
        const SPIKY_MOUNTAINS: f64 = 0.95;
        const MOUNTAIN_HIGH: f64 = 0.7;
        const MOUNTAIN_MID: f64 = 0.5;
        const HILLS: f64 = -0.5;
        // Darunter: Flachland; Vielleicht eine Waldregion?

        // Biom Wert
        let biom_value = self.biom_noise.get([world_x as f64, world_z as f64]);     

        // Basis-Hügel (große Formen)
        let base_height = self.tarrain_noise.get([
            world_x as f64 * 0.03, 
            world_z as f64 * 0.03
        ]) * 10.0;

        // Kleinere Noise für Rauheiten
        let detail = self.detail_noise.get([
            world_x as f64 * 0.12, 
            world_z as f64 * 0.12
        ]);

        let ridged = 1.0 - detail.abs();
        let ridge_contribution = ridged.powf(1.5) * 5.0;

        let combined=  if biom_value > SPIKY_MOUNTAINS {
            base_height + ridge_contribution
        } else {
            base_height + detail.abs() * 3.0
        };
        let spiky_mountains = combined.abs().powf(1.2) * combined.signum();
        let mountain_high = combined.abs().powf(1.5) * combined.signum();
        let mountain_mid = combined.abs().powf(1.3) * combined.signum();
        let hills = combined.abs().powf(1.1) * combined.signum();
        let flat = combined * 0.3;
        
        let height = if biom_value > SPIKY_MOUNTAINS {
            spiky_mountains
        } else if biom_value > MOUNTAIN_HIGH {
            let t = (biom_value - MOUNTAIN_HIGH) / (SPIKY_MOUNTAINS - MOUNTAIN_HIGH);
            self.lerp(mountain_high, spiky_mountains, t)
        } else if biom_value > MOUNTAIN_MID {
            // Übergang zwischen den Biomen via Linearer Interpolation
            let t = (biom_value - MOUNTAIN_MID) / (MOUNTAIN_HIGH - MOUNTAIN_MID);
            self.lerp(mountain_mid, mountain_high, t)
        } else if biom_value > HILLS {
            let t = (biom_value - HILLS) / (MOUNTAIN_MID - HILLS);
            self.lerp(hills, mountain_mid, t)
        } else {
            // Übergang zwischen Flachland und Hügeln
            let t = ((biom_value + 1.0) / 1.5).max(0.0); // Normalisiere auf 0-1
            self.lerp(flat, hills, t)
        };
        // Die Finale Höhe mit einer Basis von 30
        30 + height as i32 
    }

    pub fn get_block_at(&self, world_x: i32, world_y: i32, world_z: i32, ) -> BlockType {
        let height = self.get_height(world_x, world_z);
        // Über den Tarrain
        if world_y >= height {
            if world_y < 23 { // WATER_LEVEL
                return BlockType::Water;
            }
            return BlockType::Air;
        }
        // Höhlen: ob es das ist?
        // if world_y > 3 {
        //     let cave = self.cave_noise.get([
        //         world_x as f64 * 0.07,
        //         world_y as f64 * 0.07,
        //         world_z as f64 * 0.07,
        //     ]);
        //     if cave > 0.5 {
        //         return BlockType::Air
        //     }
        // }
        // Unter Terrain
        if world_y < height - 4 {
            BlockType::Stone
        } else if world_y <= height - 2 && world_y >= height - 4 {
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
                // Height könnte zu hoch sein
                if height <= 0 || height >= CHUNK_HEIGHT as i32 {
                    continue;
                }
                let ground_idx = Chunk::index(x, height as usize - 1, z);
                if blocks[ground_idx] != BlockType::Grass {
                    continue;
                }

                let tree_noise = self.tarrain_noise.get([world_x as f64 * 0.1, world_z as f64 * 0.1]);

                // Höhere Wahrscheinlichkeit in einem Wald bei biomnoise unter -0.3
                let biom_noise = self.biom_noise.get([world_x as f64, world_z as f64]);
                let noise_threshold = if biom_noise < -0.7 {
                    0.5
                } else {
                    0.7
                };
                let min_dist = if biom_noise < -0.7 {
                    4
                } else {
                    5
                };

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
    // Linear Interpolation: Findet einen Mittelwert zwischen Punkten, hier mit einem biom faktor.
    fn lerp(&self, a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }
}