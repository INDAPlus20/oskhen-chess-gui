use chess::game::{Game, GameState, Rank, Team};
use chess::moves::{Action, ActionType};
use chess::pgn;
#[allow(unused_imports)]

/**
 * Chess GUI template.
 * Author: Eskil Queseth <eskilq@kth.se>, Viola Söderlund <violaso@kth.se>
 * Last updated: 2020-09-20
 */
use ggez::event;
use ggez::event::MouseButton;
use ggez::event::{EventHandler, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawMode, DrawParam};
use ggez::input::keyboard;
use ggez::{Context, GameResult};
use std::io;
use std::path;

const MULTIPLE_SCREEN: f32 = 1.5;

/// A chess board is 8x8 tiles.
const GRID_SIZE: (i16, i16) = (8, 8);
/// Sutible size of each tile.
const GRID_CELL_SIZE: (i16, i16) = (
    (45.0 * MULTIPLE_SCREEN) as i16,
    (45.0 * MULTIPLE_SCREEN) as i16,
);

/// Size of the application window.
const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

// GUI Color representations
const BLACK: Color = Color::new(60.0 / 255.0, 60.0 / 255.0, 60.0 / 255.0, 1.0);
const WHITE: Color = Color::new(120.0 / 255.0, 120.0 / 255.0, 120.0 / 255.0, 1.0);
const AVAILABLE_TILE: Color = Color::new(190.0 / 255.0, 120.0 / 255.0, 100.0 / 255.0, 0.5);

const REPLAY_BUTTON_SIZE: (f32, f32) = (120f32, 120f32);

/// GUI logic and event implementation structure.
struct AppState {
    sprites: Vec<((Team, Rank), graphics::Image)>,
    board: Game,
    // Save piece positions, which tiles has been clicked, current Team, etc...
    available_tiles: Vec<Tile>,
    selected_piece: Option<Tile>,
    available_actions: Vec<Action>,
    state: State,
    is_replay: bool,
    text:String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Active,
    Gameover,
    Pause,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tile {
    pos: BoardPosition,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoardPosition {
    x: isize,
    y: isize,
}

impl BoardPosition {
    fn new(pos: (isize, isize)) -> BoardPosition {
        BoardPosition { x: pos.0, y: pos.1 }
    }
    fn to_letter(&self) -> String {
        let row_letter: String = (self.y).to_string();
        let column_number = self.x + 1;
        let column_letter = match column_number {
            1 => "a",
            2 => "b",
            3 => "c",
            4 => "d",
            5 => "e",
            6 => "f",
            7 => "g",
            8 => "h",
            _ => panic!("there shouldnt be a out of bounds letter here"),
        };
        String::from(column_letter) + &row_letter
    }
}

impl From<BoardPosition> for graphics::Rect {
    fn from(pos: BoardPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_CELL_SIZE.0 as i32,
            pos.y as i32 * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

impl From<BoardPosition> for ggez::mint::Point2<f32> {
    fn from(pos: BoardPosition) -> Self {
        ggez::mint::Point2 {
            x: pos.x as f32 * GRID_CELL_SIZE.0 as f32,
            y: (7 - pos.y) as f32 * GRID_CELL_SIZE.1 as f32,
        }
    }
}

impl Tile {
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            self.pos.into(),
            AVAILABLE_TILE,
        )?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))
    }
}

impl AppState {
    /// Initialise new application, i.e. initialise new game and load resources.
    fn new(ctx: &mut Context) -> GameResult<AppState> {
        let sprites = AppState::load_sprites();
        let board = Game::new();

        let state = AppState {
            sprites: sprites
                .iter()
                .map(|_sprite| {
                    (
                        _sprite.0,
                        graphics::Image::new(ctx, _sprite.1.clone()).unwrap(),
                    )
                })
                .collect::<Vec<((Team, Rank), graphics::Image)>>(),
            board,
            available_tiles: vec![],
            available_actions: vec![],
            selected_piece: None,
            state: State::Active,
            is_replay: false,
            text:String::new(),
        };

        Ok(state)
    }

    /// Loads chess piese images into vector.
    fn load_sprites() -> Vec<((Team, Rank), String)> {
        let mut sprites = Vec::new();
        sprites.push(((Team::Black, Rank::King), "/black_king.png".to_string()));
        sprites.push(((Team::Black, Rank::Queen), "/black_queen.png".to_string()));
        sprites.push(((Team::Black, Rank::Rook), "/black_rook.png".to_string()));
        sprites.push(((Team::Black, Rank::Pawn), "/black_pawn.png".to_string()));
        sprites.push(((Team::Black, Rank::Bishop), "/black_bishop.png".to_string()));
        sprites.push(((Team::Black, Rank::Knight), "/black_knight.png".to_string()));
        sprites.push(((Team::White, Rank::King), "/white_king.png".to_string()));
        sprites.push(((Team::White, Rank::Queen), "/white_queen.png".to_string()));
        sprites.push(((Team::White, Rank::Rook), "/white_rook.png".to_string()));
        sprites.push(((Team::White, Rank::Pawn), "/white_pawn.png".to_string()));
        sprites.push(((Team::White, Rank::Bishop), "/white_bishop.png".to_string()));
        sprites.push(((Team::White, Rank::Knight), "/white_knight.png".to_string()));
        sprites
    }
}

/// Implement each stage of the application event loop.
impl event::EventHandler for AppState {
    /// For updating game logic, which front-end doesn't handle.
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.is_replay {
            self.board = Game::new();
            self.available_tiles = vec![];
            self.selected_piece = None;
            self.available_actions = vec![];
            self.state = State::Active;
            self.is_replay = false;
        }
        Ok(())
    }

    /// Draw interface, i.e. draw game board
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // clear interface with gray background Team

        match self.state {
            State::Active => {
                graphics::clear(ctx, [0.5, 0.5, 0.5, 1.0].into());
                // create text representation
                let text=self.text.clone();
                let state_text = graphics::Text::new(
                    graphics::TextFragment::from((text))
                    .scale(graphics::Scale { x: 30.0, y: 30.0 }),
                );

                // get size of text
                let text_dimensions = state_text.dimensions(ctx);
                // create background rectangle with white coulouring
                let background_box = graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    graphics::Rect::new(
                        (SCREEN_SIZE.0 - text_dimensions.0 as f32) / 2f32 as f32 - 8.0,
                        (SCREEN_SIZE.0 - text_dimensions.1 as f32) / 2f32 as f32,
                        text_dimensions.0 as f32 + 16.0,
                        text_dimensions.1 as f32,
                    ),
                    [1.0, 1.0, 1.0, 1.0].into(),
                )?;

                // draw background
                graphics::draw(ctx, &background_box, DrawParam::default());

                // draw tiles
                for i in 0..64 {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new_i32(
                            i % 8 * GRID_CELL_SIZE.0 as i32,
                            i / 8 * GRID_CELL_SIZE.1 as i32,
                            GRID_CELL_SIZE.0 as i32,
                            GRID_CELL_SIZE.1 as i32,
                        ),
                        match i % 2 {
                            0 => match i / 8 {
                                _row if _row % 2 == 0 => WHITE,
                                _ => BLACK,
                            },
                            _ => match i / 8 {
                                _row if _row % 2 == 0 => BLACK,
                                _ => WHITE,
                            },
                        },
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },));
                }

                for available_tile in self.available_tiles.iter() {
                    let board_position: ggez::mint::Point2<f32> = available_tile.pos.into();
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(
                            board_position.x,
                            board_position.y,
                            GRID_CELL_SIZE.0 as f32,
                            GRID_CELL_SIZE.1 as f32,
                        ),
                        AVAILABLE_TILE,
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },));
                }

                //draw pieces
                for square_column in self.board.matrix.iter() {
                    for square in square_column {
                        if let Some(piece) = square.piece {
                            let team_rank = (piece.team, piece.rank);
                            let sprite = &self
                                .sprites
                                .iter()
                                .filter(|l: &&((Team, Rank), graphics::Image)| l.0 == team_rank)
                                .next()
                                .unwrap()
                                .1;
                            let board_position = BoardPosition::new(square.coordinate);
                            graphics::draw(
                                ctx,
                                sprite,
                                DrawParam::default()
                                    .scale(ggez::mint::Point2 {
                                        x: GRID_CELL_SIZE.0 as f32 / sprite.width() as f32,
                                        y: GRID_CELL_SIZE.1 as f32 / sprite.height() as f32,
                                    })
                                    .dest(board_position),
                            );
                        }
                    }
                }

                // draw text with dark gray Teaming and center position
                graphics::draw(
                    ctx,
                    &state_text,
                    DrawParam::default()
                        .color([0.0, 0.0, 0.0, 1.0].into())
                        .dest(ggez::mint::Point2 {
                            x: (SCREEN_SIZE.0 - text_dimensions.0 as f32) / 2f32 as f32,
                            y: (SCREEN_SIZE.0 - text_dimensions.1 as f32) / 2f32 as f32,
                        }),
                );
            }
            //pause menu
            _ => {
                let background_box = graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    graphics::Rect::new(
                        0 as f32,
                        0 as f32,
                        SCREEN_SIZE.0 as f32,
                        SCREEN_SIZE.1 as f32,
                    ),
                    Color::new(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.5),
                )?;

                // draw background
                graphics::draw(
                    ctx,
                    &background_box,
                    DrawParam::default().color(Color::new(
                        255.0 / 255.0,
                        255.0 / 255.0,
                        255.0 / 255.0,
                        0.2,
                    )),
                );

                let replay_box = graphics::Mesh::new_rectangle(
                    ctx,
                    DrawMode::fill(),
                    graphics::Rect::new(
                        SCREEN_SIZE.0 as f32 / 2f32 - REPLAY_BUTTON_SIZE.0 as f32/2f32,
                        SCREEN_SIZE.1 as f32 / 2f32 - REPLAY_BUTTON_SIZE.0 as f32/2f32,
                        REPLAY_BUTTON_SIZE.0 as f32,
                        REPLAY_BUTTON_SIZE.1 as f32,
                    ),
                    Color::new(200.0 / 200.0, 200.0 / 255.0, 150.0 / 255.0, 1.0),
                )?;

                graphics::draw(
                    ctx,
                    &replay_box,
                    DrawParam::default()
                );

                let replay_text = graphics::Text::new(
                    graphics::TextFragment::from(format!(
                        "Restart!"
                    ))
                    .scale(graphics::Scale { x: 30.0, y: 30.0 }),
                );
                let text_dimension=replay_text.dimensions(ctx);
                graphics::draw(
                    ctx,
                    &replay_text,
                    DrawParam::default()
                        .color([0.0, 0.0, 0.0, 1.0].into())
                        .dest(ggez::mint::Point2 {
                            x: (SCREEN_SIZE.0 - text_dimension.0 as f32) / 2f32 as f32,
                            y: (SCREEN_SIZE.0 - text_dimension.1 as f32) / 2f32 as f32,
                        }),
                );

                
                    let promotion_ranks = [Rank::Queen, Rank::Bishop, Rank::Rook, Rank::Knight];
                    for x in 0..4 {
                        let team_rank = (self.board.player, promotion_ranks[x]);
                        let sprite = &self
                            .sprites
                            .iter()
                            .filter(|l: &&((Team, Rank), graphics::Image)| l.0 == team_rank)
                            .next()
                            .unwrap()
                            .1;

                        graphics::draw(
                            ctx,
                            sprite,
                            DrawParam::default()
                                .scale(ggez::mint::Point2 {
                                    x: GRID_CELL_SIZE.0 as f32 / sprite.width() as f32,
                                    y: GRID_CELL_SIZE.1 as f32 / sprite.height() as f32,
                                })
                                .dest(ggez::mint::Point2 {
                                    x: 10 as f32,
                                    y: SCREEN_SIZE.1 as f32 / 2f32
                                        + GRID_CELL_SIZE.1 as f32 * x as f32,
                                }),
                        );
                    }
                
            }
        }

        // render updated graphics
        graphics::present(ctx)?;

        Ok(())
    }

    /// Update game on mouse click
    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match self.state {
            State::Active => {
                if button == MouseButton::Left {
                    /* check click position and update board accordingly */
                
                    let game_x = (x / GRID_CELL_SIZE.0 as f32) as isize;
                    let game_y = 7 - (y / GRID_CELL_SIZE.1 as f32) as isize;
                    let clicked_tile = Tile {
                        pos: BoardPosition::new((game_x, game_y)),
                    };
                    if self.selected_piece.is_some() && clicked_tile == self.selected_piece.unwrap()
                    {
                        return;
                    }

                    if let Ok(actions) = self
                        .board
                        .move_from_string(&coordinate_to_string((game_x, game_y)))
                    {
                        self.available_tiles.clear();
                        self.available_actions = actions;
                        for a in &self.available_actions {
                            let board_position = BoardPosition::new(a.to.coordinate);
                            let this_available = Tile {
                                pos: board_position,
                            };
                            self.available_tiles.push(this_available)
                        }
                    } else if !self.available_tiles.is_empty() {
                        for (i, a) in self.available_tiles.iter().enumerate() {
                            if clicked_tile == *a {
                                if self.available_actions[i].action_type == ActionType::Promotion {      
                                    if self.board.promotion_piece==None{
                                        self.text=String::from("Set promotion piece in menu. Press Q for menu.");
                                        self.available_tiles.clear();
                                        self.available_actions.clear();
                                        return;
                                    }    
                                }
                                self.board.perform_action(self.available_actions[i]);
                                self.available_tiles.clear();
                                self.available_actions.clear();
                                break;
                            }
                        }
                    }

                    if self.board.get_game_state() == GameState::Checkmate {
                        self.state = State::Gameover;
                    }
                    self.text=format!("Gamestate:{:?}",self.board.get_game_state())
                }
            }
            _ => {
                if x > SCREEN_SIZE.0 as f32 / 2f32 - REPLAY_BUTTON_SIZE.0 as f32/2.0
                    && x < SCREEN_SIZE.0 as f32 / 2f32 + REPLAY_BUTTON_SIZE.0 as f32/2.0
                {
                    if y > SCREEN_SIZE.1 as f32 / 2f32 - REPLAY_BUTTON_SIZE.1 as f32/2.0
                        && y < SCREEN_SIZE.1 as f32 / 2f32 + REPLAY_BUTTON_SIZE.1 as f32/2.0
                    {
                        self.is_replay = true;
                    }
                }

                let promotion_ranks = [Rank::Queen, Rank::Bishop, Rank::Rook, Rank::Knight];
                    if 10f32 < x && x < (10f32 + GRID_CELL_SIZE.0 as f32) {
                        let index_y = (y - (SCREEN_SIZE.1 as f32 / 2f32)) as i16 / GRID_CELL_SIZE.1;
                        if index_y >= 0 && index_y < 4 {
                            self.board
                                .set_promotion_piece(promotion_ranks[index_y as usize]);
                            self.state=State::Active;
                        }
                    }
            }
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, key: KeyCode, mods: KeyMods, _: bool) {
        match key {
            // Quit if Shift+Ctrl+Q is pressed.
            KeyCode::Q => {
                match self.state {
                    State::Pause => self.state = State::Active,
                    State::Active => self.state = State::Pause,
                    _ => {}
                }
                // if mods.contains(KeyMods::SHIFT & KeyMods::CTRL) {
                //     let mut input = String::new();
                //     println!("Do you want to replay your game? Yes or No");
                //     io::stdin().read_line(&mut input).unwrap();
                //     let mut replay = false;
                //     loop {
                //         match input.trim().to_lowercase() {
                //             s if s == "yes" || s == "y" => {
                //                 replay = true;
                //                 break;
                //             }
                //             s if s == "no" || s == "n" => {
                //                 replay = false;
                //                 break;
                //             }
                //             _ => {
                //                 println!("Invalid option. Try again");
                //             }
                //         };
                //     }

                //     println!("Terminating!");
                //     event::quit(ctx)
                // } else if mods.contains(KeyMods::SHIFT) || mods.contains(KeyMods::CTRL) {
                //     println!("You need to hold both Shift and Control to quit.");
                // } else {
                //     println!("Now you're not even trying!");
                // }
            }
            _ => (),
        }
    }
}

fn coordinate_to_string(coordinate: (isize, isize)) -> String {
    let row_letter: String = (coordinate.1 + 1).to_string();
    let column_number = coordinate.0 + 1;
    let column_letter = match column_number {
        1 => "a",
        2 => "b",
        3 => "c",
        4 => "d",
        5 => "e",
        6 => "f",
        7 => "g",
        8 => "h",
        _ => panic!("there shouldnt be a out of bounds letter here"),
    };

    String::from(column_letter) + &row_letter
}


pub fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./custom_resources");

    let context_builder = ggez::ContextBuilder::new("schack", "vem vet")
        .add_resource_path(resource_dir) // Import image files to GGEZ
        .window_setup(
            ggez::conf::WindowSetup::default()
                .title("Schack") // Set window title "Schack"
                .icon("/icon.ico"), // Set application icon
        )
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1) // Set window dimenstions
                .resizable(false), // Fixate window size
        );

    let (contex, event_loop) = &mut context_builder.build()?;

    let state = &mut AppState::new(contex)?;
    event::run(contex, event_loop, state); // Run window event loop

    Ok(())
}
