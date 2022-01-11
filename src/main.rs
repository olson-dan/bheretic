use bevy::prelude::*;

mod engine;
mod menu;

use engine::{Engine, SCREEN_HEIGHT, SCREEN_WIDTH};
use menu::Menus;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Heretic".to_string(),
            width: SCREEN_WIDTH as f32 * 2.0,
            height: SCREEN_HEIGHT as f32 * 2.0,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Engine {
            wadfile: "heretic.wad",
        })
        .add_plugin(Menus)
        .run();
}
