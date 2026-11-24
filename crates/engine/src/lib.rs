pub mod ai;
pub mod board;
pub mod error;
pub mod game;
pub mod rules;

pub use ai::{AiAlgorithm, AiConfig, SearchResult, SearchTelemetry, choose_move};
pub use board::{Board, MAX_BOARD_SIZE, MAX_PLAYERS, MIN_BOARD_SIZE};
pub use error::EngineError;
pub use game::{Game, GameConfig, GameSnapshot, MoveRecord, Outcome, PlayerConfig, PlayerKind};
pub use rules::{FourCornersRule, LinesRule, RuleSet, WinKind, WinMask, WinRule};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_validates_sizes_and_bounds() {
        assert!(Board::new(2).is_err());
        assert!(Board::new(11).is_err());
        let mut board = Board::new(3).unwrap();
        assert_eq!(board.cell_count(), 9);
        assert!(board.place(0, 9).is_err()); // fixes 2019 off-by-one bug
        assert!(board.place(0, 0).is_ok());
        assert!(board.place(1, 0).is_err()); // occupied
    }

    #[test]
    fn coordinate_conversion_is_exact_roundtrip() {
        for size in 3..=10 {
            let board = Board::new(size).unwrap();
            for idx in 0..board.cell_count() {
                let (r, c) = board.index_to_coord(idx).unwrap();
                assert_eq!(board.coord_to_index(r, c).unwrap(), idx);
            }
            assert!(board.coord_to_index(0, 1).is_err());
            assert!(board.coord_to_index(size + 1, 1).is_err());
        }
    }

    #[test]
    fn four_corners_respects_actual_board_size() {
        for size in [3u8, 5, 10] {
            let mut board = Board::new(size).unwrap();
            let rules = RuleSet::build(size, size, true).unwrap();
            let corners = [0, size - 1, size * (size - 1), size * size - 1];
            for c in corners {
                board.place(0, c).unwrap();
            }
            let win = rules.check_winner(&board, 0).expect("four corners win");
            assert_eq!(win.kind, WinKind::FourCorners);
            assert_eq!(win.cells, corners);
        }
    }

    #[test]
    fn ai_takes_immediate_win_across_all_algorithms() {
        for algo in [AiAlgorithm::Paranoid, AiAlgorithm::MaxN, AiAlgorithm::Greedy] {
            let mut board = Board::new(3).unwrap();
            let rules = RuleSet::build(3, 3, false).unwrap();
            // AI (player 0) has top-left and top-middle -> must take cell 2.
            board.place(0, 0).unwrap();
            board.place(0, 1).unwrap();
            board.place(1, 3).unwrap();
            board.place(2, 4).unwrap();

            let before = board.clone();
            let res = choose_move(
                &board,
                &rules,
                0,
                3,
                &AiConfig {
                    algorithm: algo,
                    time_budget_ms: 80,
                    max_depth: 6,
                },
            )
            .unwrap();

            assert_eq!(res.best_move, 2, "failed for {algo:?}");
            assert_eq!(board, before, "AI must never mutate board state");
        }
    }

    #[test]
    fn ai_blocks_both_human_opponents() {
        for opponent in [1u8, 2u8] {
            for algo in [AiAlgorithm::Paranoid, AiAlgorithm::MaxN, AiAlgorithm::Greedy] {
                let mut board = Board::new(3).unwrap();
                let rules = RuleSet::build(3, 3, false).unwrap();
                board.place(0, 0).unwrap();
                board.place(opponent, 6).unwrap();
                board.place(opponent, 7).unwrap();

                let res = choose_move(
                    &board,
                    &rules,
                    0,
                    3,
                    &AiConfig {
                        algorithm: algo,
                        time_budget_ms: 80,
                        max_depth: 6,
                    },
                )
                .unwrap();

                assert_eq!(res.best_move, 8, "failed to block player {opponent} with {algo:?}");
            }
        }
    }

    #[test]
    fn ai_handles_10x10_within_time_budget() {
        let mut board = Board::new(10).unwrap();
        let rules = RuleSet::build(10, 4, true).unwrap();
        board.place(0, 44).unwrap();
        board.place(1, 45).unwrap();
        board.place(2, 54).unwrap();

        let res = choose_move(
            &board,
            &rules,
            0,
            3,
            &AiConfig {
                algorithm: AiAlgorithm::Paranoid,
                time_budget_ms: 80,
                max_depth: 6,
            },
        )
        .unwrap();

        assert!(res.best_move < 100);
        assert!(board.cell_owner(res.best_move).is_none());
        assert!(res.telemetry.elapsed_ms < 400.0);
    }

    #[test]
    fn game_turn_order_and_undo_work_for_arbitrary_player_order() {
        let mut cfg = GameConfig::default_classic(3);
        // Move AI to index 2 (unlike 2019 where AI had to be index 0).
        cfg.players.rotate_left(1);
        let mut game = Game::new(cfg).unwrap();

        assert_eq!(game.current_turn(), 0);
        game.play_move(0).unwrap();
        assert_eq!(game.current_turn(), 1);
        game.play_move(1).unwrap();
        assert_eq!(game.current_turn(), 2);

        let ai_res = game.play_ai_turn().unwrap();
        assert!(ai_res.best_move < 9);
        assert_eq!(game.current_turn(), 0);

        game.undo().unwrap();
        assert_eq!(game.current_turn(), 2);
        assert_eq!(game.history().len(), 2);
    }
}
