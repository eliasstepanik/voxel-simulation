use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};

/// Configuration and handle for the voxel texture atlas.
#[derive(Resource, Clone)]
pub struct VoxelTextureAtlas {
    pub handle: Handle<Image>,
    pub columns: usize,
    pub rows: usize,
}

impl VoxelTextureAtlas {
    /// Create a simple procedural atlas with solid colors.
    pub fn generate(images: &mut Assets<Image>) -> Self {
        let tile_size = 16u32;
        let columns = 2;
        let rows = 3;
        let width = tile_size * columns as u32;
        let height = tile_size * rows as u32;
        let mut data = vec![0u8; (width * height * 4) as usize];
        let colors = [
            [255, 0, 0, 255],   // red
            [0, 255, 0, 255],   // green
            [0, 0, 255, 255],   // blue
            [255, 255, 0, 255], // yellow
            [255, 0, 255, 255], // magenta
            [0, 255, 255, 255], // cyan
        ];
        for (i, col) in colors.iter().enumerate() {
            let cx = (i % columns) as u32 * tile_size;
            let cy = (i / columns) as u32 * tile_size;
            for y in 0..tile_size {
                for x in 0..tile_size {
                    let idx = (((cy + y) * width + (cx + x)) * 4) as usize;
                    data[idx..idx + 4].copy_from_slice(col);
                }
            }
        }
        let image = Image::new_fill(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let handle = images.add(image);
        Self {
            handle,
            columns,
            rows,
        }
    }

    /// Compute UV coordinates for the given atlas index.
    pub fn uv_rect(&self, index: usize) -> [[f32; 2]; 4] {
        let col = index % self.columns;
        let row = index / self.columns;
        let cols = self.columns as f32;
        let rows = self.rows as f32;
        let u0 = col as f32 / cols;
        let v0 = row as f32 / rows;
        let u1 = (col + 1) as f32 / cols;
        let v1 = (row + 1) as f32 / rows;
        [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
    }
}
