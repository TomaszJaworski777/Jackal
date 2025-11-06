use crate::{Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Side, Square, attacks::Rays};

const SEE_VALUES: [i32; 7] = [100, 450, 450, 650, 1250, 0, 0];

fn see_value(piece: Piece) -> i32 {
    SEE_VALUES[usize::from(piece)]
}

impl ChessBoard {
    pub fn see_value(piece: Piece) -> i32 {
        see_value(piece)
    }

    pub fn see(&self, mv: Move, threshold: i32) -> bool {
        let from = mv.from_square();
        let to = mv.to_square();
        let side = self.side();

        let moved_pc = self.piece_on_square(from);
        let captured_pc = if mv.is_en_passant() { Piece::PAWN } else { self.piece_on_square(to) };

        let from_bb = Bitboard::from(from);
        let ksq = self.king_square(side);
        let pinned = {
            let occ = self.occupancy();
            let boys = self.occupancy_for_side(side);
            let opps = self.occupancy_for_side(side.flipped());
            let rooks = self.piece_mask(Piece::QUEEN) | self.piece_mask(Piece::ROOK);
            let bishop = self.piece_mask(Piece::QUEEN) | self.piece_mask(Piece::BISHOP);

            let mut pinned = Bitboard::EMPTY;
            let mut pinners = xray_rook(ksq, occ, boys) & opps & rooks;
            pinners.map(|sq| {
                pinned |= Rays::get_ray(sq, ksq) & boys;
            });

            pinners = xray_rook(ksq, occ, boys) & opps & bishop;
            pinners.map(|sq| {
                pinned |= Rays::get_ray(sq, ksq) & boys;
            });

            pinned
        };

        if (pinned & from_bb).is_not_empty() && (Rays::get_ray(ksq, to) & from_bb).is_empty() {
            return false;
        }

        let mut score = SEE_VALUES[usize::from(captured_pc)] - threshold;

        if mv.is_promotion() {
            let promo_val = SEE_VALUES[usize::from(mv.promotion_piece())];
            score += promo_val - SEE_VALUES[usize::from(Piece::PAWN)];
            if score < 0 {
                return false;
            }
            score -= promo_val;
            if score >= 0 {
                return true;
            }
        } else {
            if score < 0 {
                return false;
            }
            score -= SEE_VALUES[usize::from(moved_pc)];
            if score >= 0 {
                let to_bb = Bitboard::from(to);
                let cap_sq = if mv.is_en_passant() { to ^ 8 } else { to };
                let cap_bb = Bitboard::from(cap_sq);

                // build board after the capture to check for further attackers
                let mut occ_after = self.occupancy();
                occ_after ^= from_bb;
                occ_after ^= cap_bb;
                occ_after |= to_bb;
                let occ_att = occ_after ^ to_bb;

                let mut pieces_after = self.pieces();
                let mut occupancy_after = [self.occupancy_for_side(Side::WHITE), self.occupancy_for_side(Side::BLACK)];
                pieces_after[usize::from(moved_pc)] ^= from_bb;
                occupancy_after[usize::from(side)] ^= from_bb;
                if captured_pc != Piece::NONE {
                    pieces_after[usize::from(captured_pc)] ^= cap_bb;
                    occupancy_after[usize::from(side.flipped())] ^= cap_bb;
                }
                pieces_after[usize::from(moved_pc)] |= to_bb;
                occupancy_after[usize::from(side)] |= to_bb;

                let opp = side.flipped();
                let queens = pieces_after[usize::from(Piece::QUEEN)];
                let rooks = pieces_after[usize::from(Piece::ROOK)] | queens;
                let bishops = pieces_after[usize::from(Piece::BISHOP)] | queens;
                let pawns_w = pieces_after[usize::from(Piece::PAWN)] & occupancy_after[usize::from(Side::WHITE)];
                let pawns_b = pieces_after[usize::from(Piece::PAWN)] & occupancy_after[usize::from(Side::BLACK)];
                let mut opp_attackers = (Attacks::get_king_attacks(to) & pieces_after[usize::from(Piece::KING)])
                    | (Attacks::get_knight_attacks(to) & pieces_after[usize::from(Piece::KNIGHT)])
                    | (Attacks::get_bishop_attacks(to, occ_att) & bishops)
                    | (Attacks::get_rook_attacks(to, occ_att) & rooks)
                    | (Attacks::get_pawn_attacks(to, Side::WHITE) & pawns_b)
                    | (Attacks::get_pawn_attacks(to, Side::BLACK) & pawns_w);
                opp_attackers &= occupancy_after[usize::from(opp)];

                if opp_attackers.is_empty() {
                    return true;
                }

                let promo_attackers = pieces_after[usize::from(Piece::PAWN)] & occupancy_after[usize::from(opp)] & [Bitboard::RANK_7, Bitboard::RANK_2][usize::from(opp)];
                if (Attacks::get_pawn_attacks(to, side) & promo_attackers).is_empty() {
                    return true;
                }
                let promo_penalty = SEE_VALUES[usize::from(Piece::QUEEN)] - SEE_VALUES[usize::from(Piece::PAWN)];
                if score >= promo_penalty {
                    return true;
                }
                // if a pawn can recapture and promote, fall through to full
                // static exchange evaluation without modifying the score so
                // that further recaptures (e.g. king capturing the promoted
                // queen) are considered.
            }
        }

        let mut occ = self.occupancy();
        let to_bb = Bitboard::from(to);
        occ &= !from_bb;
        occ &= !to_bb;
        if mv.is_en_passant() {
            occ &= !Bitboard::from(to ^ 8);
        }

        if mv.flag() == MoveFlag::DOUBLE_PUSH {
            let ep_sq = to ^ 8;
            let opp = side.flipped();
            let mut ep_attackers = Attacks::get_pawn_attacks(ep_sq, side) & self.piece_mask_for_side(Piece::PAWN, opp);
            if ep_attackers.is_not_empty() {
                let mut occ_after = self.occupancy();
                occ_after ^= from_bb | Bitboard::from(to);
                let mut pieces_after = self.pieces();
                let mut occupancy_after = [self.occupancy_for_side(Side::WHITE), self.occupancy_for_side(Side::BLACK)];
                pieces_after[usize::from(Piece::PAWN)] ^= from_bb | Bitboard::from(to);
                occupancy_after[usize::from(side)] ^= from_bb | Bitboard::from(to);
                let pinned_opp = recompute_pins(&pieces_after, &occupancy_after, occ_after, opp, self.king_square(opp));
                ep_attackers &= !pinned_opp | (Rays::get_ray(self.king_square(opp), ep_sq) & pinned_opp);
                if ep_attackers.is_not_empty() {
                    let mut legal = false;
                    let mut attackers = ep_attackers;
                    while attackers.is_not_empty() {
                        let src = attackers.pop_ls1b_square();
                        let from_bit = Bitboard::from(src);
                        let mut occ_cap = occ_after ^ from_bit ^ Bitboard::from(to);
                        occ_cap |= Bitboard::from(ep_sq);
                        let mut pieces_cap = pieces_after;
                        let mut occup_cap = occupancy_after;
                        pieces_cap[usize::from(Piece::PAWN)] ^= from_bit | Bitboard::from(to) | Bitboard::from(ep_sq);
                        occup_cap[usize::from(opp)] ^= from_bit;
                        occup_cap[usize::from(opp)] |= Bitboard::from(ep_sq);
                        occup_cap[usize::from(side)] &= !Bitboard::from(to);

                        let king_sq = self.king_square(opp);
                        let queens = pieces_cap[usize::from(Piece::QUEEN)];
                        let rooks = pieces_cap[usize::from(Piece::ROOK)] | queens;
                        let bishops = pieces_cap[usize::from(Piece::BISHOP)] | queens;
                        let mut checkers = (Attacks::get_king_attacks(king_sq) & pieces_cap[usize::from(Piece::KING)])
                            | (Attacks::get_knight_attacks(king_sq) & pieces_cap[usize::from(Piece::KNIGHT)])
                            | (Attacks::get_bishop_attacks(king_sq, occ_cap) & bishops)
                            | (Attacks::get_rook_attacks(king_sq, occ_cap) & rooks)
                            | (Attacks::get_pawn_attacks(king_sq, opp) & pieces_cap[usize::from(Piece::PAWN)]);
                        checkers &= occup_cap[usize::from(side)];
                        if checkers.is_empty() {
                            legal = true;
                            break;
                        }
                    }
                    if legal {
                        return threshold <= -SEE_VALUES[usize::from(Piece::PAWN)];
                    }
                }
            }
        }

        let mut pieces = self.pieces();
        let mut occupancy = [self.occupancy_for_side(Side::WHITE), self.occupancy_for_side(Side::BLACK)];
        pieces[usize::from(moved_pc)] &= !from_bb;
        occupancy[usize::from(side)] &= !from_bb;

        if captured_pc != Piece::NONE {
            let cap_sq = if mv.is_en_passant() { to ^ 8 } else { to };
            let cap_bb = Bitboard::from(cap_sq);
            pieces[usize::from(captured_pc)] &= !cap_bb;
            occupancy[usize::from(side.flipped())] &= !cap_bb;
        }

        // after making the move on the board, see if the opponent is in check. If they
        // are, they might be restricted in how they can recapture: in a double check or
        // if the checking piece isn't the one on the target square, only king captures
        // can be considered. We compute this information here and apply it after
        // generating the attackers.
        let mut pieces_after = pieces;
        let mut occupancy_after = occupancy;
        let occ_after = occ | to_bb;
        pieces_after[usize::from(moved_pc)] |= to_bb;
        occupancy_after[usize::from(side)] |= to_bb;

        let opp = side.flipped();
        let ksq_opp = self.king_square(opp);
        let queens_after = pieces_after[usize::from(Piece::QUEEN)];
        let rooks_after = pieces_after[usize::from(Piece::ROOK)] | queens_after;
        let bishops_after = pieces_after[usize::from(Piece::BISHOP)] | queens_after;

        let mut checkers = (Attacks::get_king_attacks(ksq_opp) & pieces_after[usize::from(Piece::KING)])
            | (Attacks::get_knight_attacks(ksq_opp) & pieces_after[usize::from(Piece::KNIGHT)])
            | (Attacks::get_bishop_attacks(ksq_opp, occ_after) & bishops_after)
            | (Attacks::get_rook_attacks(ksq_opp, occ_after) & rooks_after)
            | (Attacks::get_pawn_attacks(ksq_opp, opp) & pieces_after[usize::from(Piece::PAWN)]);
        checkers &= occupancy_after[usize::from(side)];

        let opp_in_check = checkers.is_not_empty();
        let double_check = (checkers & (checkers - 1)).is_not_empty();
        let checker_on_to = (checkers & to_bb).is_not_empty();

        let mut stm = side.flipped();
        let mut attackers = {
            let queens = pieces[usize::from(Piece::QUEEN)];
            let rooks = pieces[usize::from(Piece::ROOK)] | queens;
            let bishops = pieces[usize::from(Piece::BISHOP)] | queens;
            let knights = pieces[usize::from(Piece::KNIGHT)];
            let kings = pieces[usize::from(Piece::KING)];
            let pawns_w = pieces[usize::from(Piece::PAWN)] & occupancy[usize::from(Side::WHITE)];
            let pawns_b = pieces[usize::from(Piece::PAWN)] & occupancy[usize::from(Side::BLACK)];
            (Attacks::get_king_attacks(to) & kings)
                | (Attacks::get_knight_attacks(to) & knights)
                | (Attacks::get_bishop_attacks(to, occ) & bishops)
                | (Attacks::get_rook_attacks(to, occ) & rooks)
                | (Attacks::get_pawn_attacks(to, Side::WHITE) & pawns_b)
                | (Attacks::get_pawn_attacks(to, Side::BLACK) & pawns_w)
        };

        if opp_in_check && (double_check || !checker_on_to) {
            attackers &= Attacks::get_king_attacks(to);
        }

        #[inline]
        fn recompute_pins(pieces: &[Bitboard; 6], occupnacy: &[Bitboard; 2], occ: Bitboard, side: Side, ksq: Square) -> Bitboard {
            let boys = occupnacy[usize::from(side)];
            let opps = occupnacy[usize::from(side.flipped())];
            let rq = pieces[usize::from(Piece::QUEEN)] | pieces[usize::from(Piece::ROOK)];
            let bq = pieces[usize::from(Piece::QUEEN)] | pieces[usize::from(Piece::BISHOP)];
            let mut pinned = Bitboard::EMPTY;

            let mut pinners = xray_rook(ksq, occ, boys) & opps & rq;
            pinners.map(|sq| {
                pinned |= Rays::get_ray(sq, ksq) & boys;
            });

            pinners = xray_rook(ksq, occ, boys) & opps & bq;
            pinners.map(|sq| {
                pinned |= Rays::get_ray(sq, ksq) & boys;
            });

            pinned
        }

        let mut pinned_w = recompute_pins(&pieces, &occupancy, occ, Side::WHITE, self.king_square(Side::WHITE));
        let mut pinned_b = recompute_pins(&pieces, &occupancy, occ, Side::BLACK, self.king_square(Side::BLACK));

        fn remove_least(
            pieces: &mut [Bitboard; 6],
            occupancy: &mut [Bitboard; 2],
            mask: Bitboard,
            occ: &mut Bitboard,
            opp_king: Square,
            opp_pinned: Bitboard,
            to: Square,
        ) -> Option<(usize, Bitboard)> {
            const ORDER: [Piece; 6] =
                [Piece::PAWN, Piece::KNIGHT, Piece::BISHOP, Piece::ROOK, Piece::QUEEN, Piece::KING];

            let mut global_fallback: Option<(usize, Bitboard)> = None;

            for &piece_idx in &ORDER.map(|x| usize::from(x)) {
                let mut bb = pieces[piece_idx] & mask;
                if bb.is_empty() {
                    continue;
                }

                // prefer moves that do not release pins on the opponent and do not
                // uncover x-ray attacks on the destination square
                let mut fallback_no_xray = None;

                while bb.is_not_empty() {
                    let bit = bb & bb.get_value().wrapping_neg();
                    bb ^= bit;
                    let sq = bit.ls1b_square();

                    let releases_pin = (Rays::get_ray(opp_king, sq) & opp_pinned).is_not_empty();

                    // check if moving this piece reveals a new slider attack on `to`
                    let occ_after = *occ ^ bit;
                    let side = if (occupancy[usize::from(Side::WHITE)] & bit).is_not_empty() { Side::WHITE } else { Side::BLACK };
                    let opp = side.flipped();
                    let bishops = (pieces[usize::from(Piece::BISHOP)] | pieces[usize::from(Piece::QUEEN)]) & occupancy[usize::from(opp)];
                    let rooks = (pieces[usize::from(Piece::ROOK)] | pieces[usize::from(Piece::QUEEN)]) & occupancy[usize::from(opp)];
                    let pawns = pieces[usize::from(Piece::PAWN)] & occupancy[usize::from(opp)];
                    let pawn_attack = (Attacks::get_pawn_attacks(to, side) & pawns).is_not_empty();
                    let existing_xray = pawn_attack
                        || (Attacks::get_bishop_attacks(to, *occ) & bishops).is_not_empty()
                        || (Attacks::get_rook_attacks(to, *occ) & rooks).is_not_empty();
                    let opens_xray = !existing_xray
                        && ((Attacks::get_bishop_attacks(to, occ_after) & bishops).is_not_empty()
                            || (Attacks::get_rook_attacks(to, occ_after) & rooks).is_not_empty());

                    // If the attacker is a pawn on the promotion rank, prefer it even if
                    // it releases a pin or opens an x-ray. Such captures are often the
                    // only legal reply and ignoring them can dramatically skew the SEE.
                    let promo_pawn = piece_idx == usize::from(Piece::PAWN) && (bit & [Bitboard::RANK_7, Bitboard::RANK_2][usize::from(side)]).is_not_empty();

                    #[allow(clippy::nonminimal_bool)]
                    if (!releases_pin || promo_pawn) && (!opens_xray || promo_pawn) {
                        // best option: neither releases a pin nor opens an x-ray, or
                        // we must consider the promotion capture regardless
                        pieces[piece_idx] ^= bit;
                        if (occupancy[usize::from(Side::WHITE)] & bit).is_not_empty() {
                            occupancy[usize::from(Side::WHITE)] ^= bit;
                        } else {
                            occupancy[usize::from(Side::BLACK)] ^= bit;
                        }
                        *occ ^= bit;
                        return Some((piece_idx, bit));
                    }

                    if (!opens_xray || promo_pawn) && fallback_no_xray.is_none() {
                        fallback_no_xray = Some(bit);
                    }

                    if global_fallback.is_none() {
                        global_fallback = Some((piece_idx, bit));
                    }
                }

                if let Some(bit) = fallback_no_xray {
                    pieces[piece_idx] ^= bit;
                    if (occupancy[usize::from(Side::WHITE)] & bit).is_not_empty() {
                        occupancy[usize::from(Side::WHITE)] ^= bit;
                    } else {
                        occupancy[usize::from(Side::BLACK)] ^= bit;
                    }
                    *occ ^= bit;
                    return Some((piece_idx, bit));
                }
            }

            if let Some((pc, bit)) = global_fallback {
                pieces[pc] ^= bit;
                if (occupancy[usize::from(Side::WHITE)] & bit).is_not_empty() {
                    occupancy[usize::from(Side::WHITE)] ^= bit;
                } else {
                    occupancy[usize::from(Side::BLACK)] ^= bit;
                }
                *occ ^= bit;
                return Some((pc, bit));
            }

            None
        }

        while (attackers & occupancy[usize::from(stm)]).is_not_empty() {
            let allowed = {
                let all_pinned = pinned_w | pinned_b;
                let white_allowed = pinned_w & Rays::get_ray(self.king_square(Side::WHITE), to);
                let black_allowed = pinned_b & Rays::get_ray(self.king_square(Side::BLACK), to);
                !all_pinned | white_allowed | black_allowed
            };

            let our_attackers = attackers & occupancy[usize::from(stm)] & allowed;
            let opp_pinned = if stm == Side::WHITE { pinned_b } else { pinned_w };
            let opp_king_sq = self.king_square(stm.flipped());
            let Some((mut attacker_pc, from_bit)) =
                remove_least(&mut pieces, &mut occupancy, our_attackers, &mut occ, opp_king_sq, opp_pinned, to)
            else {
                break;
            };

            // after hypothetically moving this attacker to `to`, check if it leaves the king in check
            {
                let mut pieces_after = pieces;
                let mut occupancy_after = occupancy;
                let occ_after = occ | to_bb;
                pieces_after[attacker_pc] |= to_bb;
                occupancy_after[usize::from(stm)] |= to_bb;
                let ksq = if attacker_pc == usize::from(Piece::KING) { to } else { self.king_square(stm) };

                let queens = pieces_after[usize::from(Piece::QUEEN)];
                let rooks = pieces_after[usize::from(Piece::ROOK)] | queens;
                let bishops = pieces_after[usize::from(Piece::BISHOP)] | queens;
                let pawns_w = pieces_after[usize::from(Piece::PAWN)] & occupancy_after[usize::from(Side::WHITE)];
                let pawns_b = pieces_after[usize::from(Piece::PAWN)] & occupancy_after[usize::from(Side::BLACK)];
                let pawn_attacks = if stm == Side::WHITE {
                    Attacks::get_pawn_attacks(ksq, Side::WHITE) & pawns_b
                } else {
                    Attacks::get_pawn_attacks(ksq, Side::BLACK) & pawns_w
                };

                let mut checkers = (Attacks::get_king_attacks(ksq) & pieces_after[usize::from(Piece::KING)])
                    | (Attacks::get_knight_attacks(ksq) & pieces_after[usize::from(Piece::KNIGHT)])
                    | (Attacks::get_bishop_attacks(ksq, occ_after) & bishops)
                    | (Attacks::get_rook_attacks(ksq, occ_after) & rooks)
                    | pawn_attacks;
                checkers &= occupancy_after[usize::from(stm.flipped())];
                if checkers.is_not_empty() {
                    // revert removal and skip this attacker
                    pieces[attacker_pc] |= from_bit;
                    occupancy[usize::from(stm)] |= from_bit;
                    occ |= from_bit;
                    attackers &= !from_bit;
                    continue;
                }
            }

            let capture_val = SEE_VALUES[attacker_pc];
            if attacker_pc == usize::from(Piece::PAWN) && ((stm == Side::WHITE && usize::from(to) >= 56) || (stm == Side::BLACK && usize::from(to) < 8)) {
                attacker_pc = usize::from(Piece::QUEEN);
            }

            let queens = pieces[usize::from(Piece::QUEEN)];
            let rooks = pieces[usize::from(Piece::ROOK)] | queens;
            let bishops = pieces[usize::from(Piece::BISHOP)] | queens;

            if attacker_pc == usize::from(Piece::PAWN) || attacker_pc == usize::from(Piece::BISHOP) || attacker_pc == usize::from(Piece::QUEEN) {
                attackers |= Attacks::get_bishop_attacks(to, occ) & bishops;
            }
            if attacker_pc == usize::from(Piece::ROOK) || attacker_pc == usize::from(Piece::QUEEN) {
                attackers |= Attacks::get_rook_attacks(to, occ) & rooks;
            }

            attackers &= occ;

            if attacker_pc == usize::from(Piece::KING) && (attackers & occupancy[usize::from(stm.flipped())]).is_not_empty() {
                break;
            }

            score = -score - 1 - capture_val;
            stm.flip();

            pinned_w = recompute_pins(&pieces, &occupancy, occ, Side::WHITE, self.king_square(Side::WHITE));
            pinned_b = recompute_pins(&pieces, &occupancy, occ, Side::BLACK, self.king_square(Side::BLACK));

            let promo_attackers = attackers & occupancy[usize::from(stm)] & pieces[usize::from(Piece::PAWN)] & [Bitboard::RANK_7, Bitboard::RANK_2][usize::from(stm)];
            if score >= 0 && promo_attackers.is_empty() {
                break;
            }
        }

        stm != side
    }
}

#[inline]
fn xray_rook(sq: Square, occ: Bitboard, blockers: Bitboard) -> Bitboard {
    let attacks = Attacks::get_rook_attacks(sq, occ);
    attacks ^ Attacks::get_rook_attacks(sq, occ ^ (attacks & blockers))
}