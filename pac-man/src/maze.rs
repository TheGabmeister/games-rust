use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cell {
    Wall,
    Path,
    Gate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pellet {
    None,
    Dot,
    Power,
}

#[derive(Clone)]
pub(crate) struct Maze {
    pub(crate) width: i32,
    pub(crate) height: i32,
    cells: Vec<Cell>,
    pellets: Vec<Pellet>,
    pub(crate) warp_row: i32,
}

impl Maze {
    pub(crate) fn from_layout(layout: &[&str], warp_row: i32) -> Self {
        let height = layout.len() as i32;
        let width = layout[0].chars().count() as i32;

        let mut cells = Vec::with_capacity((width * height) as usize);
        let mut pellets = Vec::with_capacity((width * height) as usize);

        for row in layout {
            debug_assert_eq!(row.chars().count() as i32, width);
            for ch in row.chars() {
                match ch {
                    '#' => {
                        cells.push(Cell::Wall);
                        pellets.push(Pellet::None);
                    }
                    '.' => {
                        cells.push(Cell::Path);
                        pellets.push(Pellet::Dot);
                    }
                    'o' => {
                        cells.push(Cell::Path);
                        pellets.push(Pellet::Power);
                    }
                    '-' => {
                        cells.push(Cell::Gate);
                        pellets.push(Pellet::None);
                    }
                    _ => {
                        cells.push(Cell::Path);
                        pellets.push(Pellet::None);
                    }
                }
            }
        }

        Self {
            width,
            height,
            cells,
            pellets,
            warp_row,
        }
    }

    fn index(&self, tile: IVec2) -> usize {
        (tile.y * self.width + tile.x) as usize
    }

    pub(crate) fn in_bounds(&self, tile: IVec2) -> bool {
        tile.x >= 0 && tile.x < self.width && tile.y >= 0 && tile.y < self.height
    }

    pub(crate) fn cell(&self, tile: IVec2) -> Option<Cell> {
        if !self.in_bounds(tile) {
            return None;
        }
        Some(self.cells[self.index(tile)])
    }

    pub(crate) fn pellet_at(&self, tile: IVec2) -> Pellet {
        if !self.in_bounds(tile) {
            return Pellet::None;
        }
        self.pellets[self.index(tile)]
    }

    pub(crate) fn take_pellet(&mut self, tile: IVec2) -> Pellet {
        if !self.in_bounds(tile) {
            return Pellet::None;
        }
        let idx = self.index(tile);
        let p = self.pellets[idx];
        self.pellets[idx] = Pellet::None;
        p
    }

    pub(crate) fn pellets_left(&self) -> usize {
        self.pellets
            .iter()
            .filter(|p| matches!(p, Pellet::Dot | Pellet::Power))
            .count()
    }

    pub(crate) fn passable_for_pacman(&self, tile: IVec2) -> bool {
        if !self.in_bounds(tile) {
            return tile.y == self.warp_row;
        }
        !matches!(self.cells[self.index(tile)], Cell::Wall | Cell::Gate)
    }

    pub(crate) fn passable_for_ghost(&self, tile: IVec2) -> bool {
        if !self.in_bounds(tile) {
            return tile.y == self.warp_row;
        }
        !matches!(self.cells[self.index(tile)], Cell::Wall)
    }

    pub(crate) fn reset_pellets(&mut self, layout: &[&str]) {
        *self = Maze::from_layout(layout, self.warp_row);
    }

    pub(crate) fn is_tunnel_tile(&self, tile: IVec2) -> bool {
        tile.y == self.warp_row && (tile.x <= 2 || tile.x >= self.width - 3)
    }
}
