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

Für jeden Spielmodus gilt Folgendes:
Wenn man eine Figur bewegen möchte, muss man diese anklicken und daraufhin werden alle legalen Bewegungsmöglichkeiten grün markiert. Die Farben dürfen abwechselnd jeweils einen Zug machen, Weiß fängt an. Der zuletzt gemachte Zug wird gelb markiert. Es ist jederzeit möglich, mit dem "Reset Game" Button das Schachbrett zurückzusetzen oder mit dem Button "Zurück zum Menü" zum Hauptmenü zurückzukehren.
Die normalen Bewegungsmöglichkeiten aus Schach sind implementiert, inklusive En Passant und Roschade. Es sind zudem nur legale Züge möglich, somit ist es zum Beispiel nicht erlaubt, wenn der weiße König im Schach ist, eine weiße Figur so zu bewegen, dass es nicht verhindert,dass der König weiterhin im Schach ist. Ein anderes Beispiel wäre, dass sich ein schwarzer Bauer nicht bewegen kann, wenn durch seinen Spielzug der schwarzer König im Schach wäre. Wenn man mit einem Bauern ans Ende des Spielfeldes kommt, kann man in einem kleinen, grauen Bereich unterhalb des Spielfelds auswählen, mit welcher Art von Figur man den Bauern gerne austauschen würde. Nachdem man diese gewählt (angeklickt) hat, mutiert der Bauer zu dieser Figur der gleichen Farbe.
Nach der Auswahl des Spielmodus wird zunächst eine Information über die Wahl in Grün angezeigt und sobald ein Zug gemacht wird, steht an dieser Stelle die "Current Evaluation". Diese beschreibt, welche Farbe gerade im Vorteil ist, hierbei sind positive Werte gut für Weiß und negative Werte gut für Schwarz.
Das Spiel ist vorbei, wenn eine Farbe Matt gesetzt wurde oder es zu einem Unentschieden kam. Bei beiden Möglichkeiten steht nun an der Stelle, wo "Current Evaluation" war, ein roter Text, der anzeigt, welche Farbe das Spiel gewonnen hat oder der anzeigt, dass es sich um ein Unentschieden handelt. Hier kommt es zu einem Unentschieden, wenn sich eine Farbe gar nicht mehr bewegen kann oder wenn es zu einer dreifachen Stellungswiederholung gekommen ist. Daraufhin kann man das Spiel mit dem Reset Button zurücksetzen und eine neue Partie starten.
Bei einem lokalen Spiel kann man selbst, oder mit jemand anderem alle Figuren für beide Farben bewegen.
Bei einem Spiel gegen den Bot kann man mit einem Slider (der sich oben rechts bei dem Fenster befindet) das Level des Bots einstellen. Hier ist Level 1 bis 4 möglich. Wenn man dann auf den Button "Spiele gegen einen Bot (Level n)" klickt, kommt man als Weiß in ein Schachfenster und kann gegen den Bot spielen. Bei einem höheren Level ist mit mehr Wartezeit zu rechnen, bis der Bot einen Zug macht, da beim Berechnen der Züge mit einer höheren Rekursionstiefe gearbeitet wird. Allgemein basiert der Bot auf einem Min-Max-Algorithmus mit Alpha-Beta Pruning und einem Quiescence Search Algorithmus zur Erweiterung des Min-Max.
Um den Raum zu erstellen und ihm beizutreten, siehe 3.1. Clients und Anwendung. Wenn der Spielbildschirm nun für beide Spieler geöffnet ist, sieht man für das Mulitplayerspiel eine kleine Information darüber, welche Farbe man ist und wer nun den nächsten Zug machen darf. Wenn ein Bauer am Ende des Spielfeldes angekommen ist, kann der Spieler einfach auswählen, welche Figur er gerne haben möchte und dann wird bei dem anderen Spieler diese Figur auch sofort gesetzt. Wenn man das Spiel neustarten möchte, kann man das hier weiterhin über den Reset Button.

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

Bei der Implementierung des MCTS habe ich mich an diesen Projekten von [Mamy Ratsimbazafy](https://github.com/mratsim/rustygo) und [Fioelkais](https://github.com/Fioelkais/MCTS) orientiert, um den grundlegenden MCTS-Algorithmus mit UCT zu verstehen. Die eigentliche Implementierung unterscheidet sich jedoch deutlich: kontinuierliches Sigmoid-Scoring statt binärem Gewinn/Verlust, heuristische Simulation mit Atari-Erkennung statt reiner Zufalls-Playouts, und eine zeitbasierte Suchsteuerung.

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

Bei Kniffel kann man ein lokales oder ein online multiplayer Kniffelspiel starten.
Der online Multiplayer untersützt zwei Spieler, während der lokale Modus bis zu vier
Spielererlaubt. Dort können auch bis zu drei Botgegner antreten.

Durch Drücken des Würfelbuttons wird gewürfelt und die entsprechende Spalte des Spielers
in der Punktetabelle freigeschaltet um die Punkte einzutragen.
Durch Hovern über den Kategorien kann man einsehen was die Anforderungen sind und wie viele
Punkte sie bringen.
Der Zug wird durch das eintragen der Punkte beendet und der nächste Spieler ist dran.

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
