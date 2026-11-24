use std::env;
use std::io::{self, Write};

use tic_tac_twist_engine::{AiAlgorithm, AiConfig, Game, GameConfig, Outcome, PlayerKind};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED_BG: &str = "\x1b[41m";

fn ansi_for_player(index: usize) -> &'static str {
    match index {
        0 => "\x1b[1;35m", // magenta
        1 => "\x1b[1;34m", // blue
        2 => "\x1b[1;32m", // green
        _ => "\x1b[1;33m", // yellow
    }
}

fn parse_args() -> Result<GameConfig, String> {
    let mut size: u8 = 5;
    let mut win_length: Option<u8> = None;
    let mut four_corners = false;
    let mut algorithm = AiAlgorithm::Paranoid;
    let mut time_budget_ms: u64 = 120;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!(
                    "tic-tac-twist-cli\n\n\
                     OPTIONS:\n  \
                       --size <3..10>           Board dimension (default: 5)\n  \
                       --win-length <3..size>   Consecutive symbols to win (default: board size)\n  \
                       --four-corners           Enable Four Corners win rule\n  \
                       --ai <paranoid|maxn|greedy>  AI strategy (default: paranoid)\n  \
                       --time-ms <ms>           AI time budget in ms (default: 120)\n  \
                       -h, --help               Show this help"
                );
                std::process::exit(0);
            }
            "--size" => {
                let v = args.next().ok_or("missing value for --size")?;
                size = v.parse().map_err(|_| format!("invalid --size: {v}"))?;
            }
            "--win-length" => {
                let v = args.next().ok_or("missing value for --win-length")?;
                win_length = Some(v.parse().map_err(|_| format!("invalid --win-length: {v}"))?);
            }
            "--four-corners" => four_corners = true,
            "--ai" => {
                let v = args.next().ok_or("missing value for --ai")?;
                algorithm = match v.to_lowercase().as_str() {
                    "paranoid" => AiAlgorithm::Paranoid,
                    "maxn" => AiAlgorithm::MaxN,
                    "greedy" => AiAlgorithm::Greedy,
                    _ => return Err(format!("unknown --ai algorithm: {v}")),
                };
            }
            "--time-ms" => {
                let v = args.next().ok_or("missing value for --time-ms")?;
                time_budget_ms = v.parse().map_err(|_| format!("invalid --time-ms: {v}"))?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let mut cfg = GameConfig::default_classic(size);
    cfg.win_length = win_length.unwrap_or(size);
    cfg.four_corners = four_corners;
    if let Some(first) = cfg.players.first_mut() {
        first.kind = PlayerKind::Ai(AiConfig {
            algorithm,
            time_budget_ms,
            max_depth: 12,
        });
    }

    Ok(cfg)
}

fn render_board(game: &Game) {
    let size = game.board().size() as usize;
    let players = &game.config().players;

    print!("\n    ");
    for c in 1..=size {
        print!("{DIM}{c:^3}{RESET} ");
    }
    println!();

    for r in 0..size {
        print!("{DIM}{:>2}{RESET}  ", r + 1);
        for c in 0..size {
            let idx = (r * size + c) as u8;
            match game.board().cell_owner(idx) {
                Some(p) => {
                    let color = ansi_for_player(p as usize);
                    print!(" {color}{}{RESET} ", players[p as usize].symbol);
                }
                None => print!("   "),
            }
            if c + 1 < size {
                print!("┃");
            }
        }
        println!();
        if r + 1 < size {
            print!("    ");
            for c in 0..size {
                print!("━━━");
                if c + 1 < size {
                    print!("╋");
                }
            }
            println!();
        }
    }
    println!();
}

fn parse_coords(input: &str) -> Option<(u8, u8)> {
    let mut parts = input.split(',');
    let row = parts.next()?.trim().parse::<u8>().ok()?;
    let col = parts.next()?.trim().parse::<u8>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((row, col))
}

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{RED_BG}Error:{RESET} {e}");
            std::process::exit(1);
        }
    };

    let mut game = match Game::new(config) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{RED_BG}Error:{RESET} {e}");
            std::process::exit(1);
        }
    };

    println!(
        "\n{BOLD}Welcome to Tic-Tac-Twist Neo!{RESET} ({size}x{size}, win={win}, four-corners={fc})\n",
        size = game.config().board_size,
        win = game.config().win_length,
        fc = game.config().four_corners
    );

    for (idx, p) in game.config().players.iter().enumerate() {
        let color = ansi_for_player(idx);
        let tag = match &p.kind {
            PlayerKind::Human => "Human".to_string(),
            PlayerKind::Ai(ai) => format!("AI:{:?}", ai.algorithm),
        };
        println!("  Player {}: {color}{} ({}){RESET} [{tag}]", idx + 1, p.name, p.symbol);
    }
    println!("\n{DIM}Enter moves as row,col (e.g. 1,3), 'undo', or 'quit'.{RESET}");

    render_board(&game);

    let stdin = io::stdin();
    let mut input = String::new();

    while !game.is_over() {
        let turn = game.current_turn() as usize;
        let player = game.current_player().clone();
        let color = ansi_for_player(turn);

        match player.kind {
            PlayerKind::Ai(_) => match game.play_ai_turn() {
                Ok(res) => {
                    let (r, c) = game.board().index_to_coord(res.best_move).unwrap();
                    let t = &res.telemetry;
                    println!(
                        "{color}{} (AI){RESET} played {BOLD}{r},{c}{RESET}  {DIM}[{:?} | d={} | {} nodes | {:.1}ms | {} n/s]{RESET}",
                        player.name,
                        t.algorithm,
                        t.depth_reached,
                        t.nodes_evaluated,
                        t.elapsed_ms,
                        t.nodes_per_sec
                    );
                    render_board(&game);
                }
                Err(e) => {
                    eprintln!("{RED_BG}AI Error:{RESET} {e}");
                    break;
                }
            },
            PlayerKind::Human => {
                print!("{color}{} >{RESET} ", player.name);
                let _ = io::stdout().flush();

                input.clear();
                if stdin.read_line(&mut input).unwrap_or(0) == 0 {
                    break;
                }
                let trimmed = input.trim();

                if trimmed.eq_ignore_ascii_case("quit") || trimmed.eq_ignore_ascii_case("exit") {
                    println!("Goodbye!");
                    return;
                }
                if trimmed.eq_ignore_ascii_case("undo") {
                    if let Err(e) = game.undo() {
                        println!("{RED_BG}{e}{RESET}");
                    } else {
                        render_board(&game);
                    }
                    continue;
                }

                let Some((r, c)) = parse_coords(trimmed) else {
                    println!("{RED_BG}Wrong input! Use row,col (e.g. 2,3).{RESET}");
                    continue;
                };

                match game
                    .board()
                    .coord_to_index(r, c)
                    .and_then(|idx| game.play_move(idx).map(|_| ()))
                {
                    Ok(()) => render_board(&game),
                    Err(e) => println!("{RED_BG}Invalid move: {e}{RESET}"),
                }
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match game.outcome() {
        Outcome::Win { player, kind, .. } => {
            let p = &game.config().players[*player as usize];
            let color = ansi_for_player(*player as usize);
            println!("  Winner: {color}{} {}{RESET} ({kind:?})", p.name, p.symbol);
        }
        Outcome::Draw => println!("  It's a draw!"),
        Outcome::Ongoing => println!("  Game ended early."),
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}
