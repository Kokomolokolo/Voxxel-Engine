use crate::chunk::BlockType;

#[derive(Clone, Copy, Debug)]
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
    /// Gibt die Noise Änderungswerte zurück für Interpolation zurück (min, max)
    pub fn noise_range(&self) -> (f64, f64) {
        match self {
            Self::Plains => (-1.0, 0.0),
            Self::Hills => (0.0, 0.4),
            Self::MidMountains => (0.4, 0.6),
            Self::HighMountains => (0.6, 0.8),
            Self::SpikyMountains => (0.8, 1.0),
        }
    }
    pub fn height_exponent(&self) -> f64 {
        match self {
            Self::SpikyMountains => 1.3,
            Self::HighMountains => 1.6,
            Self::MidMountains => 1.4,
            Self::Hills => 1.1,
            Self::Plains => 0.9,
        }
    }

    // Die höhe des Biomes auf der Kurve berechnen
    pub fn apply_height_curve(&self, combined: f64) -> f64 {
        combined.abs().powf(self.height_exponent()) * combined.signum()
    }

    pub fn next_biom(&self) -> Option<Biome> {
        match self {
            Self::Plains => Some(Self::Hills),
            Self::Hills => Some(Self::MidMountains),
            Self::MidMountains => Some(Self::HighMountains),
            Self::HighMountains => Some(Self::SpikyMountains),
            Self::SpikyMountains => None,
        }
    }


    pub fn get_surface_block(&self, depth: i32) -> BlockType {
        match (depth, self) {
            (0..3, Biome::Plains | Biome::Hills | Biome::MidMountains | Biome::HighMountains | Biome::SpikyMountains ) => BlockType::Grass,
            (_, _) => BlockType::Sand
        }
    }
}