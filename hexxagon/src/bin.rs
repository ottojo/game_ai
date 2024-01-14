use std::time::Duration;

use game_ai::GameAi;
use ggez::glam::Vec2;
use mcts::GenericMonteCarloTreeSearchAi;
//use hexxagon_lib::ai::HexxagonAi;
use hexxagon_lib::game::CellState;
use hexxagon_lib::game::GameResult;
use hexxagon_lib::game::GameState;
use hexxagon_lib::game::Player;

use ggez::event;
use ggez::glam::*;
use ggez::graphics::DrawParam;
use ggez::graphics::TextLayout;
use ggez::graphics::{self, Color};
use ggez::Context;
use hexxagon_lib::game::rules::HexxagonRules;
use hexxagon_lib::hexgrid::AxialVector;

use hexxagon_lib::game::MoveResult;

enum UIState {
    SelectingSource,
    SelectingTarget(AxialVector /* Source */),
}

struct MainState {
    gamestate: GameState,
    uistate: UIState,
    board_position: Vec2,
    cell_size: f32,
    cell_aspect_ratio: f32,
    ai: Box<dyn GameAi<HexxagonRules>>,
}

impl MainState {
    fn new() -> MainState {
        MainState {
            gamestate: GameState::initialize(),
            uistate: UIState::SelectingSource,
            cell_size: 40.0,
            cell_aspect_ratio: 0.5,
            board_position: Vec2::new(0.0, 0.0),
            ai: Box::new(GenericMonteCarloTreeSearchAi::<HexxagonRules>::new(
                mcts::StopCondition::Time(Duration::from_millis(5000)),
            )),
        }
    }
}

fn get_tile_color(state: &CellState) -> Color {
    match state {
        CellState::Empty => graphics::Color::from_rgb(255, 65, 255),
        CellState::Occupied(Player::Pearls) => graphics::Color::from_rgb(0, 0, 203),
        CellState::Occupied(Player::Rubies) => graphics::Color::from_rgb(190, 0, 0),
        CellState::Blocked => graphics::Color::BLACK,
    }
}

fn get_axial_basis_vectors(width: f32, height: f32) -> (Vec2, Vec2) {
    let q_basis_vector = Vec2::new(0.75 * width, 0.5 * height);
    let r_basis_vector = Vec2::new(0.0, height);
    (q_basis_vector, r_basis_vector)
}

impl MainState {
    fn is_in_gameplay_state(&self) -> bool {
        self.gamestate.result().is_none()
    }
}

fn draw_field(
    ctx: &mut Context,
    canvas: &mut graphics::Canvas,
    state: &mut MainState,
) -> ggez::GameResult {
    let tile_width = 2.0 * state.cell_size;
    let tile_height = state.cell_aspect_ratio * tile_width;
    let tile_border_width = 0.1 * state.cell_size;
    let hex_points = [
        // Clockwise order
        Vec2::new(0.25 * tile_width, -0.5 * tile_height),
        Vec2::new(0.5 * tile_width, 0.0),
        Vec2::new(0.25 * tile_width, 0.5 * tile_height),
        Vec2::new(-0.25 * tile_width, 0.5 * tile_height),
        Vec2::new(-0.5 * tile_width, 0.0),
        Vec2::new(-0.25 * tile_width, -0.5 * tile_height),
    ];

    let hex_bg = graphics::Mesh::new_polygon(
        ctx,
        graphics::DrawMode::fill(),
        &hex_points,
        graphics::Color::WHITE,
    )?;
    let hex_outline = graphics::Mesh::new_polygon(
        ctx,
        graphics::DrawMode::stroke(tile_border_width),
        &hex_points,
        graphics::Color::from_rgb(59, 86, 100),
    )?;

    let (q_basis_vector, r_basis_vector) = get_axial_basis_vectors(tile_width, tile_height);

    state.board_position = Vec2::from(canvas.scissor_rect().center());

    for (axial_position, value) in state.gamestate.get_field().tile_iter() {
        let cartesian_position = Vec2::new(
            q_basis_vector.x * axial_position.q() as f32
                + r_basis_vector.x * axial_position.r() as f32,
            q_basis_vector.y * axial_position.q() as f32
                + r_basis_vector.y * axial_position.r() as f32,
        ) + state.board_position;
        canvas.draw(
            &hex_bg,
            DrawParam::new()
                .dest(cartesian_position)
                .color(get_tile_color(value)),
        );

        canvas.draw(&hex_outline, cartesian_position);

        let mut coordinate_text = graphics::Text::new(format!("{}", axial_position));
        coordinate_text.set_layout(TextLayout::center());
        coordinate_text.set_scale(12.0);
        canvas.draw(
            &coordinate_text,
            DrawParam::new()
                .dest(cartesian_position)
                .color(Color::WHITE),
        );
    }
    Ok(())
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> ggez::GameResult {
        // TODO: Input handling?
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> ggez::GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.0, 0.0, 0.0, 1.0]));

        draw_field(ctx, &mut canvas, self)?;

        let fps_text = graphics::Text::new(format!("{} FPS", ctx.time.fps()));
        canvas.draw(&fps_text, DrawParam::new().color(Color::WHITE));

        let score = self.gamestate.scores();
        let mut score_text = graphics::Text::new(format!(
            "Pearls (blue): {}\nRubies (red): {}",
            score.pearls, score.rubies,
        ));
        score_text.set_layout(TextLayout {
            h_align: graphics::TextAlign::End,
            v_align: graphics::TextAlign::End,
        });
        canvas.draw(
            &score_text,
            DrawParam::new()
                .color(Color::WHITE)
                .dest(1.95 * Vec2::from(canvas.scissor_rect().center())),
        );

        if let Some(result) = self.gamestate.result() {
            let mut winner_text = match result {
                GameResult::Tie => graphics::Text::new("Tie!"),
                GameResult::Win(Player::Pearls) => graphics::Text::new("Pearls Win!"),
                GameResult::Win(Player::Rubies) => graphics::Text::new("Rubies Win!"),
            };

            winner_text.set_layout(TextLayout::center());
            winner_text.set_scale(40.0);
            canvas.draw(
                &winner_text,
                DrawParam::new()
                    .color(Color::WHITE)
                    .dest(canvas.scissor_rect().center()),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        if !self.is_in_gameplay_state() {
            println!("Clicked button but not in gameplay state");
            return Ok(());
        }

        // Get axial coordinate from cartesian
        let x = x - self.board_position.x;
        let y = y - self.board_position.y;
        let tile_width = 2.0 * self.cell_size;
        let tile_height = self.cell_aspect_ratio * tile_width;
        let (q_basis_vector, r_basis_vector) = get_axial_basis_vectors(tile_width, tile_height);
        let determinant = q_basis_vector.x * r_basis_vector.y - q_basis_vector.y * r_basis_vector.x;
        let axial_coordinate = AxialVector::round_nearest(
            (x * r_basis_vector.y - y * r_basis_vector.x) / determinant,
            (x * -q_basis_vector.y + y * q_basis_vector.x) / determinant,
        );

        if !self.gamestate.get_field().is_in_bounds(axial_coordinate) {
            //println!("Selection out of bounds");
            return Ok(());
        }

        self.uistate = match self.uistate {
            UIState::SelectingSource => UIState::SelectingTarget(axial_coordinate),
            UIState::SelectingTarget(source) => {
                let player_move_result = self.gamestate.player_move(source, axial_coordinate); // TODO: Display result

                if self.gamestate.result().is_none() && player_move_result == MoveResult::Success {
                    let ai_move = self.ai.determine_next_move(&self.gamestate);
                    let ai_move_result = self.gamestate.player_move(ai_move.src, ai_move.dst);
                    assert_eq!(ai_move_result, MoveResult::Success);

                    println!("AI made move: {:?}", ai_move)
                }

                UIState::SelectingSource
            }
        };

        Ok(())
    }
}

fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("hexxagon", "ottojo");
    let (ctx, event_loop) = cb.build()?;
    ctx.gfx.set_window_title("Hexxagon");
    let state = MainState::new();
    event::run(ctx, event_loop, state)
}
