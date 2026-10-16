//! The tray icon bitmap.
//!
//! To keep the crate free of an image-decoding dependency (and of binary assets
//! that differ per platform) the default icon is rendered procedurally into raw
//! RGBA. Replace [`default_icon`] with `Icon::from_path`/`Icon::from_resource`
//! if you ship real `.ico`/`.png` artwork.

use tray_icon::Icon;

use crate::error::{AppError, AppResult};

/// Edge length, in pixels, of the generated icon.
pub const DEFAULT_ICON_SIZE: u32 = 32;

/// Render the default icon into a ready-to-use [`Icon`].
///
/// # Errors
/// Returns [`AppError::Tray`] if the generated buffer is rejected by the backend.
pub fn default_icon(size: u32) -> AppResult<Icon> {
    let rgba = render_badge(size);
    Icon::from_rgba(rgba, size, size)
        .map_err(|err| AppError::Tray(format!("invalid tray icon bitmap: {err}")))
}

/// Draw a filled accent disc with a white centre dot, anti-aliased at the edges.
fn render_badge(size: u32) -> Vec<u8> {
    let edge = size.max(1);
    let center = (edge as f32 - 1.0) / 2.0;
    let outer_radius = center + 0.5;
    let inner_radius = outer_radius * 0.45;

    let accent = [0x2F_u8, 0x81, 0xF7];
    let highlight = [0xFF_u8, 0xFF, 0xFF];

    let mut rgba = vec![0_u8; (edge * edge * 4) as usize];

    for y in 0..edge {
        for x in 0..edge {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();

            // 1.0 well inside the disc, 0.0 outside, smooth across the border.
            let coverage = smoothstep(outer_radius, outer_radius - 1.0, distance);
            if coverage <= 0.0 {
                continue;
            }
            let centre_mix = smoothstep(inner_radius + 0.75, inner_radius - 0.75, distance);

            let idx = ((y * edge + x) * 4) as usize;
            for channel in 0..3 {
                let base = f32::from(accent[channel]);
                let top = f32::from(highlight[channel]);
                rgba[idx + channel] = (base + (top - base) * centre_mix).round() as u8;
            }
            rgba[idx + 3] = (coverage * 255.0).round() as u8;
        }
    }

    rgba
}

/// Hermite smoothstep; `edge0` and `edge1` may be given in either order.
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if (edge1 - edge0).abs() < f32::EPSILON {
        return if x < edge0 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_has_expected_dimensions_and_alpha() {
        let rgba = render_badge(DEFAULT_ICON_SIZE);
        assert_eq!(
            rgba.len(),
            (DEFAULT_ICON_SIZE * DEFAULT_ICON_SIZE * 4) as usize
        );

        // The centre is fully opaque, the corner fully transparent.
        let centre =
            ((DEFAULT_ICON_SIZE / 2) * DEFAULT_ICON_SIZE + DEFAULT_ICON_SIZE / 2) as usize * 4;
        assert_eq!(rgba[centre + 3], 255);
        assert_eq!(rgba[3], 0);
    }

    #[test]
    fn smoothstep_is_monotonic_and_clamped() {
        assert_eq!(smoothstep(0.0, 10.0, -5.0), 0.0);
        assert_eq!(smoothstep(0.0, 10.0, 15.0), 1.0);
        assert!(smoothstep(0.0, 10.0, 3.0) < smoothstep(0.0, 10.0, 7.0));
    }
}
