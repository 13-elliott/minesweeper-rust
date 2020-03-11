//#![allow(dead_code)]

use rand::Rng;
use std::collections::HashSet;

pub type MinesweeperModel = Field;
pub type ModelResult<T> = Result<T, ErrorKind>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/**
 * Enumeration for use in a ModelResult
 * The `OutOfBounds` variant indicates that the given
 * coordinates are not in range of the Model's dimensions
 * The `NoOp` variant indicates that the requested action would
 * have no change or does not make sense for the given coordinate position
 */
pub enum ErrorKind {
    OutOfBounds,
    NoOp,
}

struct Zone {
    flagged: bool,
    revealed: bool,
    has_mine: bool,
    adj_mine_count: u32,
}

impl Zone {
    fn new(has_mine: bool) -> Self {
        Zone {
            flagged: false,
            revealed: false,
            has_mine,
            adj_mine_count: 0,
        }
    }
}

pub struct Field {
    num_mines: u32,
    num_flagged: u32,
    grid: Vec<Vec<Zone>>,
}

impl Field {
    /**
     * Create a new Field. width and height must be greater than 0.
     */
    pub fn new(width: u32, height: u32, num_mines: u32) -> Option<Self> {
        if num_mines > width * height {
            None
        } else {
            let mine_placements = Self::generate_placements(num_mines, width, height);
            Self::with_mine_placements(width, height, mine_placements)
        }
    }

    pub fn with_mine_placements(
        width: u32,
        height: u32,
        placements: impl IntoIterator<Item = (u32, u32)>,
    ) -> Option<Self> {
        let placements: HashSet<_> = placements.into_iter().collect();
        if placements.len() > (width * height) as usize {
            return None;
        }
        let mut freshly_made = Field {
            num_mines: placements.len() as u32,
            num_flagged: 0,
            grid: Self::generate_grid(width, height, &placements),
        };
        freshly_made.set_adj_counts(placements);
        Some(freshly_made)
    }

    /**
     * The height of this Field
     */
    pub fn height(&self) -> u32 {
        self.grid[0].len() as u32
    }

    /**
     * The width of this Field
     */
    pub fn width(&self) -> u32 {
        self.grid.len() as u32
    }

    /**
     * The number of mines buried in this Field
     */
    pub fn num_mines(&self) -> u32 {
        self.num_mines
    }

    /**
     * The total number of flags planted on this Field
     */
    pub fn num_flagged(&self) -> u32 {
        self.num_flagged
    }

    /**
     * boolean indicating if there is a flag planted at the given coordinates
     */
    pub fn is_flagged_at(&self, x: u32, y: u32) -> Option<bool> {
        self.zone_at(x, y).map(|z| z.flagged)
    }

    /**
     * if `add` is true, adds a flag, otherwise removes a flag
     * if trying to add a flag to a zone that is already flagged,
     * or trying to remove a flag from a zone without a flag, then
     * nothing will be done and Err(ErrorKind::NoOp) will be returned.
     */
    pub fn change_flag_at(&mut self, x: u32, y: u32, new_flag_value: bool) -> ModelResult<()> {
        let zone = self.zone_at_mut(x, y).ok_or(ErrorKind::OutOfBounds)?;
        if zone.flagged == new_flag_value {
            return Err(ErrorKind::NoOp);
        }
        zone.flagged = new_flag_value;
        if new_flag_value {
            self.num_flagged += 1;
        } else {
            self.num_flagged -= 1;
        }
        Ok(())
    }

    pub fn is_revealed_at(&self, x: u32, y: u32) -> Option<bool> {
        self.zone_at(x, y).map(|z| z.revealed)
    }

    /**
     * returns:
     *  on success, returns a boolean indicating if the revealed zone
     *     contains a mine
     *  ErrorKind::NoOp indicates that the zone at the given
     *      coordinates has already been revealed
     */
    pub fn reveal_at(&mut self, x: u32, y: u32) -> ModelResult<bool> {
        let zone = self.zone_at_mut(x, y).ok_or(ErrorKind::OutOfBounds)?;
        if zone.revealed {
            Err(ErrorKind::NoOp)
        } else {
            zone.revealed = true;
            Ok(zone.has_mine)
        }
    }

    pub fn has_mine_at(&self, x: u32, y: u32) -> Option<bool> {
        self.zone_at(x, y).map(|z| z.has_mine)
    }

    /**
     * if the given coordinates are within the dimensions of the Field,
     * returns the number of positions adjacent to the given position
     * which contain a buried mine
     */
    pub fn mines_adjacent_to(&self, x: u32, y: u32) -> Option<u32> {
        self.zone_at(x, y).map(|z| z.adj_mine_count)
    }

    /**
     * Produces a vector containing all valid, in-bounds (x, y) coordinate pairs
     * that are adjacent to the given coordinates.
     * If include_diag is true, then diagonal adjacencies will be included.
     * TODO: expound
     */
    pub fn adjacent_positions(&self, x: u32, y: u32, include_diag: bool) -> Vec<(u32, u32)> {
        let x = x as i32;
        let y = y as i32;
        let mut positions;
        if include_diag {
            // there are at most 8 adjacent positions
            positions = Vec::with_capacity(8);
            for some_x in (x - 1)..=(x + 1) {
                for some_y in (y - 1)..=(y + 1) {
                    if some_x != x || some_y != y {
                        positions.push((some_x, some_y));
                    }
                }
            }
        } else {
            positions = vec![(x, y - 1), (x - 1, y), (x + 1, y), (x, y + 1)];
        }
        positions
            .into_iter()
            // safely convert back to u32
            .filter(|&(x, y)| x >= 0 && y >= 0)
            .map(|(x, y)| (x as u32, y as u32))
            // filter out coordinates that aren't in-bounds
            .filter(|&(x, y)| self.zone_at(x, y).is_some())
            .collect()
    }

    /**
     * Updates the adjacent mine counts for Zones in this Field.
     * Shouldn't be called more than once.
     */
    fn set_adj_counts(&mut self, mine_placements: HashSet<(u32, u32)>) {
        // reset all counts to 0.
        // for x in 0..self.width() {
        //     for y in 0..self.height() {
        //         self.grid[x][y].adj_mine_count = 0;
        //     }
        // }
        for (x, y) in mine_placements {
            for (adj_x, adj_y) in self.adjacent_positions(x, y, true) {
                let zone = &mut self.grid[adj_x as usize][adj_y as usize];
                // increment mine count
                zone.adj_mine_count += 1;
            }
        }
    }

    /**
     * TODO
     */
    fn zone_at(&self, x: u32, y: u32) -> Option<&Zone> {
        self.grid.get(x as usize)?.get(y as usize)
    }

    /**
     * TODO
     */
    fn zone_at_mut(&mut self, x: u32, y: u32) -> Option<&mut Zone> {
        self.grid.get_mut(x as usize)?.get_mut(y as usize)
    }

    /**
     * TODO
     */
    fn generate_placements(
        num_mines: u32,
        upper_x_bound: u32,
        upper_y_bound: u32,
    ) -> HashSet<(u32, u32)> {
        let num_mines = num_mines as usize;
        let mut rng = rand::thread_rng();
        let mut coordinates = HashSet::with_capacity(num_mines);
        while coordinates.len() < num_mines {
            let x = rng.gen_range(0, upper_x_bound);
            let y = rng.gen_range(0, upper_y_bound);
            coordinates.insert((x, y));
        }
        coordinates
    }

    /**
     * TODO
     */
    fn generate_grid(
        width: u32,
        height: u32,
        mine_placements: &HashSet<(u32, u32)>,
    ) -> Vec<Vec<Zone>> {
        let mut grid = Vec::with_capacity(width as usize);
        for x in 0..width {
            let mut column = Vec::with_capacity(height as usize);
            for y in 0..height {
                let has_mine = mine_placements.contains(&(x, y));
                column.push(Zone::new(has_mine));
            }
            grid.push(column);
        }
        grid
    }
}
