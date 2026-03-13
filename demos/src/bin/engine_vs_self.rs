use whalecrab_engine::engine::Engine;

fn main() {
    let mut engine = Engine::default();

    for _ in 0..100 {
        let m = engine.minimax(2).unwrap();
        println!("Chose to play: {}", m);
        engine.game.play(&m);
    }

    println!("=========================");
    println!("Final score: {}", engine.grade_position());
    println!("Final fen: {}", engine.game.to_fen());
}
