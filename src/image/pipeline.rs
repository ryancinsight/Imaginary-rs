use crate::image::operations;
use image::DynamicImage;

pub struct ImagePipeline {
    operations: Vec<Box<dyn Fn(DynamicImage) -> DynamicImage>>,
}

impl ImagePipeline {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn add_operation<F>(&mut self, op: F)
    where
        F: Fn(DynamicImage) -> DynamicImage + 'static,
    {
        self.operations.push(Box::new(op));
    }

    pub fn process(&self, image: DynamicImage) -> DynamicImage {
        self.operations.iter().fold(image, |img, op| op(img))
    }
}