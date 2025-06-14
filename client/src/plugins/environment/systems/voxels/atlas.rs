use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::GenericImageView;

/// Configuration and handle for the voxel texture atlas.
#[derive(Resource, Clone)]
pub struct VoxelTextureAtlas {
    pub handle: Handle<Image>,
    pub columns: usize,
    pub rows: usize,
}

impl VoxelTextureAtlas {
    /// Generate an atlas from PNG files located in `assets/textures/packs/mc/grass`.
    pub fn generate(images: &mut Assets<Image>) -> Self {
        // Include the PNG files at compile time so we don't rely on runtime IO.
        const TOP: &[u8] = include_bytes!("../../../../../assets/textures/packs/mc/grass/grass_block_top.png");
        const BOTTOM: &[u8] = include_bytes!("../../../../../assets/textures/packs/mc/grass/dirt.png");
        const SIDE: &[u8] = include_bytes!("../../../../../assets/textures/packs/mc/grass/grass_block_side.png");

        let textures = [TOP, BOTTOM, SIDE];
        // Assume all textures have the same dimensions
        let first = image::load_from_memory(TOP).expect("failed to load texture");
        let tile_size = first.width();

        let columns = textures.len();
        let rows = 1usize;
        let width = tile_size * columns as u32;
        let height = tile_size;
        let mut data = vec![0u8; (width * height * 4) as usize];

        for (i, tex_bytes) in textures.iter().enumerate() {
            let img = image::load_from_memory(tex_bytes)
                .expect("failed to load texture")
                .to_rgba8();
            for y in 0..tile_size {
                for x in 0..tile_size {
                    let idx = (((y) * width + x + (i as u32) * tile_size) * 4) as usize;
                    let pixel = img.get_pixel(x, y).0;
                    data[idx..idx + 4].copy_from_slice(&pixel);
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