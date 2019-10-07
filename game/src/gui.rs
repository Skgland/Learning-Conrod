#![allow(dead_code)]

use conrod_core::image::Map;
use conrod_core::Ui;

use std::fmt::Debug;
use conrod_core::widget_ids;
use conrod_core::widget;
use conrod_core::position::Positionable;
use conrod_core::Labelable;
use conrod_core::position::Sizeable;
use conrod_core::widget::Widget;
use piston_window::UpdateArgs;
use conrod_core::UiCell;
use piston_window::PistonWindow;
use glutin_window::GlutinWindow;
use crate::game::GameState;
use crate::game::LevelTemplate;
use conrod_core::image::Id;
use graphics::{Graphics, Context, clear};
use std::path::PathBuf;
use crate::TextureMap;
use std::rc::Rc;
use crate::gui::MenuState::InGame;
use crate::app::{Action, UpdateAction};
use std::collections::BTreeMap;
use std::borrow::Cow;
use piston::input::{ButtonState, ButtonArgs, Button, RenderArgs, Key};

// Generate a unique `WidgetId` for each widget.
widget_ids! {
    pub struct Ids {
        main_canvas,
        menu_title,
        level_buttons[],
        level_selection_button,
        editor_button,
        contiue_button,
        options_button,
        back_button,
        quit_button,
    }
}

pub struct GUI {
    pub image_map: Map<opengl_graphics::Texture>,
    pub image_ids: Vec<Id>,
    pub ui: Ui,
    pub ids: Ids,
    pub fullscreen: bool,
}

#[derive(Debug)]
pub enum MenuState {
    MainMenu,
    PauseMenu(GameState),
    InGame(GameState),
    LevelSelect(LevelSelectState),
}

#[derive(Debug)]
pub struct LevelSelectState(Vec<Rc<LevelTemplate>>);

#[derive(Debug)]
pub enum LevelEditorState {
    Empty,
    Edit { level: LevelTemplate, saved: bool, file: Option<PathBuf> },
    Testing { level: LevelTemplate, saved: bool, file: Option<PathBuf>, state: GameState },
}


pub trait Menu: Debug {
    fn menu_name(&self) -> Cow<'static, str>;

    fn handle_input(&self, event: piston::input::Input);

    fn draw_raw<G: Graphics>(&self, args: &RenderArgs, context: Context, gl: &mut G, texture_map: &TextureMap<G>);

    fn update(&mut self, ui: &mut UiCell, ids: &mut Ids, args: &UpdateArgs);

    fn handle_esc(&mut self, window: &mut PistonWindow<GlutinWindow>) ->  UpdateAction;
}

impl MenuState {
    pub(crate) fn open_level_selection(&mut self) {
        let levels = crate::game::level::loading::load_levels(crate::get_asset_path().as_path()).unwrap_or_else(|_err| Vec::new()).into_iter().map(Rc::new).collect();

        *self = MenuState::LevelSelect(LevelSelectState(levels));
    }
}


impl Menu for MenuState {
    fn menu_name(&self) -> Cow<'static, str> {
        match self {
            MenuState::MainMenu => Cow::Borrowed("Main Menu"),
            MenuState::PauseMenu(_) => Cow::Borrowed("Pause Menu"),
            MenuState::LevelSelect(_) => Cow::Borrowed("Level Selection"),
            MenuState::InGame(_) => Cow::Borrowed("")
        }
    }

    fn handle_input(&self, event: piston::input::Input) {
        use piston::input::*;
        match self {
           MenuState::InGame(state) => {
                if let GameState::GameState {keys_down,..} = state {
                    if let piston::input::Input::Button(ButtonArgs { button: Button::Keyboard(key), state: button_state, .. }) = event {
                        match button_state {
                            ButtonState::Press => keys_down.try_borrow_mut().unwrap().insert(key),
                            ButtonState::Release => keys_down.try_borrow_mut().unwrap().remove(&key),
                        };
                        //println!("{:?}", key);
                    };
                }
            }
            _ => {}
        }
    }

    fn draw_raw<G: Graphics>(&self, args: &RenderArgs, context: Context, gl: &mut G, texture_map: &TextureMap<G>) {
        match self {
            MenuState::InGame(game_state) => {
                clear(crate::game::color::D_RED, gl);
                game_state.draw_game(args, context, gl, texture_map);
            }
            _ => {
                clear(crate::game::color::BLUE, gl);
            }
        }
    }

    fn update(&mut self, ui: &mut UiCell, ids: &mut Ids, args: &UpdateArgs) {
        match self {
            MenuState::PauseMenu(_state) => {
                widget::Text::new("Pause Menu").font_size(30).mid_top_of(ids.main_canvas).set(ids.menu_title, ui);
                widget::Button::new().label("Continue")
                                     .label_font_size(30)
                                     .middle_of(ids.main_canvas)
                                     .padded_kid_area_wh_of(ids.main_canvas, ui.win_h / 4.0)
                                     .set(ids.contiue_button, ui);
            }
            MenuState::LevelSelect(level_list) => {
                widget::Text::new("Level Selection").font_size(30).mid_top_of(ids.main_canvas).set(ids.menu_title, ui);


                ids.level_buttons.resize(level_list.0.len(), &mut ui.widget_id_generator());


                for (button_id, level) in ids.level_buttons.iter().zip(level_list.0.iter()) {
                    let clicked = widget::Button::new().label(&level.name).set(*button_id, ui);
                    if clicked.was_clicked() {
                        let state = GameState::new(level.clone());
                        *self = InGame(state);
                        break;
                    }
                }
            }
            MenuState::MainMenu => {
                widget::Button::new().label("Level Editor").middle_of(ids.main_canvas).set(ids.editor_button, ui);
                widget::Text::new("Main Menu").font_size(30).mid_top_of(ids.main_canvas).set(ids.menu_title, ui);
            }
            MenuState::InGame(state) => {
                match state {
                    GameState::Won { .. } => {
                        widget::Text::new("Won").font_size(30).mid_top_of(ids.main_canvas).set(ids.menu_title, ui)
                    }
                    GameState::GameState { show_hud, rotation, keys_down, .. } => {
                        if *show_hud {
                            widget::Text::new("HUD").font_size(30).mid_top_of(ids.main_canvas).set(ids.menu_title, ui)
                        }

                        // Rotate 2 radians per second.
                        *rotation += 8.0 * args.dt;

                        let mut key_map: BTreeMap<Key, Action> = BTreeMap::new();

                        key_map.insert(Key::W, Action::UP);
                        key_map.insert(Key::A, Action::LEFT);
                        key_map.insert(Key::S, Action::DOWN);
                        key_map.insert(Key::D, Action::RIGHT);

                        let down_clone = keys_down.clone();

                        key_map.iter().filter(|(&k, _)| down_clone.borrow().contains(&k)).for_each(|(_, action)| action.perform(state));
                    }
                }
                state.handle_input();
            }
        }
    }


    fn handle_esc(&mut self, window: &mut PistonWindow<GlutinWindow>) -> UpdateAction {
        match self {
            MenuState::MainMenu => {
                return UpdateAction::Close
            },
            MenuState::PauseMenu(_state) => {
                self.open_level_selection()
            }
            MenuState::LevelSelect(_) => {
                *self = MenuState::MainMenu;
            }
            InGame(state) => { *self = MenuState::PauseMenu(state.clone()) }
        }

        UpdateAction::Nothing
    }
}

