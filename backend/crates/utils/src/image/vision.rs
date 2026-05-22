use image::{ColorType, GenericImageView, codecs::jpeg::JpegEncoder, imageops::FilterType};

use super::{
    data_url::{ImageDataUrlError, Result, image_data_url},
    mime::normalize_image_mime,
};

pub fn vision_image_data_url(mime: &str, bytes: &[u8]) -> Result<String> {
    vision_image_data_url_with_options(mime, bytes, 1024, 82)
}

pub fn vision_image_data_url_with_options(
    mime: &str,
    bytes: &[u8],
    max_edge: u32,
    jpeg_quality: u8,
) -> Result<String> {
    normalize_image_mime(mime)
        .ok_or_else(|| ImageDataUrlError::UnsupportedMime(mime.to_string()))?;
    let image =
        image::load_from_memory(bytes).map_err(|err| ImageDataUrlError::Decode(err.to_string()))?;
    let (width, height) = image.dimensions();
    let max_edge = max_edge.max(1);
    let resized = if width.max(height) > max_edge {
        image.resize(max_edge, max_edge, FilterType::Triangle)
    } else {
        image
    };
    let rgb = resized.to_rgb8();
    let mut output = Vec::new();
    JpegEncoder::new_with_quality(&mut output, jpeg_quality)
        .encode(&rgb, rgb.width(), rgb.height(), ColorType::Rgb8.into())
        .map_err(|err| ImageDataUrlError::Encode(err.to_string()))?;
    image_data_url("image/jpeg", &output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_resized_vision_data_url() {
        let image = image::RgbImage::from_pixel(2, 2, image::Rgb([255, 0, 0]));
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, 90)
            .encode(
                &image,
                image.width(),
                image.height(),
                ColorType::Rgb8.into(),
            )
            .unwrap();

        let data_url = vision_image_data_url_with_options("image/jpeg", &jpeg, 1, 80).unwrap();
        assert!(data_url.starts_with("data:image/jpeg;base64,"));
    }
}
