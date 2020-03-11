use crate::model::{ErrorKind::*, MinesweeperModel, ModelResult};

pub struct MinesweeperController {
    model: MinesweeperModel,
    num_correctly_flagged: u32,
    exploded_mine: Option<(u32, u32)>,
}

impl MinesweeperController {
    pub fn new(model: MinesweeperModel) -> Self {
        MinesweeperController {
            model,
            num_correctly_flagged: 0,
            exploded_mine: None,
        }
    }

    /**
     * returns an immutable reference to the model in this controller
     */
    pub fn model(&self) -> &MinesweeperModel {
        &self.model
    }

    pub fn won(&self) -> bool {
        self.num_correctly_flagged == self.model.num_flagged()
    }

    pub fn lost(&self) -> bool {
        self.exploded_mine.is_none()
    }

    pub fn exploded_mine_pos(&self) -> Option<(u32, u32)> {
        self.exploded_mine
    }

    /**
     * The number of flags placed on positions in the model
     * in which mines are buried
     */
    pub fn num_correctly_flagged(&self) -> u32 {
        self.num_correctly_flagged
    }

    /**
     * Flag the zone at the given coordinates. If the zone at those
     * coordinates is already flagged, nothing happens.
     * Fails if given coordinates were out of bounds
     * On success, returns a boolean indicating if a flag was
     * added (true) or removed (false)
     */
    pub fn toggle_flag_at(&mut self, x: u32, y: u32) -> Result<bool, ()> {
        let add_flag = match self.model.is_flagged_at(x, y) {
            // if was not flagged, add a flag (& vice versa)
            Some(b) => !b,
            None => return Err(()),
        };
        // disregard err variants --
        //  OutOfBounds errors should be handled above
        //  and NoOp errors are covered by the fact that
        //  we are toggling based on the result of is_flagged_at
        self.model.change_flag_at(x, y, add_flag).unwrap();
        if self.model.has_mine_at(x, y).unwrap() {
            if add_flag {
                self.num_correctly_flagged += 1;
            } else {
                self.num_correctly_flagged -= 1;
            }
        }
        Ok(add_flag)
    }

    /**
     * TODO
     */
    pub fn reveal_zone_at(&mut self, x: u32, y: u32) -> ModelResult<bool> {
        let has_mine = self.model.reveal_at(x, y)?;
        if has_mine {
            self.exploded_mine = Some((x, y));
        } else if self.model.mines_adjacent_to(x, y).unwrap() == 0 {
            self.cascading_reveal_from(x, y);
        }
        Ok(has_mine)
    }

    /**
     * TODO
     * pre-condition: self.model.num_mines_adjacent_to(starting_x, starting_y).unwrap() == 0
     */
    fn cascading_reveal_from(&mut self, starting_x: u32, starting_y: u32) {
        debug_assert!(
            self.model
                .mines_adjacent_to(starting_x, starting_y)
                .unwrap()
                == 0
        );
        let mut stack = self.model.adjacent_positions(starting_x, starting_y, false);
        while let Some((x, y)) = stack.pop() {
            match self.model.reveal_at(x, y) {
                Ok(_) => stack.extend(
                    // add all adjacent postitions with 0 adjacent mines
                    self.model
                        .adjacent_positions(x, y, false)
                        .into_iter()
                        .filter(|&(x, y)| self.model.mines_adjacent_to(x, y).unwrap() == 0),
                ),
                Err(NoOp) => continue,
                Err(OutOfBounds) => panic!("out of bounds with coordinates {:?}", (x, y)),
            }
        }
    }
}
