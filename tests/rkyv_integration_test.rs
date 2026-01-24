use imaginary::image::params::{ResizeParams, ResizeFilter};
use imaginary::http::handlers::info_handler::ImageInfo;

#[test]
fn test_resize_params_serialization() {
    let params = ResizeParams {
        width: 100,
        height: 200,
        filter: ResizeFilter::Triangle,
    };

    // Serialize
    let bytes = rkyv::to_bytes::<_, 256>(&params).expect("failed to serialize params");

    // Deserialize
    let deserialized: ResizeParams = rkyv::from_bytes(&bytes).expect("failed to deserialize params");

    assert_eq!(params.width, deserialized.width);
    assert_eq!(params.height, deserialized.height);

    match deserialized.filter {
        ResizeFilter::Triangle => {},
        _ => panic!("Wrong filter deserialized"),
    }

    // Access archived value directly (zero-copy)
    let archived = rkyv::check_archived_root::<ResizeParams>(&bytes).expect("failed to check bytes");
    assert_eq!(archived.width, params.width);
    assert_eq!(archived.height, params.height);
}

#[test]
fn test_image_info_serialization() {
    let info = ImageInfo {
        width: 800,
        height: 600,
        format: "png".to_string(),
        space: "srgb".to_string(),
        channels: 3,
        depth: 8,
        has_alpha: false,
    };

    // Serialize
    let bytes = rkyv::to_bytes::<_, 256>(&info).expect("failed to serialize info");

    // Deserialize
    let deserialized: ImageInfo = rkyv::from_bytes(&bytes).expect("failed to deserialize info");

    assert_eq!(info.width, deserialized.width);
    assert_eq!(info.height, deserialized.height);
    assert_eq!(info.format, deserialized.format);
    assert_eq!(info.space, deserialized.space);
    assert_eq!(info.channels, deserialized.channels);
    assert_eq!(info.depth, deserialized.depth);
    assert_eq!(info.has_alpha, deserialized.has_alpha);

    // Access archived value directly
    let archived = rkyv::check_archived_root::<ImageInfo>(&bytes).expect("failed to check bytes");
    assert_eq!(archived.width, info.width);
    assert_eq!(archived.format, info.format);
}
