# About
A chess engine, library, UCI, and TUI written in Rust from scratch.
You can play against the latest stable release of Whalecrab here: https://lichess.org/@/Whalecrab

# Interfaces
## Library and Engine
Whalecrab includes its own independant but comprehensive chess library inspired by [chess-rs](https://crates.io/crates/chess-rs). Supporting legal move generation, FEN, move scoring, minimax algorithm, and much more.
## Universal Chess Interface (UCI)
Whalecrab has a basic UCI client, fully compatible with [lichess](https://lichess.org) and any other chess clients or servers that support the UCI protocol. 
## Terminal User Interface (TUI)
Whalecrab comes with a pretty TUI client if you want to play against it locally. The client supports both player-vs-player, and player-vs-engine. The TUI was originally made for debugging and testing Whalecrab before the library was finalized and the UCI client was made, but both clients are still supported today.
