pub struct ImageLoaderGatewayResult {
    pub data: Box<[u8]>,
    pub dimensions: (u32, u32),
}

pub trait ImageLoaderGateway {
    fn load_sync(&self, id: &str) -> anyhow::Result<ImageLoaderGatewayResult>;
}