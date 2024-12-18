use std::time::Instant;

use crate::spear::{base_structures::Side, ChessBoard, StringUtils, FEN};

pub struct Perft;
impl Perft {
    pub fn perft<const BULK: bool, const SPLIT: bool, const PRINT: bool>(
        fen: &FEN,
        depth: u8,
    ) -> (u128, u128) {
        let board = ChessBoard::from_fen(fen);

        if PRINT {
            let pext = {
                #[cfg(feature = "pext")]
                {
                    true
                }

                #[cfg(not(feature = "pext"))]
                {
                    false
                }
            };

            board.draw_board();
            println!("-----------------------------------------------------------");
            println!("  Starting PERFT");
            println!("  Depth: {depth}");
            println!("  Split: {SPLIT}");
            println!("  Bulk: {BULK}");
            println!("  PEXT: {pext}",);
            println!("-----------------------------------------------------------");
        }

        let timer = Instant::now();
        let result = if board.side_to_move() == Side::WHITE {
            perft_internal::<BULK, SPLIT, PRINT, true, true, false>(&board, depth)
        } else {
            perft_internal::<BULK, SPLIT, PRINT, true, false, true>(&board, depth)
        };
        let duration = timer.elapsed().as_millis();

        if PRINT {
            println!("-----------------------------------------------------------");
            println!(
                "  Perft ended! {} nodes, {}, {}n/s",
                result,
                StringUtils::time_to_string(duration),
                StringUtils::large_number_to_string(
                    ((result * 1000) as f64 / duration as f64) as u128
                )
            );
            println!("-----------------------------------------------------------");
        }

        (result, duration)
    }
}

fn perft_internal<
    const BULK: bool,
    const SPLIT: bool,
    const PRINT: bool,
    const FIRST: bool,
    const STM_WHITE: bool,
    const NSTM_WHITE: bool,
>(
    board: &ChessBoard,
    depth: u8,
) -> u128 {
    let mut node_count = 0u128;

    if BULK && depth == 1 {
        board.map_moves::<_, STM_WHITE, NSTM_WHITE>(|_| {
            node_count += 1;
        });
        return node_count;
    }

    if !BULK && depth == 0 {
        return 1;
    }

    board.map_moves::<_, STM_WHITE, NSTM_WHITE>(|mv| {
        let mut board_copy = *board;
        board_copy.make_move::<STM_WHITE, NSTM_WHITE>(mv);
        let result = perft_internal::<BULK, SPLIT, PRINT, false, NSTM_WHITE, STM_WHITE>(
            &board_copy,
            depth - 1,
        );
        node_count += result;

        if SPLIT && PRINT && FIRST {
            println!("{mv} - {result}")
        }
    });

    node_count
}
