//! Logic to handle encoding and decoding of attachments.

use image::{DynamicImage, GenericImageView, codecs::jpeg::JpegEncoder};
use std::io::{Read, BufReader};

use crate::error::AttachmentError;

/// Max file size is 500 KiB.
const MAX_FILE_SIZE: usize = 500 * 1024;

/// Reencode outgoing image to bytes.
/// This strips metadata.
pub fn reencode_image_to_bytes<P: AsRef<std::path::Path>>(
    input: P,
) -> Result<Vec<u8>, AttachmentError> {
    let path = input.as_ref();
    // Open once, and load image from memory to avoid TOCTOU.
    let file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read file data into bytes.
    let mut bytes = Vec::with_capacity(MAX_FILE_SIZE.min(64 * 1024));
    reader
        .by_ref()
        .take((MAX_FILE_SIZE + 1) as u64) // MAX_FILE_SIZE + 1 to detect exceeding file size limit.
        .read_to_end(&mut bytes)?;

    // Check max file size.
    if bytes.len() > MAX_FILE_SIZE {
        return Err(AttachmentError::FileSizeExceedsLimit);
    }

    // Decode.
    let image: DynamicImage = image::load_from_memory(&bytes)?;

    reencode_image(image)
}

/// Reencode bytes of incoming message.
pub fn reencode_bytes(
    input: Vec<u8>,
) -> Result<Vec<u8>, AttachmentError> {
    // Check file size.
    if input.len() > MAX_FILE_SIZE {
        return Err(AttachmentError::FileSizeExceedsLimit);
    }
    
    // Incoming bytes should always be JPEG.
    if input.len() < 2 || input[0] != 0xFF || input[1] != 0xD8 {
        return Err(AttachmentError::FileUnsupportedFormat);
    }

    // Decode.
    let image: DynamicImage = image::load_from_memory(&input)?;

    reencode_image(image)
}

/// Shared reencode implementation.
fn reencode_image(
    image: DynamicImage,
) -> Result<Vec<u8>, AttachmentError> {
    // Check max width and height.
    let (x, y) = image.dimensions();
    if x > 1025 || y > 1025 {
        return Err(AttachmentError::ImageDimensionsExceedsLimit);
    }

    // Copy to clean buffer.
    let rgba = image.to_rgba8();
    let buffer = DynamicImage::ImageRgba8(rgba);

    // Convert to JPEG and return bytes.
    let mut output = Vec::new();
    // Low quality to reduce chance to image fingerprinting.
    let mut encoder = JpegEncoder::new_with_quality(&mut output, 50);
    encoder.encode_image(&buffer)?;

    Ok(output)
}

