# Spielesammlung
Eine Sammlung von bekannten Spielen, zu 100% in Rust geschrieben. 
Spielbar sind folgende Spiele: Schach, Go, Kniffel, Minesweeper.

Die Spiele Schach, Go und Kniffel bieten Bot-Gegner 
und einen Mehrspielermodus. 

### Voraussetzungen

### Wie man das Projekt ausführt
Das Projekt wird gestartet durch die Eingabe von:

``
cargo run -p platform 
``

### Starten des Multiplayer-Servers

## Features

### 1. Hauptmenü
Das Hauptmenü bietet die Möglichkeit, zwischen einem Light- und Dark-Mode zu wählen,
sowie über 4 Knöpfe das darauf abgebildete Spiel auszuwählen.

### 2. Spielmodus-Auswahl
Entscheidet man sich für eines der folgenden 3 Spiele – Schach, Go oder Kniffel –,
wird man nach dem Hauptmenü zur Auswahl des Spielmodus weitergeleitet.
Hier kann zwischen einem lokalen Spiel, dem Spiel gegen einen Bot
oder dem Mehrspielermodus entschieden werden.
(Mehr zur Nutzung des Mehrspielermodus folgt im nächsten Punkt.)
Die Auswahl eines oder mehrerer Bots erfolgt für das Kniffel-Spiel nach der Auswahl eines lokalen Spieles.
Dort stehen daher nur 2 Knöpfe zur Verfügung.

### 3. Mehrspieler

### 4. Schach

### 5. Go

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



