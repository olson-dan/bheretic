use crate::engine::{Patch, Vid, Wad, TIME_STEP};
use bevy::core::FixedTimestep;
use bevy::prelude::*;

#[derive(PartialEq, Copy, Clone)]
enum Menu {
    None,
    Main,
    Episode,
    Skill,
    Options,
    Options2,
    Files,
    Load,
    Save,
}

#[derive(Component)]
struct MenuIdent(Menu);

#[derive(Component)]
struct MenuOffset(usize, usize);

#[derive(Clone)]
enum MenuAction {
    None,
    SetMenu(Menu),
    NetCheck(u32, Menu),
    Info,
    QuitGame,
    Episode(u32),
    LoadGame(u32),
    SaveGame(u32),
    Skill(u32),
    EndGame,
    Messages,
    MouseSensitivity,
    ScreenSize,
    SfxVolume,
    MusicVolume,
}

#[derive(Component)]
struct MenuState {
    cur: Menu,
    stack: Vec<Menu>,
    background: Option<&'static str>,
    time: u32,
    selection: usize,
}

#[derive(Component)]
struct MenuItem {
    text: &'static str,
    action: MenuAction,
}

fn render_specific_menus(wad: &Wad, vid: &mut Vid, menu: &Menu, time: u32) {
    match *menu {
        Menu::Main => {
            let base = wad.get_num_for_name("M_SKL00").expect("missing M_SKL00");
            let frame = ((time / 3) % 18) as usize;
            vid.draw_patch(wad, 88, 0, "M_HTIC");
            vid.draw_patch_raw(wad.cache_lump_num(base + (17 - frame)).unwrap(), 40, 10);
            vid.draw_patch_raw(wad.cache_lump_num(base + frame).unwrap(), 232, 10);
        }
        _ => {}
    }
}

fn render_text(wad: &Wad, vid: &mut Vid, font: &str, text: &str, x: usize, y: usize) {
    let mut x = x;
    let base = wad.get_num_for_name(font).expect("Missing font") + 1;
    for c in text.chars() {
        if let Ok(c) = u32::try_from(c) {
            if c < 33 {
                x += 5;
            } else {
                let lump = wad.cache_lump_num(base + (c as usize) - 33).unwrap();
                vid.draw_patch_raw(lump, x, y);
                let patch = Patch::from_lump(lump);
                x += patch.w - 1;
            }
        }
    }
}

fn text_width(wad: &Wad, font: &str, text: &str) -> usize {
    let mut w = 0;
    let base = wad.get_num_for_name(font).expect("Missing font") + 1;
    for c in text.chars() {
        if let Ok(c) = u32::try_from(c) {
            if c < 33 {
                w += 5;
            } else {
                let lump = wad.cache_lump_num(base + (c as usize) - 33).unwrap();
                let patch = Patch::from_lump(lump);
                w += patch.w - 1;
            }
        }
    }
    w
}

fn setup(mut commands: Commands) {
    let main_items = [
        ("NEW GAME", MenuAction::NetCheck(1, Menu::Episode)),
        ("OPTIONS", MenuAction::SetMenu(Menu::Options)),
        ("GAME FILES", MenuAction::SetMenu(Menu::Files)),
        ("INFO", MenuAction::Info),
        ("QUIT GAME", MenuAction::QuitGame),
    ];

    let episode_items = [
        ("CITY OF THE DAMNED", MenuAction::Episode(1)),
        ("HELL's MAW", MenuAction::Episode(2)),
        ("THE DOME OF D'SPARIL", MenuAction::Episode(3)),
        ("THE OSSUARY", MenuAction::Episode(4)),
        ("THE STAGNANT DEMESNE", MenuAction::Episode(5)),
    ];

    let files_items = [
        ("LOAD GAME", MenuAction::NetCheck(2, Menu::Load)),
        ("SAVE GAME", MenuAction::SetMenu(Menu::Save)),
    ];

    let skill_items = [
        ("THOU NEEDETH A WET-NURSE", MenuAction::Skill(0)),
        ("YELLOWBELLIES-R-US", MenuAction::Skill(1)),
        ("BRINGEST THEM ONETH", MenuAction::Skill(2)),
        ("THOU ART A SMITE-MEISTER", MenuAction::Skill(3)),
        ("BLACK PLAGUE POSSESSES THEE", MenuAction::Skill(4)),
    ];

    let options_items = [
        ("END GAME", MenuAction::EndGame),
        ("MESSAGES : ", MenuAction::Messages),
        ("MOUSE SENSITIVITY", MenuAction::MouseSensitivity),
        ("", MenuAction::None),
        ("MORE...", MenuAction::SetMenu(Menu::Options2)),
    ];

    let options2_items = [
        ("SCREEN SIZE", MenuAction::ScreenSize),
        ("", MenuAction::None),
        ("SFX VOLUME", MenuAction::SfxVolume),
        ("", MenuAction::None),
        ("MUSIC VOLUME", MenuAction::MusicVolume),
    ];

    // Main
    commands
        .spawn()
        .insert(MenuIdent(Menu::Main))
        .insert(MenuOffset(110, 56));
    for (text, action) in &main_items {
        commands
            .spawn()
            .insert(MenuIdent(Menu::Main))
            .insert(MenuItem {
                text: text,
                action: action.clone(),
            });
    }

    // Episode
    commands
        .spawn()
        .insert(MenuIdent(Menu::Episode))
        .insert(MenuOffset(80, 50));
    for (text, action) in &episode_items {
        commands
            .spawn()
            .insert(MenuIdent(Menu::Episode))
            .insert(MenuItem {
                text: text,
                action: action.clone(),
            });
    }

    // Files
    commands
        .spawn()
        .insert(MenuIdent(Menu::Files))
        .insert(MenuOffset(110, 60));
    for (text, action) in &files_items {
        commands
            .spawn()
            .insert(MenuIdent(Menu::Files))
            .insert(MenuItem {
                text: text,
                action: action.clone(),
            });
    }

    // Skill
    commands
        .spawn()
        .insert(MenuIdent(Menu::Skill))
        .insert(MenuOffset(38, 30));
    for (text, action) in &skill_items {
        commands
            .spawn()
            .insert(MenuIdent(Menu::Skill))
            .insert(MenuItem {
                text: text,
                action: action.clone(),
            });
    }

    // Options
    commands
        .spawn()
        .insert(MenuIdent(Menu::Options))
        .insert(MenuOffset(88, 30));
    for (text, action) in &options_items {
        commands
            .spawn()
            .insert(MenuIdent(Menu::Options))
            .insert(MenuItem {
                text: text,
                action: action.clone(),
            });
    }

    // Options2
    commands
        .spawn()
        .insert(MenuIdent(Menu::Options2))
        .insert(MenuOffset(88, 30));
    for (text, action) in &options2_items {
        commands
            .spawn()
            .insert(MenuIdent(Menu::Options2))
            .insert(MenuItem {
                text: text,
                action: action.clone(),
            });
    }
}

fn update(
    mut state: ResMut<MenuState>,
    items: Query<(&MenuIdent, &MenuItem)>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut app_exit_events: EventWriter<bevy::app::AppExit>,
) {
    if state.cur == Menu::None {
        if keyboard_input.clear_just_pressed(KeyCode::Escape) {
            state.cur = Menu::Main;
        }
        return;
    }

    state.time += 1;

    if keyboard_input.clear_just_pressed(KeyCode::Escape) {
        if let Some(m) = state.stack.pop() {
            state.cur = m;
        } else {
            state.cur = Menu::None;
        }
        return;
    }

    if keyboard_input.clear_just_pressed(KeyCode::Return) {
        let mut item_count = 0;
        for (ident, items) in items.iter() {
            if ident.0 == state.cur {
                if state.selection == item_count {
                    match items.action {
                        MenuAction::None => break,
                        MenuAction::SetMenu(m) => {
                            state.selection = 0;
                            let last = state.cur;
                            state.stack.push(last);
                            state.cur = m;
                            return;
                        }
                        MenuAction::QuitGame => {
                            app_exit_events.send(bevy::app::AppExit);
                            return;
                        }
                        _ => {}
                    }
                    //
                }
                item_count += 1;
            }
        }
    }

    let labels = items
        .iter()
        .filter(|(ident, _)| ident.0 == state.cur)
        .map(|(_, item)| item.text)
        .collect::<Vec<_>>();

    if keyboard_input.clear_just_pressed(KeyCode::Up) {
        loop {
            if state.selection == 0 {
                state.selection = labels.len() - 1;
            } else {
                state.selection -= 1;
            }
            if !labels[state.selection].is_empty() {
                break;
            }
        }
    }

    if keyboard_input.clear_just_pressed(KeyCode::Down) {
        loop {
            if state.selection + 1 == labels.len() {
                state.selection = 0;
            } else {
                state.selection += 1;
            }
            if !labels[state.selection].is_empty() {
                break;
            }
        }
    }
}

fn render(
    wad: Res<Wad>,
    mut vid: ResMut<Vid>,
    state: Res<MenuState>,
    menus: Query<(&MenuIdent, &MenuOffset)>,
    items: Query<(&MenuIdent, &MenuItem)>,
) {
    vid.set_palette(&wad, "PLAYPAL");
    if let Some(patch) = state.background {
        vid.draw_raw_screen(&wad, patch);
    }

    if state.cur == Menu::None {
        return;
    }

    let mut x = 0;
    let mut orig_y = 0;

    for (ident, offset) in menus.iter() {
        if state.cur == ident.0 {
            x = offset.0;
            orig_y = offset.1;
            break;
        }
    }

    render_specific_menus(&wad, &mut vid, &state.cur, state.time);

    let mut y = orig_y;
    let mut item_count = 0;
    for (ident, item) in items.iter() {
        if ident.0 == state.cur {
            item_count += 1;
            if !item.text.is_empty() {
                render_text(&wad, &mut vid, "FONTB_S", item.text, x, y);
            }
            y += 20;
        }
    }

    let item_num = std::cmp::min(item_count, state.selection);
    let y = orig_y + item_num * 20 - 1;
    if (state.time & 16) != 0 {
        vid.draw_patch(&wad, x - 28, y, "M_SLCTR1");
    } else {
        vid.draw_patch(&wad, x - 28, y, "M_SLCTR2");
    }
}

pub struct Menus;

impl Plugin for Menus {
    fn build(&self, app: &mut App) {
        app.insert_resource(MenuState {
            cur: Menu::None,
            stack: Vec::new(),
            background: Some("TITLE"),
            selection: 0,
            time: 0,
        })
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .label("game")
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update)
                .with_system(render),
        );
    }
}
