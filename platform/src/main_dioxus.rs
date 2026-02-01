use dioxus::prelude::*;

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
    let game_name = match game_type {
        GameType::Chess => "Schach",
        GameType::Go => "Go",
        GameType::Kniffel => "Kniffel",
        GameType::Minesweeper => "Minesweeper",
    };

    rsx! {
        div { class: "game-view",
            button { class: "back-button", onclick: move |_| on_back.call(()), "← Zurück" }
            h2 { "{game_name}" }
            p { class: "placeholder", "Game ui ..." }
        }
    }
}
