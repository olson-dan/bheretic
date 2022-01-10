use bevy::core::FixedTimestep;
use bevy::prelude::*;

mod engine;
mod menu;

use engine::{render, DoomEngine, TIME_STEP};
use menu::Menus;

fn main() {
    let engine = DoomEngine {
        wadfile: "heretic.wad",
    };
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(engine)
        .add_plugin(Menus)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(render),
        )
        .run();
}
