extern crate piston;
extern crate piston_window;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use opengl_graphics::{GlGraphics, OpenGL};
//use conrod_core::color::Colorable;
use piston_window::TextureSettings;
use piston_window::PistonWindow;
use conrod_core::image::Map;

mod gui;
mod app;
mod game;

use app::*;
use gui::*;
use conrod_core::Ui;
use rusttype::gpu_cache::Cache;
use opengl_graphics::Texture;
use glutin_window::GlutinWindow;
use crate::gui::MenuType::LevelSelect;
use crate::game::TileTextureIndex;
use std::collections::btree_map::BTreeMap;
use crate::game::LevelTemplate;
use toml::Deserializer;
use std::fs::File;
use std::io::Read;
use serde::ser::Serialize;
use toml::ser::Error::Custom;
use std::io::Write;
use std::path::PathBuf;

extern crate find_folder;

//
//Initial Setting
//

// Change this to OpenGL::V2_1 if not working.

const OPEN_GL_VERSION: OpenGL = OpenGL::V3_2;
const INIT_WIDTH: u32 = 200;
const INIT_HEIGHT: u32 = 200;

fn main() -> Result<(), toml::ser::Error> {
    let mut window = create_window();

    let ui = create_ui();

    println!("Writing test level to disc!");
    /*if let Err(e) = save_level(get_asset_path().join("levels").join("test.level").as_path(), &gui::test_level()) {
        eprintln!("{}", e);
    }*/


    println!("Construction app!");
    // Create a new game and run it.
    let mut app = create_app(ui);


    println!("Creating render Context!");
    let mut context = create_render_context();


    println!("Creating event loop iterator");
    let mut events = Events::new(EventSettings::new());


    while let Some(e) = events.next(&mut window) {
        e.render(|r| app.render(&mut context, r));

        if let Event::Input(i) = e {
            app.input(i, &mut window);
        } else {
            e.update(|u| app.update(u, &mut window));
        }
    }

    Ok(())
}

struct TextCache<'font> {
    text_vertex_data: Vec<u8>,
    glyph_cache: Cache<'font>,
    text_texture_cache: Texture,
}

fn create_text_cache<'font>(_: &()) -> TextCache {
    // Create a texture to use for efficiently caching text on the GPU.
    let text_vertex_data: Vec<u8> = Vec::new();
    let (glyph_cache, text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = conrod_core::text::GlyphCache::builder()
            .dimensions(INIT_WIDTH, INIT_HEIGHT)
            .scale_tolerance(SCALE_TOLERANCE)
            .position_tolerance(POSITION_TOLERANCE)
            .build();
        let buffer_len = INIT_WIDTH as usize * INIT_HEIGHT as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let texture = opengl_graphics::Texture::from_memory_alpha(&init, INIT_WIDTH, INIT_HEIGHT, &settings).unwrap();
        (cache, texture)
    };
    TextCache { text_vertex_data, glyph_cache, text_texture_cache }
}

fn create_window() -> PistonWindow<GlutinWindow> {
    // Create an Glutin window.
    WindowSettings::new(
        "spinning-square",
        [INIT_WIDTH, INIT_HEIGHT],
    ).opengl(OPEN_GL_VERSION)
     .vsync(true)
     .fullscreen(false)
     .build()
     .unwrap()
}

fn get_asset_path() -> PathBuf {
    find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap()
}

fn create_ui() -> Ui {

    //construct Ui
    let mut ui = conrod_core::UiBuilder::new([INIT_WIDTH as f64, INIT_HEIGHT as f64])
        .build();


    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = get_asset_path();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();
    ui
}

type TextureMap = std::collections::btree_map::BTreeMap<TileTextureIndex, Texture>;

fn load_levels() -> Vec<LevelTemplate> {
    let assets = get_asset_path();
    let path = assets.join("levels");
    let mut levels = vec![];
    if let Ok(dir) = path.read_dir() {
        for entry in dir {
            if let Ok(entry) = entry {
                if let Ok(f_type) = entry.file_type() {
                    if f_type.is_file() {
                        if let Ok(level) = load_level(entry.path().as_path()) {
                            levels.push(level);
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Expected assets/levels to be a folder, but it wasn't!");
    }
    levels
}

fn load_level(path: &std::path::Path) -> Result<LevelTemplate, toml::de::Error> {
    let mut content = vec![];

    use serde::Deserialize;
    use game::*;

    if let Err(_) = File::open(path).unwrap().read_to_end(&mut content) {
        return toml::de::from_str("Failed to read File!");
    };

    let smthg = String::from_utf8_lossy(content.as_slice()).to_string();
    let mut des = Deserializer::new(smthg.as_str());

    LevelTemplate::deserialize(&mut des)
}

fn save_level(path: &std::path::Path, level: &LevelTemplate) -> Result<(), toml::ser::Error> {
    let mut out = String::new();

    let mut serializer = toml::ser::Serializer::new(&mut out);

    level.serialize(&mut serializer)?;

    if let Ok(mut file) = File::create(path) {
        if let Err(_) = file.write_all(out.as_bytes()) {
            Err(Custom("Failed to write to file!".to_string()))
        } else {
            Ok(())
        }
    } else {
        Err(Custom("Failed to write to File!".to_string()))
    }
}

fn load_textures(texture_map: &mut TextureMap) -> () {

    load_texture_into_map(texture_map, TileTextureIndex::Goal { active: true }, "goal.png");
    load_texture_into_map(texture_map, TileTextureIndex::Goal { active: false }, "goal.png");
    load_texture_into_map(texture_map, TileTextureIndex::Start, "start.png");
    load_texture_into_map(texture_map, TileTextureIndex::Path, "path.png");


/*
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: false, left: false, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: false, left: false, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: false, left: true, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: false, left: true, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: true, left: false, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: true, left: false, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: true, left: true, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: false, down: true, left: true, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: false, left: false, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: false, left: false, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: false, left: true, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: false, left: true, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: true, left: false, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: true, left: false, right: true }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: true, left: true, right: false }), "wall.png");
    load_texture_into_map(texture_map, TileTextureIndex::Wall(Connections { up: true, down: true, left: true, right: true }), "wall.png");
    */
}

fn load_texture_into_map(texture_map: &mut TextureMap, key: TileTextureIndex, name: &str) -> () {
    let assets = get_asset_path();
    let path = assets.join("textures").join(name);
    let settings = TextureSettings::new();
    if let Ok(texture) = Texture::from_path(&path, &settings) {
        texture_map.insert(key, texture);
    }
}

fn create_app(mut ui: Ui) -> App {


    // Load the rust logo from file to a piston_window texture.
    //let test_texture = load_texture("test.png");

    // Create our `conrod_core::image::Map` which describes each of our widget->image mappings.
    // In our case we have no image, however the macro may be used to list multiple.
    let image_map: Map<opengl_graphics::Texture> = conrod_core::image::Map::new();
    //let test_texture = image_map.insert(test_texture);

    let mut texture_map: TextureMap = BTreeMap::new();

    load_textures(&mut texture_map);

    let level_list = load_levels();

    // Instantiate the generated list of widget identifiers.
    let generator = ui.widget_id_generator();
    let ids = Ids::new(generator);

    App::new(
        GUI {
            ui,
            ids,
            image_ids: vec![],
            image_map,
            active_menu: GUIVisibility::MenuOnly(LevelSelect),
            fullscreen: false,
        }, texture_map, level_list,
    )
}

fn create_render_context<'font>() -> RenderContext<'font> {
    let TextCache { text_vertex_data, glyph_cache, text_texture_cache } = create_text_cache(&());
    let gl = GlGraphics::new(OPEN_GL_VERSION);
    RenderContext {
        text_texture_cache,
        glyph_cache,
        text_vertex_data,
        gl,
    }
}