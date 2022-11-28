use std::{fmt, str};

mod generate;
mod make;

use crate::{
    error::{BoardError, ErrorKind},
    piece::Piece,
    square::Square,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Move {
    pub origin: Square,
    pub dest: Square,
    pub promotion: Piece, //Doesn't really need a color...
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!("{}{}", self.origin, self.dest);
        if self.promotion != Piece::Empty {
            output += self.promotion.to_string().to_lowercase().as_str();
        }
        write!(f, "{}", output)
    }
}

impl str::FromStr for Move {
    type Err = BoardError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if (s.len() != 5 && s.len() != 4) || !s.is_ascii() {
            return Err(BoardError::new(
                ErrorKind::InvalidInput,
                "Improperly formatted move",
            ));
        }
        let get_err = || BoardError::new(ErrorKind::InvalidInput, "Improperly formatted move");
        let origin: Square = s.get(0..2).ok_or_else(get_err)?.parse()?;
        let dest: Square = s.get(2..4).ok_or_else(get_err)?.parse()?;

        let promotion = if let Some(promo) = s.get(4..5) {
            let white_promotion =
                s.get(1..2).ok_or_else(get_err)? == "7" && s.get(3..4).ok_or_else(get_err)? == "8";
            let black_promotion =
                s.get(1..2).ok_or_else(get_err)? == "2" && s.get(3..4).ok_or_else(get_err)? == "1";
            if !["k", "q", "b", "n", "r"].contains(&promo) || !(white_promotion || black_promotion)
            {
                return Err(BoardError::new(
                    ErrorKind::InvalidInput,
                    "Improperly formatted move",
                ));
            }
            let promo_case = if white_promotion {
                promo.to_uppercase()
            } else {
                promo.to_lowercase()
            };
            promo_case.parse()?
        } else {
            Piece::Empty
        };
        Ok(Move {
            origin,
            dest,
            promotion,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MoveState {
    pub mv: Move,
    pub capture: Piece,
    pub castle: [bool; 4],
    pub halfmove: u32,
    pub ep_target: Option<Square>,
}
