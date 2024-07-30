use image::GenericImageView;
use common::gateways::{ImageLoaderGateway, ImageLoaderGatewayResult};

pub struct ImageCrateImageLoaderAdapter;

impl ImageCrateImageLoaderAdapter {
    pub fn new() -> ImageCrateImageLoaderAdapter {
        ImageCrateImageLoaderAdapter
    }
}

impl ImageLoaderGateway for ImageCrateImageLoaderAdapter {
    fn load_sync(&self, _: &str) -> anyhow::Result<ImageLoaderGatewayResult> {
        // FIXME - hardcoded image
        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes)?;
        
        Ok(ImageLoaderGatewayResult { 
            data: diffuse_image.to_rgba8().into_raw().into_boxed_slice(),
            dimensions: diffuse_image.dimensions(),
        })
    }
}