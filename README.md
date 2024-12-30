# imaginary-rs

A Rust implementation of the [h2non/imaginary](https://github.com/h2non/imaginary) image processing service.

## Features

- HTTP server for image processing
- Various image operations
- Security middleware
- Configuration options

## Supported Operations

- resize: Resize an image while maintaining aspect ratio
- crop: Smart cropping of images
- rotate: Rotate image by specified degrees
- thumbnail: Generate thumbnails efficiently
- watermark: Apply watermark to images
- optimize: Optimize image size and quality

## API Endpoints

### POST /resize
Resize an image with specified dimensions.

## Building