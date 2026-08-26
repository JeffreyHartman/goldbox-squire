//! Reads a save folder through the real pipeline and prints what it holds.
//!
//! This is the table-verification tool: point it at any save folder and it
//! decodes exactly the records a live session's save readers accept, via
//! [`saves::slot_party_records`]. Nothing here re-derives a path or a walk,
//! so this tool cannot drift from what the tool it verifies actually does.
//!
//!     cargo run --example read_saves -- <game-id> <save-folder>

use squire_core::{games, record, saves};

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(id), Some(dir)) = (args.next(), args.next()) else {
        eprintln!("usage: read_saves <game-id> <save-folder>");
        std::process::exit(2);
    };
    let Some(game) = games::find(&id) else {
        let known: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
        eprintln!(
            "unknown game `{id}`. Compiled-in games: {}",
            known.join(", ")
        );
        std::process::exit(2);
    };

    let slots = match saves::populated_slots(&game, &dir) {
        Ok(slots) => slots,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    for slot in slots {
        println!("slot {}: {}", slot.letter, slot.names.join(", "));
        let records = match saves::slot_party_records(&game, &dir, slot.letter) {
            Ok(records) => records,
            Err(e) => {
                println!("  {e}");
                continue;
            }
        };
        for (name, bytes) in records {
            describe(&game, &name, &bytes);
        }
    }
}

fn describe(game: &games::Game, name: &str, bytes: &[u8]) {
    match record::validate(&game.table, bytes).and_then(|_| record::decode(&game.table, bytes)) {
        Ok(c) => println!(
            "  {name}: {} {} {} level {}, {}/{} hp, ac {}, thac0 {}, {} xp, age {}, {}",
            c.gender.as_deref().unwrap_or("?"),
            c.race.as_deref().unwrap_or("?"),
            c.class.as_deref().unwrap_or("?"),
            c.level,
            c.hit_points_current,
            c.hit_points_maximum,
            c.armor_class,
            c.thac0,
            c.experience,
            c.age,
            c.status.as_deref().unwrap_or("?"),
        ),
        Err(e) => println!("  {name}: DOES NOT VALIDATE: {e}"),
    }
}
