use log::*;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

use js_sys::Math::random;
use std::cmp::PartialEq;
use wasm_bindgen::JsCast;
use web_sys::{self, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::format::Json;
use yew::prelude::*;
use yew::services::interval::IntervalTask;
use yew::services::keyboard::{KeyListenerHandle, KeyboardService};
use yew::services::storage::{Area, StorageService};
use yew::services::IntervalService;
use yew::utils::document;

const KEY: &str = "high.score";
const TICK_RATE: u64 = 200;
pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    ctx: Option<(HtmlCanvasElement, CanvasRenderingContext2d)>,
    job: Option<IntervalTask>,
    keyboard_service: Option<Vec<KeyListenerHandle>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    snake: Vec<Coords>,
    high_score: usize,
    velocity: Velocity,
    accepting_inputs: bool,
    draw_grid: bool,
    apple: Coords,
    score: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Velocity {
    coords: Coords,
    direction: Direction,
}

#[derive(Serialize, Deserialize, Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Coords {
    x: f64,
    y: f64,
}

fn coords<T>(x: T, y: T) -> Coords
where
    f64: From<T>,
{
    Coords {
        x: x.into(),
        y: y.into(),
    }
}

impl PartialEq for Coords {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Coords {
    fn add(&self, other: &Self) -> Self {
        coords(self.x + other.x, self.y + other.y)
    }
    fn random(multi: usize, multi2: usize) -> Self {
        let multi = multi as f64;
        let multi2 = multi2 as f64;
        Coords {
            x: (random() * multi / multi2).floor() * multi2,
            y: (random() * multi / multi2).floor() * multi2,
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    Tick,
    None,
    Up,
    Left,
    Right,
    Down,
    Restart,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let high_score = {
            if let Json(Ok(restored_entries)) = storage.restore(KEY) {
                restored_entries
            } else {
                0
            }
        };
        let state = State {
            snake: vec![
                coords(200, 200),
                coords(180, 200),
                coords(160, 200),
                coords(140, 200),
            ],
            high_score,
            velocity: Velocity {
                coords: coords(20, 0),
                direction: Direction::Left,
            },
            accepting_inputs: true,
            draw_grid: false,
            score: 0,
            apple: coords(0, 0),
        };
        App {
            link,
            storage,
            state,
            ctx: None,
            job: None,
            keyboard_service: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => &self.tick(),
            Msg::Left | Msg::Right | Msg::Up | Msg::Down => &self.keydown(msg),
            Msg::Restart if self.game_over() => &self.start(),
            _ => &(),
        };
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let
        html! {
            <div class="container">
                <center>
                    <h1> {"Score:"} { self.state.score }</h1>
                    <br />
                    <h1> {"High Score:"} { self.state.high_score } </h1>
                </center>
                <div class="canvasContainer">
                    <canvas id="canvas" width= "500px" height="500px">
                    </canvas>
                </div>

            </div>
        }
    }

    fn rendered(&mut self, _first_render: bool) {
        if self.ctx == None {
            let canvas: HtmlCanvasElement = document()
                .query_selector("#canvas")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into()
                .unwrap();
            self.ctx = Some((canvas, ctx));
            self.start();
        }
    }
}

impl App {
    fn start(&mut self) {
        self.state.snake = vec![
            coords(200, 200),
            coords(180, 200),
            coords(160, 200),
            coords(140, 200),
            coords(120, 200),
        ];
        self.state.velocity = Velocity {
            coords: coords(20, 0),
            direction: Direction::Left,
        };
        self.state.apple = generate_apple(&self.state.snake);
        self.tick();
        let handle = IntervalService::spawn(
            Duration::from_millis(TICK_RATE),
            self.link.callback(|_| Msg::Tick),
        );
        self.job = Some(handle);
        self.keyboard_service = Some(self.make_keyboard_service());
    }
    fn tick(&mut self) {
        self.animate();
        let over = &self.game_over();
        self.render();

        self.state.accepting_inputs = true;
        if !over {
        } else {
            self.job = None;
            info!("Game over!");
            self.set_highscore();
        }
    }

    fn game_over(&self) -> bool {
        if self.state.snake[0].x.abs() > 460.
            || self.state.snake[0].y.abs() > 460.
            || self.state.snake[0].x < 20.
            || self.state.snake[0].y < 20.
            || self.bite()
        {
            true
        } else {
            false
        }
    }
    fn animate(&mut self) {
        self.state
            .snake
            .splice(0..0, [self.state.snake[0].add(&self.state.velocity.coords)]);
        if self.state.snake[0] == self.state.apple {
            self.state.score += 1;
            self.state.apple = generate_apple(&self.state.snake);
            info!("score {}", self.state.score);
        } else {
            self.state.snake.pop();
        };
    }
    fn render(&mut self) {
        self.clear();
        if let Some((_canvas, ctx)) = &self.ctx {
            ctx.set_fill_style(&"red".into());
            ctx.fill_rect(self.state.apple.x, self.state.apple.y, 20., 20.);
            ctx.set_fill_style(&"#010101".into());
            for coords in &self.state.snake[..] {
                ctx.fill_rect(coords.x, coords.y, 20., 20.);
            }
        }
    }
    fn clear(&self) {
        if let Some((canvas, ctx)) = &self.ctx {
            ctx.set_fill_style(&"#efefef".into());
            ctx.fill_rect(0., 0., canvas.width().into(), canvas.height().into());
            if self.state.draw_grid {
                let positions = (20..500).step_by(20).map(|x| x as f64);
                ctx.set_stroke_style(&"#aeaeae80".into());
                ctx.set_line_width(2.);
                let width = canvas.width() as f64;
                let height = canvas.height() as f64;
                for i in positions {
                    ctx.move_to(i, 0.);
                    ctx.line_to(i, height);
                    ctx.move_to(0., i);
                    ctx.line_to(width, i);
                }
                ctx.stroke();
            }
        }
    }
    fn make_keyboard_service(&self) -> Vec<KeyListenerHandle> {
        let mut services: Vec<KeyListenerHandle> = Vec::with_capacity(4);
        let handler = KeyboardService::register_key_down(
            &document(),
            self.link.callback(|key: KeyboardEvent| {
                return match &key.key().replace("Arrow", "")[..] {
                    "Left" | "a" => Msg::Left,
                    "Right" | "d" => Msg::Right,
                    "Up" | "w" => Msg::Up,
                    "Down" | "s" => Msg::Down,
                    "r" | " " => Msg::Restart,
                    _ => Msg::None,
                };
            }),
        );
        services.push(handler);
        services
    }
    fn keydown(&mut self, msg: Msg) {
        if !self.state.accepting_inputs {
            return;
        }
        use Direction::*;

        let (x, y, dir) = match (msg, &self.state.velocity.direction) {
            (Msg::Left, Up | Down) => (-20, 0, Left),
            (Msg::Right, Up | Down) => (20, 0, Right),
            (Msg::Up, Left | Right) => (0, -20, Up),
            (Msg::Down, Left | Right) => (0, 20, Down),
            _ => return,
        };
        self.state.velocity.coords = coords(x, y);
        self.state.velocity.direction = dir;
        self.state.accepting_inputs = false;
    }
    fn bite(&self) -> bool {
        let mut snake: Vec<Coords> = self.state.snake.clone();
        let head = snake.remove(0);
        for part in snake.iter() {
            if part.x == head.x && part.y == head.y {
                return true;
            }
        }
        false
    }
    fn set_highscore(&mut self) {
        if self.state.high_score < self.state.score {
            self.state.high_score = self.state.score;
            self.storage.store(KEY, Json(&self.state.high_score))
        }
    }
}

fn generate_apple(snake: &Vec<Coords>) -> Coords {
    let apple = Coords::random(500, 20);
    if let Some(_) = snake
        .iter()
        .find(|pos| pos.x == apple.x && pos.y == apple.y)
    {
        info!("apple 1 {:?}", apple);
        return generate_apple(snake);
    }
    if apple.x.abs() > 460. || apple.y.abs() > 460. || apple.x < 20. || apple.y < 20. {
        info!("apple 2 {:?}", apple);
        return generate_apple(snake);
    }
    apple
}
