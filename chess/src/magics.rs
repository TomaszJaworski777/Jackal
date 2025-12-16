use crate::{Bitboard, Square};

// -----------------------------------------------------------------------------
//  Public API
// -----------------------------------------------------------------------------

#[inline(always)]
pub fn get_rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    unsafe {
        let magic = &MAGICS[sq.get_value() as usize];
        let idx = calculate_index(occ.get_value(), magic);
        Bitboard::from_value(*ATTACK_TABLE.get_unchecked(magic.offset as usize + idx))
    }
}

#[inline(always)]
pub fn get_bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    unsafe {
        let magic = &MAGICS[64 + sq.get_value() as usize];
        let idx = calculate_index(occ.get_value(), magic);
        Bitboard::from_value(*ATTACK_TABLE.get_unchecked(magic.offset as usize + idx))
    }
}

// -----------------------------------------------------------------------------
//  Configuration & Structures
// -----------------------------------------------------------------------------

// PEXT (BMI2) Configuration
#[cfg(target_feature = "bmi2")]
const TABLE_SIZE: usize = 107_648; // Exact size for dense PEXT tables (Rooks + Bishops)

#[cfg(target_feature = "bmi2")]
struct Magic {
    mask: u64,
    offset: u32,
}

// Legacy Black Magic Configuration
#[cfg(not(target_feature = "bmi2"))]
const TABLE_SIZE: usize = 87_988; // Compressed size for Black Magic tables

#[cfg(not(target_feature = "bmi2"))]
struct Magic {
    factor: u64,
    mask: u64,
    offset: u32,
    shift: u8,
}

// -----------------------------------------------------------------------------
//  Index Calculation
// -----------------------------------------------------------------------------

#[inline(always)]
#[cfg(target_feature = "bmi2")]
fn calculate_index(occ: u64, m: &Magic) -> usize {
    // Hardware PEXT: Extract bits of 'occ' specified by 'm.mask'
    // SAFETY: This code path only active when target_feature="bmi2"
    unsafe { core::arch::x86_64::_pext_u64(occ, m.mask) as usize }
}

#[inline(always)]
#[cfg(not(target_feature = "bmi2"))]
const fn calculate_index(occ: u64, m: &Magic) -> usize {
    // Magic: ((blockers | !mask) * factor) >> shift
    (occ | !m.mask).wrapping_mul(m.factor).wrapping_shr(m.shift as u32) as usize
}

// -----------------------------------------------------------------------------
//  Static Tables (Compile-Time Generated)
// -----------------------------------------------------------------------------

#[allow(long_running_const_eval)]
static ATTACK_TABLE: [u64; TABLE_SIZE] = generate_all_attacks();

static MAGICS: [Magic; 128] = generate_magics();

// -----------------------------------------------------------------------------
//  Generators
// -----------------------------------------------------------------------------

const fn generate_magics() -> [Magic; 128] {
    // Because Magic struct fields differ based on cfg, we handle initialization separately.
    #[cfg(target_feature = "bmi2")]
    {
        let mut magics = [const { Magic { mask: 0, offset: 0 } }; 128];
        let mut offset = 0;
        let mut i = 0;
        
        // Rooks (0..64)
        while i < 64 {
            let mask = generate_rook_mask(i as u8);
            magics[i] = Magic { mask, offset };
            offset += 1 << mask.count_ones();
            i += 1;
        }
        // Bishops (64..128)
        let mut j = 0;
        while j < 64 {
            let mask = generate_bishop_mask(j as u8);
            magics[64 + j] = Magic { mask, offset };
            offset += 1 << mask.count_ones();
            j += 1;
        }
        magics
    }

    #[cfg(not(target_feature = "bmi2"))]
    {
        let mut magics = [const { Magic { factor: 0, mask: 0, offset: 0, shift: 0 } }; 128];
        let mut i = 0;
        while i < 64 {
            magics[i] = Magic {
                factor: ROOK_RAW[i].factor,
                mask: generate_rook_mask(i as u8),
                offset: ROOK_RAW[i].offset,
                shift: 64 - 12,
            };
            magics[i + 64] = Magic {
                factor: BISHOP_RAW[i].factor,
                mask: generate_bishop_mask(i as u8),
                offset: BISHOP_RAW[i].offset,
                shift: 64 - 9,
            };
            i += 1;
        }
        magics
    }
}

const fn generate_all_attacks() -> [u64; TABLE_SIZE] {
    let mut table = [0; TABLE_SIZE];
    
    // Rooks
    let mut i = 0;
    let mut current_offset = 0; // Only used for PEXT tracking
    
    while i < 64 {
        let sq = i as u8;
        let mask = generate_rook_mask(sq);
        let subsets = 1 << mask.count_ones();
        let mut j = 0;
        
        // Get offset for this square
        #[cfg(target_feature = "bmi2")]
        let offset = current_offset;
        #[cfg(not(target_feature = "bmi2"))]
        let offset = ROOK_RAW[i].offset;

        while j < subsets {
            let blockers = scatter_bits(j, mask);
            
            // Calculate Index for Table
            #[cfg(target_feature = "bmi2")]
            let idx = j as usize; // For PEXT, the index IS the iteration (j)
            
            #[cfg(not(target_feature = "bmi2"))]
            let idx = calculate_index(blockers, &Magic { 
                factor: ROOK_RAW[i].factor, mask, offset: 0, shift: 64 - 12 
            });

            table[offset as usize + idx] = calculate_rook_moves(sq, blockers);
            j += 1;
        }
        
        #[cfg(target_feature = "bmi2")]
        { current_offset += subsets as u32; }
        
        i += 1;
    }

    // Bishops
    i = 0;
    while i < 64 {
        let sq = i as u8;
        let mask = generate_bishop_mask(sq);
        let subsets = 1 << mask.count_ones();
        let mut j = 0;

        #[cfg(target_feature = "bmi2")]
        let offset = current_offset;
        #[cfg(not(target_feature = "bmi2"))]
        let offset = BISHOP_RAW[i].offset;

        while j < subsets {
            let blockers = scatter_bits(j, mask);
            
            #[cfg(target_feature = "bmi2")]
            let idx = j as usize;

            #[cfg(not(target_feature = "bmi2"))]
            let idx = calculate_index(blockers, &Magic { 
                factor: BISHOP_RAW[i].factor, mask, offset: 0, shift: 64 - 9 
            });

            table[offset as usize + idx] = calculate_bishop_moves(sq, blockers);
            j += 1;
        }

        #[cfg(target_feature = "bmi2")]
        { current_offset += subsets as u32; }

        i += 1;
    }
    table
}

// -----------------------------------------------------------------------------
//  Helper Functions
// -----------------------------------------------------------------------------

const fn scatter_bits(mut index: u64, mut mask: u64) -> u64 {
    let mut result = 0;
    while mask != 0 {
        let lowest_bit = mask & mask.wrapping_neg();
        if (index & 1) != 0 { result |= lowest_bit; }
        mask &= mask - 1;
        index >>= 1;
    }
    result
}

const fn cast_ray(sq: u8, file_delta: i8, rank_delta: i8, blockers: u64) -> u64 {
    let mut moves = 0;
    let mut r = (sq / 8) as i8;
    let mut f = (sq % 8) as i8;
    loop {
        r += rank_delta;
        f += file_delta;
        if r < 0 || r > 7 || f < 0 || f > 7 { break; }
        let dest = (r * 8 + f) as u8;
        moves |= 1 << dest;
        if (blockers & (1 << dest)) != 0 { break; }
    }
    moves
}

const fn calculate_rook_moves(sq: u8, blockers: u64) -> u64 {
    cast_ray(sq, 1, 0, blockers) | cast_ray(sq, -1, 0, blockers) |
    cast_ray(sq, 0, 1, blockers) | cast_ray(sq, 0, -1, blockers)
}

const fn calculate_bishop_moves(sq: u8, blockers: u64) -> u64 {
    cast_ray(sq, 1, 1, blockers) | cast_ray(sq, 1, -1, blockers) |
    cast_ray(sq, -1, 1, blockers) | cast_ray(sq, -1, -1, blockers)
}

const fn generate_rook_mask(sq: u8) -> u64 {
    let rank = sq / 8;
    let file = sq % 8;
    let rank_part = (0x7E_u64) << (rank * 8);
    let file_part = (0x0101010101010101_u64 << file) & 0x00FFFFFFFFFFFF00;
    (rank_part | file_part) & !(1 << sq)
}

const fn generate_bishop_mask(sq: u8) -> u64 {
    calculate_bishop_moves(sq, 0) & 0x007E7E7E7E7E7E00
}

// -----------------------------------------------------------------------------
//  Raw Magic Data (Excluded when PEXT is available)
// -----------------------------------------------------------------------------

#[cfg(not(target_feature = "bmi2"))]
#[derive(Clone, Copy)]
struct RawMagic { factor: u64, offset: u32 }

#[cfg(not(target_feature = "bmi2"))]
#[rustfmt::skip]
const BISHOP_RAW: [RawMagic; 64] = [
    RawMagic { factor: 0xa7020080601803d8, offset: 60984 }, RawMagic { factor: 0x13802040400801f1, offset: 66046 },
    RawMagic { factor: 0x0a0080181001f60c, offset: 32910 }, RawMagic { factor: 0x1840802004238008, offset: 16369 },
    RawMagic { factor: 0xc03fe00100000000, offset: 42115 }, RawMagic { factor: 0x24c00bffff400000, offset: 835 },
    RawMagic { factor: 0x0808101f40007f04, offset: 18910 }, RawMagic { factor: 0x100808201ec00080, offset: 25911 },
    RawMagic { factor: 0xffa2feffbfefb7ff, offset: 63301 }, RawMagic { factor: 0x083e3ee040080801, offset: 16063 },
    RawMagic { factor: 0xc0800080181001f8, offset: 17481 }, RawMagic { factor: 0x0440007fe0031000, offset: 59361 },
    RawMagic { factor: 0x2010007ffc000000, offset: 18735 }, RawMagic { factor: 0x1079ffe000ff8000, offset: 61249 },
    RawMagic { factor: 0x3c0708101f400080, offset: 68938 }, RawMagic { factor: 0x080614080fa00040, offset: 61791 },
    RawMagic { factor: 0x7ffe7fff817fcff9, offset: 21893 }, RawMagic { factor: 0x7ffebfffa01027fd, offset: 62068 },
    RawMagic { factor: 0x53018080c00f4001, offset: 19829 }, RawMagic { factor: 0x407e0001000ffb8a, offset: 26091 },
    RawMagic { factor: 0x201fe000fff80010, offset: 15815 }, RawMagic { factor: 0xffdfefffde39ffef, offset: 16419 },
    RawMagic { factor: 0xcc8808000fbf8002, offset: 59777 }, RawMagic { factor: 0x7ff7fbfff8203fff, offset: 16288 },
    RawMagic { factor: 0x8800013e8300c030, offset: 33235 }, RawMagic { factor: 0x0420009701806018, offset: 15459 },
    RawMagic { factor: 0x7ffeff7f7f01f7fd, offset: 15863 }, RawMagic { factor: 0x8700303010c0c006, offset: 75555 },
    RawMagic { factor: 0xc800181810606000, offset: 79445 }, RawMagic { factor: 0x20002038001c8010, offset: 15917 },
    RawMagic { factor: 0x087ff038000fc001, offset: 8512 }, RawMagic { factor: 0x00080c0c00083007, offset: 73069 },
    RawMagic { factor: 0x00000080fc82c040, offset: 16078 }, RawMagic { factor: 0x000000407e416020, offset: 19168 },
    RawMagic { factor: 0x00600203f8008020, offset: 11056 }, RawMagic { factor: 0xd003fefe04404080, offset: 62544 },
    RawMagic { factor: 0xa00020c018003088, offset: 80477 }, RawMagic { factor: 0x7fbffe700bffe800, offset: 75049 },
    RawMagic { factor: 0x107ff00fe4000f90, offset: 32947 }, RawMagic { factor: 0x7f8fffcff1d007f8, offset: 59172 },
    RawMagic { factor: 0x0000004100f88080, offset: 55845 }, RawMagic { factor: 0x00000020807c4040, offset: 61806 },
    RawMagic { factor: 0x00000041018700c0, offset: 73601 }, RawMagic { factor: 0x0010000080fc4080, offset: 15546 },
    RawMagic { factor: 0x1000003c80180030, offset: 45243 }, RawMagic { factor: 0xc10000df80280050, offset: 20333 },
    RawMagic { factor: 0xffffffbfeff80fdc, offset: 33402 }, RawMagic { factor: 0x000000101003f812, offset: 25917 },
    RawMagic { factor: 0x0800001f40808200, offset: 32875 }, RawMagic { factor: 0x084000101f3fd208, offset: 4639 },
    RawMagic { factor: 0x080000000f808081, offset: 17077 }, RawMagic { factor: 0x0004000008003f80, offset: 62324 },
    RawMagic { factor: 0x08000001001fe040, offset: 18159 }, RawMagic { factor: 0x72dd000040900a00, offset: 61436 },
    RawMagic { factor: 0xfffffeffbfeff81d, offset: 57073 }, RawMagic { factor: 0xcd8000200febf209, offset: 61025 },
    RawMagic { factor: 0x100000101ec10082, offset: 81259 }, RawMagic { factor: 0x7fbaffffefe0c02f, offset: 64083 },
    RawMagic { factor: 0x7f83fffffff07f7f, offset: 56114 }, RawMagic { factor: 0xfff1fffffff7ffc1, offset: 57058 },
    RawMagic { factor: 0x0878040000ffe01f, offset: 58912 }, RawMagic { factor: 0x945e388000801012, offset: 22194 },
    RawMagic { factor: 0x0840800080200fda, offset: 70880 }, RawMagic { factor: 0x100000c05f582008, offset: 11140 },
];

#[cfg(not(target_feature = "bmi2"))]
#[rustfmt::skip]
const ROOK_RAW: [RawMagic; 64] = [
    RawMagic { factor: 0x80280013ff84ffff, offset: 10890 }, RawMagic { factor: 0x5ffbfefdfef67fff, offset: 50579 },
    RawMagic { factor: 0xffeffaffeffdffff, offset: 62020 }, RawMagic { factor: 0x003000900300008a, offset: 67322 },
    RawMagic { factor: 0x0050028010500023, offset: 80251 }, RawMagic { factor: 0x0020012120a00020, offset: 58503 },
    RawMagic { factor: 0x0030006000c00030, offset: 51175 }, RawMagic { factor: 0x0058005806b00002, offset: 83130 },
    RawMagic { factor: 0x7fbff7fbfbeafffc, offset: 50430 }, RawMagic { factor: 0x0000140081050002, offset: 21613 },
    RawMagic { factor: 0x0000180043800048, offset: 72625 }, RawMagic { factor: 0x7fffe800021fffb8, offset: 80755 },
    RawMagic { factor: 0xffffcffe7fcfffaf, offset: 69753 }, RawMagic { factor: 0x00001800c0180060, offset: 26973 },
    RawMagic { factor: 0x4f8018005fd00018, offset: 84972 }, RawMagic { factor: 0x0000180030620018, offset: 31958 },
    RawMagic { factor: 0x00300018010c0003, offset: 69272 }, RawMagic { factor: 0x0003000c0085ffff, offset: 48372 },
    RawMagic { factor: 0xfffdfff7fbfefff7, offset: 65477 }, RawMagic { factor: 0x7fc1ffdffc001fff, offset: 43972 },
    RawMagic { factor: 0xfffeffdffdffdfff, offset: 57154 }, RawMagic { factor: 0x7c108007befff81f, offset: 53521 },
    RawMagic { factor: 0x20408007bfe00810, offset: 30534 }, RawMagic { factor: 0x0400800558604100, offset: 16548 },
    RawMagic { factor: 0x0040200010080008, offset: 46407 }, RawMagic { factor: 0x0010020008040004, offset: 11841 },
    RawMagic { factor: 0xfffdfefff7fbfff7, offset: 21112 }, RawMagic { factor: 0xfebf7dfff8fefff9, offset: 44214 },
    RawMagic { factor: 0xc00000ffe001ffe0, offset: 57925 }, RawMagic { factor: 0x4af01f00078007c3, offset: 29574 },
    RawMagic { factor: 0xbffbfafffb683f7f, offset: 17309 }, RawMagic { factor: 0x0807f67ffa102040, offset: 40143 },
    RawMagic { factor: 0x200008e800300030, offset: 64659 }, RawMagic { factor: 0x0000008780180018, offset: 70469 },
    RawMagic { factor: 0x0000010300180018, offset: 62917 }, RawMagic { factor: 0x4000008180180018, offset: 60997 },
    RawMagic { factor: 0x008080310005fffa, offset: 18554 }, RawMagic { factor: 0x4000188100060006, offset: 14385 },
    RawMagic { factor: 0xffffff7fffbfbfff, offset: 0 },     RawMagic { factor: 0x0000802000200040, offset: 38091 },
    RawMagic { factor: 0x20000202ec002800, offset: 25122 }, RawMagic { factor: 0xfffff9ff7cfff3ff, offset: 60083 },
    RawMagic { factor: 0x000000404b801800, offset: 72209 }, RawMagic { factor: 0x2000002fe03fd000, offset: 67875 },
    RawMagic { factor: 0xffffff6ffe7fcffd, offset: 56290 }, RawMagic { factor: 0xbff7efffbfc00fff, offset: 43807 },
    RawMagic { factor: 0x000000100800a804, offset: 73365 }, RawMagic { factor: 0x6054000a58005805, offset: 76398 },
    RawMagic { factor: 0x0829000101150028, offset: 20024 }, RawMagic { factor: 0x00000085008a0014, offset: 9513 },
    RawMagic { factor: 0x8000002b00408028, offset: 24324 }, RawMagic { factor: 0x4000002040790028, offset: 22996 },
    RawMagic { factor: 0x7800002010288028, offset: 23213 }, RawMagic { factor: 0x0000001800e08018, offset: 56002 },
    RawMagic { factor: 0xa3a80003f3a40048, offset: 22809 }, RawMagic { factor: 0x2003d80000500028, offset: 44545 },
    RawMagic { factor: 0xfffff37eefefdfbe, offset: 36072 }, RawMagic { factor: 0x40000280090013c1, offset: 4750 },
    RawMagic { factor: 0xbf7ffeffbffaf71f, offset: 6014 },  RawMagic { factor: 0xfffdffff777b7d6e, offset: 36054 },
    RawMagic { factor: 0x48300007e8080c02, offset: 78538 }, RawMagic { factor: 0xafe0000fff780402, offset: 28745 },
    RawMagic { factor: 0xee73fffbffbb77fe, offset: 8555 },  RawMagic { factor: 0x0002000308482882, offset: 1009 },
];