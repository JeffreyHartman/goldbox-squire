//! Reads a save folder through the real pipeline and prints what it holds.
//!
//! This is the table-verification tool: point it at any save folder and it
//! runs the same save readers, validation and decoding a live session does.
//!
//!     cargo run --example read_saves -- <game-id> <save-folder>

use squire_core::{games, saves};

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(id), Some(dir)) = (args.next(), args.next()) else {
        eprintln!("usage: read_saves <game-id> <save-folder>");
        std::process::exit(2);
    };
    let Some(game) = games::find(&id) else {
        let known: Vec<String> = games::games().into_iter().map(|g| g.id).collect();
        eprintln!("unknown game `{id}`. Compiled-in games: {}", known.join(", "));
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
        // The names come from the save files; decoding the full record is the
        // real test of the table, so decode every character too.
        match game.saves.shape {
            games::SaveShape::Chrdat => {
                for (index, name) in slot.names.iter().enumerate() {
                    let file = format!(
                        "{dir}/CHRDAT{}{}.{}",
                        slot.letter,
                        index + 1,
                        game.saves.extension
                    );
                    let Ok(bytes) = std::fs::read(&file) else {
                        println!("  {name}: cannot read {file}");
                        continue;
                    };
                    describe(&game, name, &bytes);
                }
            }
            games::SaveShape::PartyFile => {
                let file = format!("{dir}/SAVGAM{}.{}", slot.letter, game.saves.extension);
                let Ok(bytes) = std::fs::read(&file) else {
                    println!("  cannot read {file}");
                    continue;
                };
                // Walk the file the way the reader does, printing each record.
                let mut pos = game.saves.first_record_offset.unwrap_or(0);
                let mut seen = 0;
                while seen < slot.names.len() && pos + game.table.record_len <= bytes.len() {
                    let candidate = &bytes[pos..pos + game.table.record_len];
                    if squire_core::record::validate(&game.table, candidate).is_ok() {
                        let name = squire_core::record::name_at(&game.table, candidate)
                            .unwrap_or_default();
                        describe(&game, &name, candidate);
                        pos += game.table.record_len;
                        seen += 1;
                    } else {
                        pos += 1;
                    }
                }
            }
        }
    }
}

fn describe(game: &games::Game, name: &str, bytes: &[u8]) {
    match squire_core::record::validate(&game.table, bytes)
        .and_then(|_| squire_core::record::decode(&game.table, bytes))
    {
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
