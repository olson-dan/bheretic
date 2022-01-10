use anyhow::{bail, Result};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::SeekFrom;

pub const SCREEN_WIDTH: usize = 320;
pub const SCREEN_HEIGHT: usize = 200;
pub const TIME_STEP: f32 = 1.0 / 35.0;

#[derive(Component)]
pub struct FrameBuffer;

pub struct Lump {
    pub name: String,
    pub data: Vec<u8>,
    pub pos: u64,
    pub len: usize,
}

pub struct Wad {
    pub lumps: Vec<Lump>,
}

impl Wad {
    pub fn from_reader<T>(mut data: T) -> Result<Wad>
    where
        T: Read + Seek,
    {
        let mut tag = [0u8; 4];
        data.read_exact(&mut tag)?;
        let tag = std::str::from_utf8(&tag)?;
        if tag != "IWAD" && tag != "PWAD" {
            bail!(format!("Invalid WAD magic"));
        }
        let num_lumps = data.read_u32::<LittleEndian>()? as usize;
        let offset = data.read_u32::<LittleEndian>()? as u64;

        data.seek(SeekFrom::Start(offset))?;

        let mut lumps: Vec<Lump> = Vec::with_capacity(num_lumps);
        for _ in 0..num_lumps {
            let pos = data.read_u32::<LittleEndian>()? as u64;
            let len = data.read_u32::<LittleEndian>()? as usize;
            let mut name = [0u8; 8];
            data.read_exact(&mut name)?;
            let name = std::str::from_utf8(&name)?
                .trim_end_matches('\0')
                .to_string();
            lumps.push(Lump {
                name,
                pos,
                len,
                data: Vec::new(),
            });
        }
        for lump in lumps.iter_mut() {
            data.seek(SeekFrom::Start(lump.pos))?;
            let mut contents = vec![0u8; lump.len];
            data.read_exact(&mut contents)?;
            lump.data = contents;
        }

        Ok(Wad { lumps })
    }

    pub fn get_num_for_name(&self, name: &str) -> Option<usize> {
        for (i, lump) in self.lumps.iter().enumerate() {
            if lump.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn cache_lump_num(&self, num: usize) -> Option<&[u8]> {
        if num < self.lumps.len() {
            return Some(&self.lumps[num].data);
        }
        None
    }

    pub fn cache_lump_name(&self, name: &str) -> Option<&[u8]> {
        if let Some(lump) = self.lumps.iter().rev().find(|l| l.name == name) {
            Some(&lump.data)
        } else {
            None
        }
    }
}

pub struct Patch {
    pub w: usize,
    pub h: usize,
    pub left: isize,
    pub top: isize,
}

impl Patch {
    pub fn from_lump(mut data: &[u8]) -> Patch {
        let w = data.read_u16::<LittleEndian>().unwrap() as usize;
        let h = data.read_u16::<LittleEndian>().unwrap() as usize;
        let left = data.read_i16::<LittleEndian>().unwrap() as isize;
        let top = data.read_i16::<LittleEndian>().unwrap() as isize;
        Patch { w, h, left, top }
    }
}

pub struct Vid {
    fb: Vec<u8>,
    palette: Option<Vec<[u8; 4]>>,
}

impl Vid {
    fn new() -> Vid {
        Vid {
            fb: vec![0; SCREEN_WIDTH * SCREEN_HEIGHT],
            palette: None,
        }
    }

    fn blit_raw(&mut self, data: &[u8], w: usize, h: usize) {
        if w == SCREEN_WIDTH && h == SCREEN_HEIGHT {
            self.fb.copy_from_slice(&data[0..w * h]);
        } else {
            for (y, row) in self.fb.chunks_mut(SCREEN_WIDTH).enumerate() {
                if y >= h {
                    break;
                }
                let start = y * w;
                let end = start + w;
                row[0..w].copy_from_slice(&data[start..end]);
            }
        }
    }

    fn blit_column(&mut self, data: &[u8], x: usize, y: usize) {
        for (i, p) in data.iter().enumerate() {
            self.fb[(y + i) * SCREEN_WIDTH + x] = *p;
        }
    }

    pub fn draw_patch_raw(&mut self, mut data: &[u8], x: usize, y: usize) {
        let img = &data[..];

        let w = data.read_u16::<LittleEndian>().unwrap() as usize;
        let h = data.read_u16::<LittleEndian>().unwrap() as usize;
        let left = data.read_i16::<LittleEndian>().unwrap() as isize;
        let top = data.read_i16::<LittleEndian>().unwrap() as isize;

        let x = (x as isize - left) as usize;
        let y = (y as isize - top) as usize;

        if (x + w) > SCREEN_WIDTH || (y + h) > SCREEN_HEIGHT {
            panic!("Bad V_DrawPatch");
        }

        for x_ofs in 0..w {
            let mut col_ofs = &data[4 * x_ofs..];
            let mut col_ofs = col_ofs.read_u32::<LittleEndian>().unwrap() as usize;

            let dest_x = x + x_ofs as usize;
            loop {
                let topdelta = img[col_ofs] as usize;
                if topdelta == 255 {
                    break;
                }
                let length = img[col_ofs + 1] as usize;
                let source0 = col_ofs + 3;
                let source1 = source0 + length;
                let dest_y = topdelta + y;
                self.blit_column(&img[source0..source1], dest_x, dest_y);
                col_ofs += length + 4;
            }
        }
    }

    pub fn draw_raw_screen(&mut self, wad: &Wad, lump: &str) {
        if let Some(lump) = wad.cache_lump_name(lump) {
            self.blit_raw(lump, SCREEN_WIDTH, SCREEN_HEIGHT);
        }
    }

    pub fn draw_patch(&mut self, wad: &Wad, x: usize, y: usize, lump: &str) {
        if let Some(lump) = wad.cache_lump_name(lump) {
            self.draw_patch_raw(lump, x, y);
        }
    }

    pub fn set_palette(&mut self, wad: &Wad, lump: &str) {
        if let Some(lump) = wad.cache_lump_name(lump) {
            self.palette = Some(lump.chunks(3).map(|x| [x[0], x[1], x[2], 0xff]).collect())
        }
    }
}

fn setup(mut assets: ResMut<Assets<Image>>, mut commands: Commands) {
    let frame_buffer = Image::new_fill(
        Extent3d {
            width: SCREEN_WIDTH as u32,
            height: SCREEN_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0u8, 0, 0, 0xff],
        TextureFormat::Rgba8UnormSrgb,
    );
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            texture: assets.add(frame_buffer),
            sprite: Sprite {
                custom_size: Some(Vec2::new(
                    SCREEN_WIDTH as f32 * 1.5,
                    SCREEN_HEIGHT as f32 * 1.5,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FrameBuffer);
}

pub fn render(
    vid: Res<Vid>,
    mut assets: ResMut<Assets<Image>>,
    query: Query<(&FrameBuffer, &Handle<Image>)>,
) {
    // Update the framebuffer texture.
    let (_, handle) = query.single();
    let mut image = assets.get_mut(handle).unwrap();
    if let Some(pal) = vid.palette.as_ref() {
        image.data = vid.fb.iter().map(|p| pal[*p as usize]).flatten().collect();
    }
}

pub struct DoomEngine {
    pub wadfile: &'static str,
}

impl Plugin for DoomEngine {
    fn build(&self, app: &mut App) {
        let file = File::open(self.wadfile).expect("Couldn't open main wadfile.");
        let wad = Wad::from_reader(BufReader::new(file)).expect("Error reading main wadfile.");

        app.insert_resource(wad)
            .insert_resource(Vid::new())
            .add_startup_system(setup);
    }
}
