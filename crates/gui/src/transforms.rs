use eframe::egui;
use rusty_runways_core::utils::{airport::Airport, coordinate::Coordinate};

/// Transform the coordinates of the map to the screen.
/// Computes offset for the screen and scales appropriately.
pub fn map_transforms(airports: &[(Airport, Coordinate)], target: egui::Rect, padding: f32) -> (f32, f32, f32) {

    // add padding
    let inner = target.shrink(padding);

    // world bounds
    let (min_x, max_x) = airports
        .iter()
        .map(|(_, c)| c.x)
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), x| {
            (min.min(x), max.max(x))
        });

    let (min_y, max_y) = airports
        .iter()
        .map(|(_, c)| c.y)
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), y| {
            (min.min(y), max.max(y))
        });

    // map to pixels
    let world_width = max_x - min_x;
    let width_pixels = inner.width();
    let world_height = max_y - min_y;
    let height_pixels = inner.height();

    // scale
    let scale_x = width_pixels / world_width;
    let scale_y = height_pixels / world_height;
    let scale = scale_x.min(scale_y);

    // EGUI (0,0) is top (n,n) is bottom left
    // X is fine, need to flip Y
    let offset_x = inner.left() - min_x * scale;
    let offset_y = inner.bottom() + min_y * scale;

    (scale, offset_x, offset_y)
}

/// maps world coordinates to position on the screen
pub fn world_to_screen(
    coord: &Coordinate,
    (scale, offset_x, offset_y): (f32, f32, f32),
) -> egui::Pos2 {
    egui::Pos2 {
        x: offset_x + coord.x * scale,
        y: offset_y - coord.y * scale,
    }
}
