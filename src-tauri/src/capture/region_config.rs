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


