use noise::{NoiseFn, Perlin, Seedable};

pub struct World {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
}

#[derive(Clone, Copy)]
pub enum Tile {
    Water,
    Land,
}

impl World {
    pub fn generate(width: u32, height: u32, seed: u32) -> Self {
        let perlin = Perlin::new(seed);
        let mut tiles = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 / width as f64;
                let ny = y as f64 / height as f64;

                let elevation = perlin.get([nx * 3.0, ny * 3.0]);

                if elevation < 0.0 {
                    tiles.push(Tile::Water);
                } else {
                    tiles.push(Tile::Land);
                }
            }
        }

        Self { width, height, tiles }
    }
}