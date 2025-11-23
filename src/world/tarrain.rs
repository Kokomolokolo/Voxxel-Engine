use noise::{NoiseFn, Perlin, Fbm, MultiFractal};
use super::Biome;

pub struct TarrainGenerator {
    tarrain_noise: Perlin,
    detail_noise: Perlin,
    biom_noise: Fbm<Perlin>,
}

impl TarrainGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            tarrain_noise: Perlin::new(seed),
            detail_noise: Perlin::new(seed + 100),
            biom_noise: Fbm::<Perlin>::new(seed + 200)
                .set_octaves(1)           // Je geringer dest glatter
                .set_frequency(0.0025),
        }
    }
    pub fn get_biom(&self, world_x: i32, world_z: i32) -> Biome {
        let value = self.biom_noise.get([world_x as f64 * 3.0, world_z as f64 * 3.0]);
        Biome::from_noise(value)
    }

    pub fn get_height(&self, world_x: i32, world_z: i32) -> i32 {
        // Biom
        let biome = self.get_biom(world_x, world_z);

        // Basis-Hügel (große Formen)
        let base_height = self.tarrain_noise.get([
            world_x as f64 * 0.01, 
            world_z as f64 * 0.01
        ]) * 10.0;

        // Kleinere Noise für Rauheiten
        let detail = self.detail_noise.get([
            world_x as f64 * 0.12, 
            world_z as f64 * 0.12
        ]);

        let combined = base_height + detail.abs() * 3.0;
        let modifier = biome.height_exponent();
        let height = combined.abs().powf(modifier as f64) * combined.signum();
        
        30 + height as i32
    }
}