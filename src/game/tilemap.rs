use crate::renderer::instance::TileInstance;

#[derive(Clone, Copy)]
pub struct Tile {
    pub color: [f32; 3],
}

pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
}

impl TileMap {
    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = Vec::new();
        let w = width as usize;
        let h = height as usize;

        tiles.resize(
            w * h,
            Tile {
                color: [0.3, 0.6, 0.2],
            },
        );

        Self {
            width,
            height,
            tiles,
        }
    }
    
    pub fn to_instances(&self) -> Vec<TileInstance> {
        let mut instances = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let tile = self.tiles[(y * self.width + x) as usize];

                instances.push(TileInstance {
                    position: [x as f32, 0.0, y as f32],
                    color: tile.color,
                });
            }
        }

        instances
    }
}
