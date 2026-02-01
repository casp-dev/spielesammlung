use dioxus::prelude::*;
use go::GoGame;

#[derive(Clone, PartialEq)]
enum GameType {
    Chess,
    Go,
    Kniffel,
    Minesweeper,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut current_game: Signal<Option<GameType>> = use_signal(|| None);

    rsx! {
        style { {include_str!("styles.css")} }
        div { class: "app",
            match current_game() {
                None => rsx! { Menu { on_select: move |game| current_game.set(Some(game)) } },
                Some(game) => rsx! {
                    GameView {
                        game_type: game,
                        on_back: move |_| current_game.set(None)
                    }
                }
            }
        }
    }
}

#[component]
fn Menu(on_select: EventHandler<GameType>) -> Element {
    rsx! {
        div { class: "menu",
            h1 { "Spielesammlung" }
            p { "Wähle ein Spiel:" }
            div { class: "game-buttons",
                button { onclick: move |_| on_select.call(GameType::Chess), "Schach" }
                button { onclick: move |_| on_select.call(GameType::Go), "Go" }
                button { onclick: move |_| on_select.call(GameType::Kniffel), "Kniffel" }
                button { onclick: move |_| on_select.call(GameType::Minesweeper), "Minesweeper" }
            }
        }
    }
}

#[component]
fn GameView(game_type: GameType, on_back: EventHandler<()>) -> Element {
    match game_type {
        GameType::Go => rsx! { GoGameScreen { on_back } },
        other => {
            let game_name = match other {
                GameType::Chess => "Schach",
                GameType::Kniffel => "Kniffel",
                GameType::Minesweeper => "Minesweeper",
                _ => unreachable!(),
            };
            rsx! {
                div { class: "game-view",
                    button { class: "back-button", onclick: move |_| on_back.call(()), "← Zurück" }
                    h2 { "{game_name}" }
                    p { class: "placeholder", "Game ui ..." }
                }
            }
        }
    }
}

#[component]
fn GoGameScreen(on_back: EventHandler<()>) -> Element {
    let game = use_signal(|| GoGame::new());

    rsx! {
        div { class: "game-view",
            div { class: "header",
                button { class: "back-button", onclick: move |_| on_back.call(()), "← Zurück" }
                h1 { "Go" }
            }

            div { class: "go-container",
                GoBoard { game: game }

                div { class: "info-panel",
                    p { "Status: {game.read().status_message}" }
                }
            }
        }
    }
}

#[component]
fn GoBoard(game: Signal<GoGame>) -> Element {
    let size = game.read().board_size();
    let board_px = 600.0;
    let padding = 30.0;
    let effective_width = board_px - 2.0 * padding;
    let cell_size = effective_width / (size as f64 - 1.0);

    rsx! {
        svg {
            width: "600",
            height: "600",
            view_box: "0 0 600 600",
            class: "go-board-svg",

            // Hintergrund
            rect { x: "0", y: "0", width: "600", height: "600", fill: "#DEB887" }

            // grid
            for i in 0..size {
                // horizontal
                line {
                    x1: "{padding}",
                    y1: "{padding + i as f64 * cell_size}",
                    x2: "{board_px - padding}",
                    y2: "{padding + i as f64 * cell_size}",
                    stroke: "black",
                    stroke_width: "1"
                }
                // vertikal
                line {
                    x1: "{padding + i as f64 * cell_size}",
                    y1: "{padding}",
                    x2: "{padding + i as f64 * cell_size}",
                    y2: "{board_px - padding}",
                    stroke: "black",
                    stroke_width: "1"
                }
            }

            // Punkte
            if size == 19 {
                 {
                    let stars = vec![3, 9, 15];
                    rsx! {
                        for x in stars.iter() {
                            for y in stars.iter() {
                                circle {
                                    cx: "{padding + *x as f64 * cell_size}",
                                    cy: "{padding + *y as f64 * cell_size}",
                                    r: "4",
                                    fill: "black"
                                }
                            }
                        }
                    }
                 }
            }
        }
    }
}
