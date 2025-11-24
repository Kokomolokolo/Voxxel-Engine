mod biomes;
mod tarrain;
mod features;

use biomes::*;
use tarrain::*;
use features::*;

use noise::{NoiseFn, Perlin, Fbm};
use bevy::prelude::IVec2;

use crate::chunk::{BlockType, CHUNK_HEIGHT, CHUNK_WIDTH};

pub struct WorldGenerator {
    tarrain: TarrainGenerator,
    features: FeatureGenerator,
}

impl WorldGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            tarrain: TarrainGenerator::new(seed),
            features: FeatureGenerator::new(seed)
        }
    }
    pub fn get_height(&self, world_x: i32, world_z: i32) -> i32 {
        self.tarrain.get_height(world_x, world_z)
    }
    pub fn get_biom(&self, world_x: i32, world_z: i32) -> Biome {
        self.tarrain.get_biom(world_x, world_z)
    }
    pub fn get_block_at(&self, world_x: i32, world_y: i32, world_z: i32) -> BlockType {
        let height = self.get_height(world_x, world_z);
        let biom = self.get_biom(world_x, world_z);

        let depth = height - world_y;
        let biom_surface =  biom.get_surface_block(depth); 
        if world_y >= height {
            if world_y < 24 { // Wasser Level
                return BlockType::Water;
            }
            return BlockType::Air;
        }
        if world_y < height - 4 {
            return BlockType::Stone;
        } else if world_y <= height - 2 && world_y >= height - 4 {
            return BlockType::Dirt;
        } else {
            if height <= 24 && height >= 20 {
                return BlockType::Sand;
            }
            else {
                return biom_surface;
            }
        }        
    }
    pub fn generate_trees(&self, blocks: &mut [BlockType; CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT], chunk_pos: IVec2) {
        self.features.generate_trees(blocks, &self.tarrain, chunk_pos);
    }
}