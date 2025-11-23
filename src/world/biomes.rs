use crate::chunk::BlockType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Biome {
    SpikyMountains,
    HighMountains,
    MidMountains,
    Hills,
    Plains,
}

impl Biome {
    // Welches Biom bei welchem Noise?
    pub fn from_noise(value: f64) -> Self {
        if value > 0.8 { Self::SpikyMountains }
        else if value > 0.5 { Self::HighMountains }
        else if value > 0.3 { Self::MidMountains }
        else if value > -0.5 { Self::Hills }
        else { Self::Plains }
    }

    pub fn height_exponent(&self) -> f32 {
        match self {
            Self::SpikyMountains => 1.3,
            Self::HighMountains => 1.4,
            Self::MidMountains => 1.3,
            Self::Hills => 1.1,
            Self::Plains => 0.3,
        }
    }
    pub fn surface_block(&self, depth: i32) -> BlockType {
        match (self, depth) {
            // (Self::Desert, 0..=3) => BlockType::Sand,
            // (Self::Ocean, _) => BlockType::Sand,
            (_, 0) => BlockType::Grass,
            (_, 1..=3) => BlockType::Dirt,
            _ => BlockType::Stone,
        }
    }
    pub fn get_tree_density(&self) -> f64 {
        match self {
            Self::HighMountains | Self::SpikyMountains => 0.8,
            Self::MidMountains => 0.7,
            Self::Hills => 0.5,
            Self::Plains => 0.4,
        }
    }
}