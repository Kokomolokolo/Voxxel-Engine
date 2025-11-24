use crate::chunk::BlockType;

pub enum Biome {
    SpikyMountains,
    HighMountains,
    MidMountains,
    Hills,
    Plains,
}

impl Biome {
    pub fn from_noise(noise: f64) -> Biome {
        if noise > 0.8 { Self::SpikyMountains }
        else if noise > 0.6 { Self::HighMountains }
        else if noise > 0.4 { Self::MidMountains }
        else if noise > 0.0 { Self::Hills }
        else { Self::Plains }
    }
    pub fn height_exponent(&self) -> f32 {
        match self {
            Self::SpikyMountains => 1.3,
            Self::HighMountains => 1.4,
            Self::MidMountains => 1.3,
            Self::Hills => 1.1,
            Self::Plains => 0.9,
        }
    }
    pub fn get_surface_block(&self, depth: i32) -> BlockType {
        match (depth, self) {
            (0..3, Biome::Plains | Biome::Hills | Biome::MidMountains | Biome::HighMountains | Biome::SpikyMountains ) => BlockType::Grass,
            (_, _) => BlockType::Sand
        }
    }
}