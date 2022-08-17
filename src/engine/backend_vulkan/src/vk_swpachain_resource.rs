
pub struct VkSwapchainResource<T> {
    resources: Vec<T>,
    image_count: u8
}

impl<T> VkSwapchainResource<T> {
    pub fn new(resources: Vec<T>, image_count: u8) -> Self {
        Self {
            resources,
            image_count
        }
    }
    
    pub fn get_image(&self, image: u8) -> &T {
        &self.resources[image as usize]
    }
}