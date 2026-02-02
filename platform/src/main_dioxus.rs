use dioxus::prelude::*;
use go::{GoGame, Stone};

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

    let mut hover_pos: Signal<Option<(usize, usize)>> = use_signal(|| None);

    let on_click = move |evt: Event<MouseData>| {
        let coords = evt.data.element_coordinates();
        let x = ((coords.x - padding) / cell_size).round();
        let y = ((coords.y - padding) / cell_size).round();

        let dx = coords.x - (padding + x * cell_size);
        let dy = coords.y - (padding + y * cell_size);
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < cell_size * 0.45 && x >= 0.0 && y >= 0.0 && x < size as f64 && y < size as f64 {
            let _ = game.write().place_stone(x as usize, y as usize);
        }
    };

    let on_move = move |evt: Event<MouseData>| {
        let coords = evt.data.element_coordinates();
        let x = ((coords.x - padding) / cell_size).round();
        let y = ((coords.y - padding) / cell_size).round();

        let dx = coords.x - (padding + x * cell_size);
        let dy = coords.y - (padding + y * cell_size);
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < cell_size * 0.45 && x >= 0.0 && y >= 0.0 && x < size as f64 && y < size as f64 {
            hover_pos.set(Some((x as usize, y as usize)));
        } else {
            hover_pos.set(None);
        }
    };

    rsx! {
        svg {
            width: "600",
            height: "600",
            view_box: "0 0 600 600",
            class: "go-board-svg",

            onclick: on_click,
            onmousemove: on_move,
            onmouseleave: move |_| hover_pos.set(None),

            // Hintergrund
            rect { x: "0", y: "0", width: "600", height: "600", fill: "#DEB887" }

            // grid
            for i in 0..size {
                line {
                    x1: "{padding}", y1: "{padding + i as f64 * cell_size}",
                    x2: "{board_px - padding}", y2: "{padding + i as f64 * cell_size}",
                    stroke: "black", stroke_width: "1"
                }
                line {
                    x1: "{padding + i as f64 * cell_size}", y1: "{padding}",
                    x2: "{padding + i as f64 * cell_size}", y2: "{board_px - padding}",
                    stroke: "black", stroke_width: "1"
                }
            }

            // punkte
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

            // steine
            {
                let current_game = game.read();
                rsx! {
                    for x in 0..size {
                        for y in 0..size {
                            if let Some(stone) = current_game.get_stone(x, y) {
                                circle {
                                    cx: "{padding + x as f64 * cell_size}",
                                    cy: "{padding + y as f64 * cell_size}",
                                    r: "{cell_size * 0.48}",
                                    fill: if stone == Stone::Black { "black" } else { "white" },
                                    stroke: if stone == Stone::Black { "none" } else { "black" },
                                    stroke_width: "1"
                                }
                            }
                        }
                    }
                }
            }

            // hover-stein
            if let Some((hx, hy)) = hover_pos() {
                if game.read().get_stone(hx, hy).is_none() && !game.read().is_game_over() {
                     circle {
                        cx: "{padding + hx as f64 * cell_size}",
                        cy: "{padding + hy as f64 * cell_size}",
                        r: "{cell_size * 0.48}",
                        fill: if game.read().current_turn() == Stone::Black { "rgba(0,0,0,0.5)" } else { "rgba(255,255,255,0.7)" },
                        stroke: "none"
                    }
                }
            }
        }
    }
}
