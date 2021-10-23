use log::*;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{self, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::format::Json;
use yew::prelude::*;
use yew::services::keyboard::{KeyListenerHandle, KeyboardService};
use yew::services::storage::{Area, StorageService};
use yew::services::timeout::TimeoutTask;
use yew::services::TimeoutService;
use yew::utils::document;

const KEY: &str = "high.score";
const TICK_RATE: u64 = 200;
pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    ctx: Option<(HtmlCanvasElement, CanvasRenderingContext2d)>,
    job: Option<TimeoutTask>,
    keyboard_service: Option<Vec<KeyListenerHandle>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    snake: Vec<Coords>,
    high_score: usize,
    velocity: Velocity,
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

#[derive(Serialize, Deserialize, Debug)]
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

impl Coords {
    fn add(&self, other: &Self) -> Self {
        coords(self.x + other.x, self.y + other.y)
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
            _ => &(),
        };
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
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
        self.tick();
        self.keyboard_service = Some(self.make_keyboard_service());
    }
    fn tick(&mut self) {
        &self.animate();
        &self.render();

        let over = &self.game_over();
        if !over {
            let handle = TimeoutService::spawn(
                Duration::from_millis(TICK_RATE),
                self.link.callback(|_| Msg::Tick),
            );
            self.job = Some(handle);
        } else {
            info!("over");
        }
    }

    fn game_over(&self) -> bool {
        if self.state.snake[0].x.abs() > 460.
            || self.state.snake[0].y.abs() > 460.
            || self.state.snake[0].x < 40.
            || self.state.snake[0].y < 40.
        {
            true
        } else {
            false
        }
    }
    fn animate(&mut self) {
        &self
            .state
            .snake
            .splice(0..0, [self.state.snake[0].add(&self.state.velocity.coords)]);
        &self.state.snake.pop();
    }
    fn render(&mut self) {
        &self.clear();
        if let Some((canvas, ctx)) = &self.ctx {
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
        }
    }
    fn make_keyboard_service(&self) -> Vec<KeyListenerHandle> {
        info!("keyboard service registered!");
        let mut services: Vec<KeyListenerHandle> = Vec::with_capacity(4);
        let handler = KeyboardService::register_key_down(
            &document(),
            self.link.callback(|key: KeyboardEvent| {
                return match &key.key().replace("Arrow", "")[..] {
                    "Left" | "a" => Msg::Left,
                    "Right" | "d" => Msg::Right,
                    "Up" | "w" => Msg::Up,
                    "Down" | "s" => Msg::Down,
                    _ => Msg::None,
                };
            }),
        );
        services.push(handler);
        services
    }
    fn keydown(&mut self, msg: Msg) {
        use Direction::*;

        info!("{:?} {:?}", msg, &self.state.velocity.direction);
        let (x, y, dir) = match (msg, &self.state.velocity.direction) {
            (Msg::Left, Up | Down) => (-20, 0, Left),
            (Msg::Right, Up | Down) => (20, 0, Right),
            (Msg::Up, Left | Right) => (0, -20, Up),
            (Msg::Down, Left | Right) => (0, 20, Down),
            _ => return,
        };
        self.state.velocity.coords = coords(x, y);
        self.state.velocity.direction = dir;
    }
}
