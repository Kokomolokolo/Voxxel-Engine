use noise::{NoiseFn, Perlin, Fbm, MultiFractal};
use super::Biome;
pub struct TarrainGenerator {
    tarrain_noise: Perlin,
    detail_noise: Perlin,
    biom_noise: Fbm<Perlin>,
    cave_noise: Perlin,
}

impl TarrainGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            tarrain_noise: Perlin::new(seed),
            detail_noise: Perlin::new(seed + 100),
            biom_noise: Fbm::<Perlin>::new(seed + 200)
                .set_octaves(2)           // Je geringer dest glatter
                .set_frequency(0.0025),
            cave_noise: Perlin::new(seed + 420)
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


        let combined =  if biom_noise > Biome::SpikyMountains.noise_range().0 { // Die min Höhe des SpikyMountains Biomes
            let ridged = 1.0 - detail.abs();
            let ridge_contribution = ridged.powf(1.2) * 2.;
            base_height * ridge_contribution
        } else {
            base_height + detail.abs() * 3.0
        };
        
        
        let height = self.calculate_biom_height(biom_noise, combined);
        // Die Finale Höhe mit einer Basis von 30
        30 + height as i32 
    }

    fn calculate_biom_height(&self, biom_noise: f64, combined: f64 ) -> f64 {
        let current_biom = Biome::from_noise(biom_noise);
        let (min, max) = current_biom.noise_range();

        if let Some(next_biom) = current_biom.next_biom() {
            let t =((biom_noise - min) / (max - min)).clamp(0.0, 1.0);

            let curr_height = current_biom.apply_height_curve(combined);
            let next_height = next_biom.apply_height_curve(combined);

            // Interpolieren yay
            self.lerp(curr_height, next_height, t)
        } else {
            current_biom.apply_height_curve(combined)
        }
    }
    fn lerp(&self, a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }
    pub fn is_cave(&self, world_x: i32, world_y: i32, world_z: i32) -> bool {
        let cave_noise = self.cave_noise.get([
            world_x as f64 * 0.05,
            world_y as f64 * 0.05,
            world_z as f64 * 0.05,
        ]);
        
        cave_noise > 0.6
    }
}

