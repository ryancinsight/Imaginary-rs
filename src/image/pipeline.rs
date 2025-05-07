use image::DynamicImage;
use crate::image::params::Validate;
use crate::http::errors::ImageError;

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

    // Add a new method to validate parameters before adding an operation
    pub fn add_operation_with_validation<F, P>(&mut self, op: F, params: &P) -> Result<(), ImageError>
    where
        F: Fn(DynamicImage) -> DynamicImage + 'static,
        P: Validate,
    {
        params.validate()?;
        self.operations.push(Box::new(op));
        Ok(())
    }
}

pub fn process_image(params: impl Validate) -> Result<(), ImageError> {
    params.validate()?;
    let mut pipeline = ImagePipeline::new();

    // Example operation: convert image to grayscale
    pipeline.add_operation_with_validation(
        |img| img.grayscale(),
        &params,
    )?;

    // Load your image here (this is just a placeholder)
    let image = DynamicImage::new_rgb8(800, 600);

    // Process the image through the pipeline
    let _processed_image = pipeline.process(image);

    // Save or use the processed image (this is just a placeholder)
    // processed_image.save("output.png")?;

    Ok(())
}