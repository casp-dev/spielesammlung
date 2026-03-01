# Spielesammlung

Eine Sammlung von bekannten Spielen, zu 100% in Rust geschrieben.
Spielbar sind folgende Spiele: Schach, Go, Kniffel, Minesweeper.

Die Spiele Schach, Go und Kniffel bieten Bot-Gegner
und einen Mehrspielermodus.

### Voraussetzungen

- **Rust & Cargo** – Eine aktuelle Rust-Toolchain wird benötigt (empfohlen: [rustup](https://rustup.rs/)).
- Für die GUI wird [egui / eframe](https://github.com/emilk/egui) verwendet; die nötigen Abhängigkeiten werden automatisch über Cargo aufgelöst.
- Für den Mehrspieler-Modus muss der WebSocket-Server erreichbar sein (siehe _Starten des Multiplayer-Servers_).

### Wie man das Projekt ausführt

Das Projekt wird gestartet durch die Eingabe von:

```
cargo run -p platform
```

### Starten des Multiplayer-Servers

Der Multiplayer-Server wird über ein eigenes Crate bereitgestellt und kann wie folgt gestartet werden:

```
cargo run -p server
```

Der Server läuft anschließend auf `0.0.0.0:9000` und akzeptiert WebSocket-Verbindungen.
Die Clients verbinden sich standardmäßig mit `ws://localhost:9000`.

## Projektstruktur

```
spielesammlung/
├── platform/         # Hauptanwendung – startet das Fenster und das Hauptmenü
├── game_core/        # Gemeinsame Spiellogik (Traits, Multiplayer-UI, WebSocket-Client)
├── games/
│   ├── chess/        # Schach-Implementierung inkl. Bot (Min-Max / Alpha-Beta)
│   ├── go/           # Go-Implementierung inkl. Bot (MCTS)
│   ├── kniffel/      # Kniffel-Implementierung inkl. Bots
│   └── minesweeper/  # Minesweeper-Implementierung
└── server/           # WebSocket-Multiplayer-Server (Tokio + Tungstenite)
```

## Features

### 1. Hauptmenü

Das Hauptmenü bietet die Möglichkeit, zwischen einem Light- und Dark-Mode zu wählen,
sowie über 4 Buttons das darauf abgebildete Spiel auszuwählen.

### 2. Spielmodus-Auswahl

Entscheidet man sich für eines der folgenden 3 Spiele – Schach, Go oder Kniffel –,
wird man nach dem Hauptmenü zur Auswahl des Spielmodus weitergeleitet.
Hier kann zwischen einem lokalen Spiel, dem Spiel gegen einen Bot
oder dem Mehrspielermodus entschieden werden.
Die Auswahl eines oder mehrerer Bots erfolgt für das Kniffel-Spiel nach der Auswahl eines lokalen Spieles.
Dort stehen daher nur 2 Buttons zur Verfügung.

### 3. Mehrspieler

#### 3.1. Server

Der Multiplayer-Server ist ein eigenständiges Crate auf Basis von **Tokio** und **Tungstenite** (WebSocket).
Er verwaltet Räume, in denen jeweils maximal **2 Spieler** gleichzeitig spielen können.

Ablauf auf Server-Seite:

1. Ein Spieler sendet `CreateRoom` → der Server erstellt einen neuen Raum und gibt eine **Room-ID** zurück.
2. Ein zweiter Spieler sendet `JoinRoom` mit der Room-ID → der Server ordnet ihn dem Raum zu und benachrichtigt beide Spieler (`PlayerJoined`).
3. Spielzüge (`GameMove`) werden vom Server an den jeweils anderen Spieler im selben Raum weitergeleitet.
4. Verlässt ein Spieler den Raum oder trennt die Verbindung, werden die verbleibenden Spieler benachrichtigt (`PlayerLeft`).

Der Server wird gestartet mit:

```
cargo run -p server
```

#### 3.2. Clients und Anwendung

Bei den Spielen Kniffel, Go und Schach kann man, wenn der Server an und verfügbar ist, einen Raum erstellen und sich mit dem Raum verbinden.
Wenn man eines der drei Spiele auswählt, hat man die Möglichkeit, über den Button "Mehrspieler Raum erstellen" einen Raum zu erstellen. Nun wird man, wenn eine Verbindung zum Server erstellt wurde, in einen Warteraum weitergeleitet. Dort kann man eine Room ID kopieren und diese an die Mitspieler senden. Oben links können nun die anderen Spieler die Room ID einfügen und dann auf den Button "Beitreten" gehen, daraufhin wird das Spiel gestartet und alle werden auf den Spiel-Bildschirm weitergeleitet.

### 4. Schach

Das Schachspiel unterstützt alle klassischen Regeln (inklusive **En Passant** und **Rochade**). Es wird strikt auf legale Züge geachtet (z. B. können gefesselte Figuren nicht bewegt werden, wenn dadurch der eigene König ins Schach gesetzt würde).

- **Zugmechanik**: Ein Klick auf eine Figur markiert alle legalen Züge dieser Figur **grün**. Der zuletzt gespielte Zug wird auf dem Brett **gelb** hervorgehoben.
- **Bauernumwandlung (Promotion)**: Erreicht ein Bauer die gegnerische Grundreihe, erscheint unterhalb des Brettes ein kleiner grauer Bereich zur Auswahl der neuen Figur.
- **Stellungsbewertung & Status**: Oberhalb des Bretts wird fortlaufend die "Current Evaluation" angezeigt (positive Werte = Weiß im Vorteil, negative Werte = Schwarz im Vorteil). Bei Matt, Patt oder dreifacher Stellungswiederholung wird hier das Endergebnis (Sieger/Unentschieden) in roter Schrift angezeigt.
- **Steuerung**: Über die Buttons "Reset Game" und "Zurück zum Menü" lässt sich eine Partie jederzeit neu starten oder verlassen.

#### Bot-Modus

Der Schach-Bot basiert auf einem **Min-Max-Algorithmus mit Alpha-Beta Pruning** und **Quiescence Search**.

- **Level-Auswahl**: Oben rechts im Menü befindet sich ein **Slider**, mit dem das Bot-Level (1-4) gewählt wird. Höhere Level erhöhen die Rekursionstiefe und damit die Bedenkzeit des Bots.

#### Mehrspieler-Modus

Im Online-Spiel (siehe [3.1. Server](#31-server)) wird jedem Spieler die zugewiesene Farbe sowie der aktuelle Spieler am Zug als Statusnachricht angezeigt. Züge und spezielle Aktionen wie die Bauernumwandlung werden dabei in Echtzeit synchronisiert.

### 5. Go

Das Go-Spiel wird auf einem **19×19-Brett** gespielt. Die Regeln folgen dem klassischen Go-Regelwerk inklusive:

- **Setzen von Steinen**: Schwarz beginnt. Durch Klicken auf einen Schnittpunkt wird ein Stein der eigenen Farbe platziert.
- **Schlagen**: Gegnerische Gruppen ohne Freiheiten werden automatisch vom Brett entfernt. Die Anzahl geschlagener Steine wird rechts in der Seitenleiste angezeigt.
- **Ko-Regel**: Eine Stellungswiederholung wird erkannt und verhindert.
- **Selbstmord-Regel**: Züge, die den eigenen Stein sofort töten würden (ohne gegnerische Steine zu schlagen), sind nicht erlaubt.
- **Passen**: Über den „Passen"-Button kann ein Spieler seinen Zug aussetzen. Passen beide Spieler nacheinander, endet das Spiel.
- **Punktzahl**: Am Spielende wird die Punktzahl nach chinesischer Zählung berechnet (Steine auf dem Brett + umschlossenes Gebiet); Weiß erhält 6,5 Komi-Punkte.
- **Koordinaten**: Über die Checkbox „Koordinaten" in der Seitenleiste können Spalten- und Zeilenbeschriftungen ein- und ausgeblendet werden.
- **Neustart**: Das Spiel kann jederzeit über den „Neustarten"-Button zurückgesetzt werden.

#### Bot-Modus

Der Go-Bot basiert auf einem **Monte Carlo Tree Search (MCTS)**-Algorithmus. Er nutzt:

- UCT-Formel (Upper Confidence Bound for Trees) zur Knotenauswahl
- Heuristische Zugvorauswahl (Atari-Züge, Nachbarzüge, u. a.)
- Zufällige Rollouts mit intelligentem Atari-Handling zur Bewertung

Nach jedem Bot-Zug wird angezeigt, welchen Zug er gewählt hat, wie viele MCTS-Iterationen durchgeführt wurden und welche Top-Züge bewertet wurden.

#### Multiplayer-Modus

Im Mehrspieler-Modus wird dem Spieler oben rechts angezeigt, welche Steinfarbe er hat und ob er am Zug ist. Züge, Passen und Neustarts werden automatisch an den Gegner übertragen. Verlässt der Gegner das Spiel, wird man automatisch zurück ins Menü geleitet.

### 6. Minesweeper

Zuerst hat man die Wahl zwischen 4 Schwierigkeitsgraden:

- Einfach
- Mittel
- Schwer
- Experte

Nach der Auswahl des Schwierigkeitsgrades beginnt das Spiel.
Der Spieler kann:

- Mit Linksklick auf ein ungeöffnetes Feld dieses öffnen
- Linksklick auf ein geöffnetes Feld mit Zahl halten, um die Nachbarn anzeigen zu lassen
- Mit Rechtsklick auf ein ungeöffnetes Feld eine Flagge setzen (markieren)
- Mit Rechtsklick auf ein markiertes Feld die Flagge wieder entfernen

Während des Spiels werden die noch verfügbaren Flaggen sowie die sich auf dem Spielfeld
befindenden Bomben oben links im Eck angezeigt. Zudem kann der Spieler jederzeit:

- Zum Hauptmenü zurückkehren und ein anderes Spiel wählen
- Zurück zur Auswahl des Schwierigkeitsgrades
  In beiden Fällen wird der Spielstand NICHT gespeichert.

Gewinnt oder verliert man das Spiel, öffnet sich ein Fenster mit folgenden Optionen:

- Spielfeld anzeigen, um besser nachzuvollziehen, wieso man verloren/gewonnen hat
- Eine andere Schwierigkeit wählen und noch einmal spielen
- Oder es auf der gleichen Schwierigkeitsstufe noch einmal versuchen

### 7. Kniffel

Das Kniffel-Spiel lässt sich wahlweise im lokalen Modus oder online spielen. Die Steuerung und das Punktesystem orientieren sich streng an den bekannten Kniffel-Regeln.

- **Spielmodi & Teilnehmer**:
  - Der Online-Multiplayer (siehe [3.1. Server](#31-server)) unterstützt **zwei Spieler**.
  - Der lokale Modus erlaubt bis zu **vier Spieler**. Hier können zusätzlich bis zu **drei Bot-Gegner** für die Partie eingestellt werden.
- **Spielablauf**: Ein Klick auf den Würfelbutton startet den Wurf. Nach dem Würfeln wird die zugehörige Spalte des aktiven Spielers in der grafischen Punktetabelle freigeschaltet, um ein Ergebnis direkt dort einzutragen.
- **Hilfestellung**: Wenn man mit der Maus über die jeweiligen Kategorien in der Punktetabelle fährt (Hover-Effekt), erfährt man genau, welche Würfelkombination erforderlich ist und wie sich die Punkte zusammensetzen.
- **Zugwechsel**: Sobald ein Spieler sein Würfelergebnis in eine Kategorie eingetragen hat, wird sein Zug offiziell beendet und der nächste Spieler (bzw. Bot) ist an der Reihe.

## Testen

Die Anwendung wurde überwiegend **manuell** getestet. Für kritische Spiellogik (z. B. Schlagregeln und Ko-Regel im Go) existieren ergänzende **Unit-Tests** (`cargo test`), die korrekte Züge, illegale Züge und Spielende-Bedingungen überprüfen.

Der Schwerpunkt auf manuelles Testen wurde bewusst gewählt, da er für dieses Projekt mehrere Vorteile bietet:

- **Flexibilität bei komplexen Spielszenarien**: Brettspiele erzeugen eine enorme Vielfalt an Stellungen. Durch manuelles Durchspielen konnten gezielt Randfälle getestet werden (z. B. Rochade nach vorherigem König-Zug, En Passant, Bauernumwandlung, Ko-Situationen im Go, Kniffel-Sonderfälle), die in automatisierten Tests nur schwer vollständig abzubilden wären.
- **UI- und Interaktionstests**: Die korrekte Darstellung des Spielbretts, die Hervorhebung legaler Züge, Animationen und die Responsivität der Oberfläche lassen sich am effektivsten visuell beurteilen.
- **Mehrspieler-Kommunikation**: Das Zusammenspiel von Server und mehreren Clients (Raum erstellen, beitreten, Züge synchronisieren, Disconnect-Handling) wurde durch paralleles Starten mehrerer Instanzen manuell überprüft.

## Technologien

| Bereich            | Technologie                                        |
| ------------------ | -------------------------------------------------- |
| Sprache            | Rust                                               |
| GUI-Framework      | egui / eframe 0.29.1                               |
| WebSocket (Server) | Tokio + Tungstenite                                |
| WebSocket (Client) | Tungstenite                                        |
| Serialisierung     | serde / serde_json                                 |
| Schach-Bot         | Min-Max mit Alpha-Beta Pruning + Quiescence Search |
| Go-Bot             | Monte Carlo Tree Search (MCTS)                     |
