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
                .set_octaves(2)           // Je geringer dest glatter
                .set_frequency(0.0025),
        }
    }
    pub fn get_biom(&self, world_x: i32, world_z: i32) -> Biome {
        let value = self.biom_noise.get([world_x as f64 * 3.0, world_z as f64 * 3.0]);
        Biome::from_noise(value)
    }
    fn get_biom_noise(&self, world_x: i32, world_z: i32) -> f64 {
        let value = self.biom_noise.get([world_x as f64 * 3.0, world_z as f64 * 3.0]);
        value
    }
    pub fn get_height(&self, world_x: i32, world_z: i32) -> i32 {
        // Biom
        let biome = self.get_biom(world_x, world_z);
        let biom_noise = self.get_biom_noise(world_x, world_z);

        // Basis-Hügel (große Formen)
        let base_height = self.tarrain_noise.get([
            world_x as f64 * 0.01, 
            world_z as f64 * 0.01
        ]) * 10.0;

        // Kleinere Noise für Rauheiten
        let detail = self.detail_noise.get([
            world_x as f64 * 0.07, 
            world_z as f64 * 0.07
        ]);

        let ridged = 1.0 - detail.abs();
        let ridge_contribution = ridged.powf(1.5) * 5.0;

        let combined=  if biom_noise > 0.8 {
            base_height + ridge_contribution
        } else {
            base_height + detail.abs() * 3.0
        };
        let spiky_mountains = combined.abs().powf(1.3) * combined.signum();
        let high_mountains = combined.abs().powf(1.4) * combined.signum();
        let mid_mountains = combined.abs().powf(1.3) * combined.signum();
        let hills = combined.abs().powf(1.1) * combined.signum();
        let plains = combined.abs().powf(0.9) * combined.signum();
        
        let height = if biom_noise > 0.8 {
            spiky_mountains
        } else if biom_noise > 0.6 {
            let t = (biom_noise - 0.6) / (0.8 - 0.6);
            self.lerp(high_mountains, spiky_mountains, t)
        } else if biom_noise > 0.4 {
            let t = (biom_noise - 0.4) / (0.6 - 0.4);
            self.lerp(mid_mountains, high_mountains, t)
        } else if biom_noise > 0.0 {
            let t = (biom_noise - 0.0) / (0.4 - 0.0);
            self.lerp(hills, mid_mountains, t)
        } else {
            let t = ((biom_noise + 1.0) / 1.0).clamp(0.0, 1.0);
            self.lerp(plains, hills, t)
        };
        // Die Finale Höhe mit einer Basis von 30
        30 + height as i32 
    }
    fn lerp(&self, a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }
}

