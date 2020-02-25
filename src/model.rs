//#![allow(dead_code)]

use rand::Rng;
use std::collections::BTreeSet as Set;

pub type MinesweeperModel = Field;

struct Zone {
    flagged: bool,
    revealed: bool,
    has_mine: bool,
    adj_mine_count: usize,
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

    pub fn flagged(&self) -> bool {
        self.flagged
    }

    pub fn set_flagged(&mut self, value: bool) {
        self.flagged = value;
    }

    pub fn revealed(&self) -> bool {
        self.revealed
    }

    pub fn reveal(&mut self) {
        self.revealed = true;
    }

    pub fn has_mine(&self) -> bool {
        self.has_mine
    }

    pub fn adj_mine_count(&self) -> usize {
        self.adj_mine_count
    }

    pub fn set_adj_mine_count(&mut self, value: usize) {
        assert!(value <= 8);
        self.adj_mine_count = value;
    }

    pub fn exploded(&self) -> bool {
        self.has_mine && self.revealed
    }
}

pub struct Field {
    num_mines: usize,
    num_flagged: usize,
    num_correctly_flagged: usize,
    exploded: bool,
    grid: Vec<Vec<Zone>>,
}

impl Field {
    /**
     * Create a new Field. width and height must be greater than 0.
     */
    pub fn new(width: usize, height: usize, num_mines: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        let placements = Self::generate_placements(num_mines, width, height);
        let grid = Self::generate_grid(width, height, &placements);
        let mut freshly_made = Field {
            num_mines,
            num_flagged: 0,
            num_correctly_flagged: 0,
            exploded: false,
            grid,
        };
        freshly_made.set_counts(&placements);
        freshly_made
    }

    /**
     * The height of this Field
     */
    pub fn height(&self) -> usize {
        self.grid.len()
    }

    /**
     * The width of this Field
     */
    pub fn width(&self) -> usize {
        self.grid[0].len()
    }

    /**
     * The number of mines buried in this Field
     */
    pub fn num_mines(&self) -> usize {
        self.num_mines
    }

    /**
     * The total number of flags planted in Zones on this Field
     */
    pub fn num_flagged(&self) -> usize {
        self.num_flagged
    }

    /**
     * The number of flags placed on this Zones in this Field
     * in which mines are buried
     */
    pub fn num_correctly_flagged(&self) -> usize {
        self.num_correctly_flagged
    }

    /**
     * True if the game has been lost
     */
    pub fn exploded(&self) -> bool {
        self.exploded
    }

    /**
     * TODO
     */
    pub fn explode(&mut self) {
        self.exploded = true;
    }

    /**
     * Produces a vector containing all valid, in-bounds (x, y) coordinate pairs
     * that are adjacent to the given coordinates.
     * If include_diag is true, then diagonal adjacencies will be included.
     * TODO: expound
     */
    pub fn adjacent_positions(
        &self,
        x: usize,
        y: usize,
        include_diag: bool,
    ) -> Vec<(usize, usize)> {
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
            // filter out coordinates that aren't in-bounds
            .filter(|(x, y)| self.get_zone_at(*x, *y).is_some())
            .collect()
    }

    /**
     * Flag the zone at the given coordinates. If the zone at those
     * coordinates is already flagged, nothing happens.
     * Returns Err if given coordinates were out of bounds, else Ok
     */
    pub fn flag_zone(&mut self, x: usize, y: usize) -> Result<(), ()> {
        let zone = match self.get_zone_at_mut(x, y) {
            Some(z) => z,
            None => return Err(()),
        };
        if !zone.flagged() {
            zone.set_flagged(true);
            if zone.has_mine() {
                self.num_correctly_flagged += 1;
            }
            self.num_flagged += 1;
        }
        Ok(())
    }

    /**
     * Unflag the zone at the given coordinates. If the zone at those
     * coordinates is already not flagged, nothing happens.
     * Returns Err if given coordinates were out of bounds, else Ok
     */
    pub fn unflag_zone(&mut self, x: usize, y: usize) -> Result<(), ()> {
        let zone = match self.get_zone_at_mut(x, y) {
            Some(z) => z,
            None => return Err(()),
        };
        if zone.flagged() {
            zone.set_flagged(false);
            if zone.has_mine() {
                self.num_correctly_flagged -= 1;
            }
            self.num_flagged -= 1;
        }
        Ok(())
    }

    /**
     * TODO!
     */
    pub fn reveal_zone(&mut self, x: usize, y: usize) -> Result<(), ()> {
        let zone = match self.get_zone_at_mut(x, y) {
            Some(z) => z,
            None => return Err(()),
        };
        if !zone.revealed() {
            zone.reveal();
            if zone.has_mine() {
                self.explode();
            } else {
                if zone.adj_mine_count() == 0 {
                    for (adj_x, adj_y) in self.adjacent_positions(x, y, false) {
                        let adj_zone = &mut self.grid[adj_x][adj_y];
                        if !adj_zone.has_mine() && adj_zone.adj_mine_count() == 0 {
                            self.reveal_zone(adj_x, adj_y).unwrap();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /**
     * Updates the adjacent mine counts for Zones in this Field.
     * Shouldn't need to be called more than once.
     */
    fn set_counts(&mut self, mine_placements: &Set<(usize, usize)>) {
        // reset all counts to 0.
        for x in 0..self.width() {
            for y in 0..self.height() {
                self.grid[x][y].set_adj_mine_count(0);
            }
        }
        for (x, y) in mine_placements {
            for (adj_x, adj_y) in self.adjacent_positions(*x, *y, true) {
                let zone = &mut self.grid[adj_x][adj_y];
                // increment mine count
                zone.set_adj_mine_count(zone.adj_mine_count() + 1);
            }
        }
    }

    /**
     * TODO
     */
    fn get_zone_at(&self, x: usize, y: usize) -> Option<&Zone> {
        self.grid.get(x)?.get(y)
    }

    /**
     * TODO
     */
    fn get_zone_at_mut(&mut self, x: usize, y: usize) -> Option<&mut Zone> {
        self.grid.get_mut(x)?.get_mut(y)
    }

    /**
     * TODO
     */
    fn generate_placements(
        num_mines: usize,
        upper_x_bound: usize,
        upper_y_bound: usize,
    ) -> Set<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let mut coordinates = Set::new();
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
        width: usize,
        height: usize,
        mine_placements: &Set<(usize, usize)>,
    ) -> Vec<Vec<Zone>> {
        let mut columns = Vec::with_capacity(width);
        for x in 0..width {
            let mut col = Vec::with_capacity(height);
            for y in 0..height {
                col.push(Zone::new(mine_placements.contains(&(x, y))));
            }
            columns.push(col);
        }
        columns
    }
}
