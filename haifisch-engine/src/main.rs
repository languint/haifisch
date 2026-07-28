mod eval;
mod search;
mod uci;

use std::io::{self, BufRead};

use uci::Engine;

fn main() {
    let mut engine = Engine::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };
        if !engine.handle(&line, &mut stdout) {
            break;
        }
    }
}
