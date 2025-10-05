use rand::Rng;
use whalecrab::game::Game;

fn main() {
    let mut game = Game::default();
    let mut rng = rand::rng();

    for _ in 0..10 {
        let moves = game.generate_all_legal_moves();
        let chosen_move = rng.random_range(0..moves.len());
        let m = moves.get(chosen_move).expect("Chose invalid move");
        println!("Chose to play: {}", m);
        game.play(m);
    }

    println!("=========================");
    println!("Final score: {}", game.grade_position());
    println!("Final fen: {}", game.position.to_fen());
}
