use crate::db::{Database, RegionConfig, RegionRect};
use crate::error::LookoutError;

pub fn save_region_config(
    db: &Database,
    chart_area: RegionRect,
    ticker_area: RegionRect,
    price_area: RegionRect,
) -> Result<(), LookoutError> {
    let config = RegionConfig {
        chart_area,
        ticker_area,
        price_area,
    };
    db.save_region_config(&config)
}

pub fn get_region_config(db: &Database) -> Result<Option<RegionConfig>, LookoutError> {
    db.get_region_config()
}

pub fn compute_absolute_rect(
    region: &RegionRect,
    window_width: i32,
    window_height: i32,
) -> super::screenshot::Rect {
    super::screenshot::Rect {
        x: (region.x_pct * window_width as f64) as i32,
        y: (region.y_pct * window_height as f64) as i32,
        width: (region.width_pct * window_width as f64) as i32,
        height: (region.height_pct * window_height as f64) as i32,
    }
}
