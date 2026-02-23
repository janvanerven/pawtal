//! Image processing pipeline: resize variants and WebP conversion.
//!
//! Every uploaded image is processed into a fixed set of variants so the
//! front-end can always request the right size without client-side resizing.
//! Each variant is saved twice: once in the original format and once as WebP
//! for browsers that support it.
//!
//! The `webp` crate's `from_image` only accepts `ImageRgb8` and `ImageRgba8`,
//! so we unconditionally convert to RGBA before encoding. This is cheap
//! relative to the disk write and avoids "Unimplemented" errors from uncommon
//! colour-space images (e.g. greyscale PNGs).

use image::DynamicImage;
use std::path::Path;

use crate::error::{AppError, AppResult};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Describes a single output size produced during image processing.
pub struct ImageVariant {
    /// Used as the filename stem: `{suffix}.jpg`, `{suffix}.webp`, etc.
    pub suffix: String,
    /// Maximum width in pixels after resizing.
    pub max_width: u32,
    /// Maximum height in pixels after resizing.
    pub max_height: u32,
    /// When true the image is centre-cropped to a square before scaling.
    pub crop_square: bool,
}

// ─── Variant presets ──────────────────────────────────────────────────────────

/// Standard three-tier variant set used for article/page images.
pub fn get_standard_variants() -> Vec<ImageVariant> {
    vec![
        ImageVariant {
            suffix: "thumbnail".into(),
            max_width: 200,
            max_height: 200,
            crop_square: true,
        },
        ImageVariant {
            suffix: "medium".into(),
            max_width: 800,
            max_height: 800,
            crop_square: false,
        },
        ImageVariant {
            suffix: "large".into(),
            max_width: 1600,
            max_height: 1600,
            crop_square: false,
        },
    ]
}

/// Extended variant set for app icons — includes the standard three tiers plus
/// a dedicated square icon size.
pub fn get_icon_variants() -> Vec<ImageVariant> {
    let mut variants = get_standard_variants();
    variants.push(ImageVariant {
        suffix: "icon".into(),
        max_width: 128,
        max_height: 128,
        crop_square: true,
    });
    variants
}

// ─── Processing entry point ───────────────────────────────────────────────────

/// Reads the image at `input_path`, produces each variant in `variants`, and
/// writes them all into `output_dir`.
///
/// Each variant is written twice:
/// * `{suffix}.{original_ext}` — lossless / original format, for broad compat
/// * `{suffix}.webp`           — lossy WebP at quality 80, for modern browsers
///
/// Returns the `(width, height)` of the original image so the caller can
/// persist those dimensions in the database row.
pub fn process_image(
    input_path: &Path,
    output_dir: &Path,
    variants: &[ImageVariant],
) -> AppResult<(u32, u32)> {
    let img = image::open(input_path)
        .map_err(|e| AppError::Internal(format!("Failed to open image: {}", e)))?;

    let (orig_w, orig_h) = (img.width(), img.height());

    for variant in variants {
        let resized = resize_variant(&img, variant);
        save_variant(&resized, input_path, output_dir, &variant.suffix)?;
    }

    Ok((orig_w, orig_h))
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Produces a resized (and optionally cropped) copy of `img` according to the
/// variant spec. Does not mutate the original.
fn resize_variant(img: &DynamicImage, variant: &ImageVariant) -> DynamicImage {
    if variant.crop_square {
        let size = img.width().min(img.height());
        let cropped = img.crop_imm(
            (img.width() - size) / 2,
            (img.height() - size) / 2,
            size,
            size,
        );
        cropped.resize(
            variant.max_width,
            variant.max_height,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img.resize(
            variant.max_width,
            variant.max_height,
            image::imageops::FilterType::Lanczos3,
        )
    }
}

/// Writes one resized variant to disk in both the original format and WebP.
fn save_variant(
    resized: &DynamicImage,
    input_path: &Path,
    output_dir: &Path,
    suffix: &str,
) -> AppResult<()> {
    // Original format (keep extension from the uploaded file).
    let ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");
    let native_path = output_dir.join(format!("{}.{}", suffix, ext));
    resized
        .save(&native_path)
        .map_err(|e| AppError::Internal(format!("Failed to save variant '{}': {}", suffix, e)))?;

    // WebP — convert to RGBA first so the encoder always has a supported
    // pixel layout. The webp crate returns Err for greyscale variants.
    let rgba = resized.to_rgba8();
    let encoder =
        webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
    let webp_data = encoder.encode(80.0);
    let webp_path = output_dir.join(format!("{}.webp", suffix));
    std::fs::write(&webp_path, &*webp_data).map_err(|e| {
        AppError::Internal(format!("Failed to save WebP variant '{}': {}", suffix, e))
    })?;

    Ok(())
}
