use whalecrab_lib::game::Game;

fn main() {
    let mut game = Game::default();

    for _ in 0..100 {
        let m = game.find_best_move().unwrap().0;
        println!("Chose to play: {}", m);
        game.play(&m);
    }

    println!("=========================");
    println!("Final score: {}", game.grade_position());
    println!("Final fen: {}", game.position.to_fen());
}
