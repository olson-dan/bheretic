use bevy::prelude::*;

mod engine;
mod menu;

use engine::Engine;
use menu::Menus;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Engine {
            wadfile: "heretic.wad",
        })
        .add_plugin(Menus)
        .run();
}
