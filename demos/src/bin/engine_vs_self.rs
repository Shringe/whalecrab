use whalecrab_lib::game::Game;

fn main() {
    let mut game = Game::default();

    for _ in 0..100 {
        let m = game.get_engine_move_minimax(2).unwrap();
        println!("Chose to play: {}", m);
        game.play(&m);
    }

    println!("=========================");
    println!("Final score: {}", game.grade_position());
    println!("Final fen: {}", game.position.to_fen());
}
