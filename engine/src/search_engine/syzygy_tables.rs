use chess::ChessPosition;
use shakmaty_syzygy::Tablebase;

#[derive(Debug)]
pub struct SyzygyTables {
    tb: Option<Tablebase<shakmaty::Chess>>,
    piece_count: usize,
}

impl Default for SyzygyTables {
    fn default() -> Self {
        Self { 
            tb: None,
            piece_count: 0
        }
    }
}

impl SyzygyTables {
    pub fn load_table(&mut self, path: &String) -> String {
        let mut tb: Tablebase<shakmaty::Chess> = unsafe {
            Tablebase::with_mmap_filesystem()
        };
        let result = tb.add_directory(path);
        if result.is_err() {
            *self = SyzygyTables::default();
            return format!("Incorrect syzygy files under {path}!");
        }

        self.piece_count = tb.max_pieces();
        self.tb = Some(tb);

        format!("{} files of {}-man syzygy tables has been loaded succesfully.", result.unwrap(), self.piece_count)
    }
}

fn position_to_shakmaty(position: &ChessPosition) -> shakmaty::Chess {
    let result = shakmaty::Chess::new();

    result
} 