use std::io;

fn main() {
    println!("Enter your starting Sudoku board, 9 digits per line");
    let mut board = Board::read_board();

    match board.check_possibles() {
        Ok(_) => println!("No errors"),
        Err(message) => {
            println!("Validation error: {message}")
        }
    }

    solve_puzzle(board);
}

#[derive(Debug, PartialEq, Eq)]
struct Cell {
    value: Option<usize>,
    possible: [bool; 9]
}

impl Cell {
    // value must be 1..9
    fn new(value: Option<usize>) -> Self
    {
        let mut possible = [true; 9];
        if let Some(val)= value {
            possible.iter_mut().enumerate().for_each(|(i,v)| *v= i==val-1);
        }
        Self { value, possible }
    }

    // value must be 1..9
    fn set_value(&mut self, value: usize)
    {
        self.value= Some(value);
        self.possible.iter_mut().enumerate().for_each(|(i,v)| *v= i==value-1);
    }

    // value must be 1..9
    fn is_possible(&self, value: usize) -> bool
    {
        self.possible[value-1]
    }

    // value must be 1..9
    fn not_possible(&mut self, value: usize)
    {
        self.possible[value-1] = false;
        // list of indexes into possibles where value is true
        let mut possibles = self.possible.iter().enumerate().filter(|(_,p)| **p).map(|(i,_)| i);
        // if there is only one possible left then it must be the value
        if let Some(i) = possibles.next() && possibles.next().is_none() {
            self.value = Some(i+1);
        }
    }
}

struct Board {
    cells: [[[[Cell; 3]; 3]; 3]; 3],
    moves: u16
}

impl Board {
    fn new() -> Self {
        Self {
            cells: std::array::from_fn(|_| {
                    std::array::from_fn(|_| {
                     std::array::from_fn(|_| std::array::from_fn(|_| Cell::new(None)))
                })
            }),
            moves: 0
        }
    }

    #[cfg(test)]
    fn from_grid(grid: [[usize; 9]; 9]) -> Self {
        let mut board = Self::new();
        for (row, values) in grid.iter().enumerate() {
            for (column, &value) in values.iter().enumerate() {
                if value != 0 {
                    board.set_value(
                        row / 3,
                        column / 3,
                        row % 3,
                        column % 3,
                        value,
                    );
                }
            }
        }
        board
    }

    // x and y values must be 0..2
    fn get_value(&self, quad_x: usize, quad_y: usize, cell_x: usize, cell_y: usize) -> Option<usize>
    {
        self.cells[quad_x][quad_y][cell_x][cell_y].value
    }
    // x and y values must be 0..2
    fn set_value(&mut self, quad_x: usize, quad_y: usize, cell_x: usize, cell_y: usize, value: usize)
    {
        self.cells[quad_x][quad_y][cell_x][cell_y].set_value(value);
    }

    fn is_solved(&self) -> bool {
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                for quad_y in 0..3 {
                    for cell_y in 0..3 {
                        if self.cells[quad_x][quad_y][cell_x][cell_y].value.is_none() {
                            return false;
                        }
                    }
                }
            }
        }
        return true;
    }

    fn check_possibles(&mut self) -> Result<(), String> {
        // check quads
        for quad_x in 0..3 {
            for quad_y in 0..3 {
                let mut quad: [&mut Cell; 9] = self.cells[quad_x][quad_y]
                    .iter_mut()
                    .flatten()
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a quadrant has exactly 9 cells");
                self.moves += Board::check_set(&mut quad, format!("quad {}", quad_x*3+quad_y+1))?;
                self.moves += Board::pairs_and_triplets(quad);
            }
        }
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                let mut r: [&mut Cell; 9] = self.cells[quad_x]
                    .iter_mut()
                    .flat_map(|col| col[cell_x].iter_mut())
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a row has exactly 9 cells");
                self.moves += Board::check_set(&mut r, format!("row {}", quad_x*3+cell_x+1))?;
                self.moves += Board::pairs_and_triplets(r);
            }
        }
        for quad_y in 0..3 {
            for cell_y in 0..3 {
                let mut c: [&mut Cell; 9] = self.cells
                    .iter_mut()
                    .flat_map(|r| {
                        r[quad_y].iter_mut()
                            .map(|r| &mut r[cell_y])
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a column has exactly 9 cells");
                self.moves += Board::check_set(&mut c, format!("column {}", quad_y*3+cell_y+1))?;
                self.moves += Board::pairs_and_triplets(c)
            }
        }
        Ok(())
    }

    // returns number of changes made, or error if inconsistency
    fn check_set(cells: &mut [&mut Cell; 9], set_name: String) -> Result<u16, String> {
        let mut changes:u16 = 0;
        for cell in 0..9 {
            let mut unique_possibilities = if cells[cell].value.is_none() { cells[cell].possible.clone() } else { [false; 9] };
            for other in (0..9).filter(|&o| o != cell) {
                // check for inconsistency
                if let Some(c1) = cells[cell].value && let Some(c2) = cells[other].value && c1==c2 {
                    return Err(format!("In {set_name}, cell {} and cell {} are both {c1}",cell+1,other+1));
                }
                // current cell cannot be the same as other cell
                if cells[cell].value.is_none() && let Some(c2) = cells[other].value && cells[cell].is_possible(c2) {
                    cells[cell].not_possible(c2);
                    changes+=1;
                }
                // possibility is not unique if other cell can also be it
                for (unique_p, other_p) in unique_possibilities.iter_mut().zip(cells[other].possible) {
                    *unique_p &= !other_p
                }
            }
            // if there is a unique possibility, then it must be the value
            let mut uniques = unique_possibilities.iter().enumerate().filter(|(_,p)| **p).map(|(i,_)| i);
            if let Some(i)= uniques.next() {
                if uniques.next().is_some() {
                    return Err(format!("In {set_name}, there are multiple values that only cell {} can be", cell+1))
                }
                cells[cell].set_value(i+1);
                changes+=1;
            }
        }
        Ok(changes)
    }

    fn pairs_and_triplets(cells: [&mut Cell; 9]) -> u16 {
        let mut count = 0;
        for x in 0..9 {
            if cells[x].value.is_none() {
                for y in x+1..9 {
                    if cells[y].value.is_none() {
                        let mut pair_unique = cells[x].possible.clone();
                        for (p1,p2) in pair_unique.iter_mut().zip(cells[y].possible) {
                            *p1 |= p2;
                        }
                        // if only 2 possibilities, remove them from all other cells in this set
                        if pair_unique.iter().filter(|p| **p).count()==2 {
                            for p in (0..9).filter(|p| pair_unique[*p]) {
                                for c in (0..9).filter(|c| *c!=x && *c!=y) {
                                    if cells[c].value.is_none() && cells[c].possible[p] {
                                        cells[c].not_possible(p+1);
                                        count+=1;
                                    }
                                }
                            }
                        }
                        // if 3 possibilities and there is a third cell with the same three, remove them from all other cells in this set
                        else if pair_unique.iter().filter(|p| **p).count()==3 {
                            for z in y+1..9 {
                                if cells[z].value.is_none() {
                                    let mut trio_unique = pair_unique.clone();
                                    for (p1,p2) in trio_unique.iter_mut().zip(cells[z].possible) {
                                        *p1 |= p2;
                                    }
                                    if trio_unique.iter().filter(|p| **p).count()==3 {
                                        for p in (0..9).filter(|p| trio_unique[*p]) {
                                            for c in (0..9).filter(|c| *c!=x && *c!=y && *c!=z) {
                                                if cells[c].value.is_none() && cells[c].possible[p] {
                                                    cells[c].not_possible(p+1);
                                                    count+=1;
                                                }
                                            }
                                        }
                                    }
                                    // now looking for quadruples
                                    for q in z+1..9 {
                                        if cells[q].value.is_none() {
                                            let mut quad_unique = trio_unique.clone();
                                            for (p1,p2) in quad_unique.iter_mut().zip(cells[q].possible) {
                                                *p1 |= p2;
                                            }
                                            if quad_unique.iter().filter(|p| **p).count()==4 {
                                                for p in (0..9).filter(|p| quad_unique[*p]) {
                                                    for c in (0..9).filter(|c| *c!=x && *c!=y && *c!=z && *c!=q) {
                                                        if cells[c].value.is_none() && cells[c].possible[p] {
                                                            cells[c].not_possible(p+1);
                                                            count+=1;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        count
    }

    fn find_intersections(&mut self) {
        for quad_x in 0..3 {
            for quad_y in 0..3 {
                // intersection of quad and row
                for cell_x in 0..3 {
                    let mut intersection: Vec<&mut Cell> = Vec::new();
                    let mut quad_others: Vec<&mut Cell> = Vec::new();
                    let mut line_others: Vec<&mut Cell> = Vec::new();
                    for (qy, quad) in self.cells[quad_x].iter_mut().enumerate() {
                        for (x, cell_row) in quad.iter_mut().enumerate() {
                            for cell in cell_row.iter_mut() {
                                if qy == quad_y && x == cell_x {
                                    intersection.push(cell);
                                } else if qy == quad_y {
                                    quad_others.push(cell);
                                } else if x == cell_x {
                                    line_others.push(cell);
                                }
                            }
                        }
                    }
                    self.moves += Board::handle_intersection(
                        intersection.try_into().expect("intersection has exactly 3 cells"),
                        quad_others.try_into().expect("quad_others has exactly 6 cells"),
                        line_others.try_into().expect("line_others has exactly 6 cells")
                    );
                }

                // intersection of quad and column
                for cell_y in 0..3 {
                    let mut intersection: Vec<&mut Cell> = Vec::new();
                    let mut quad_others: Vec<&mut Cell> = Vec::new();
                    let mut line_others: Vec<&mut Cell> = Vec::new();
                    for (qx, quad) in self.cells.iter_mut().map(|c| &mut c[quad_y]).enumerate() {
                        for cell_col in quad.iter_mut() {
                            for (y, cell) in cell_col.iter_mut().enumerate() {
                                if qx == quad_x && y == cell_y {
                                    intersection.push(cell);
                                } else if qx == quad_x {
                                    quad_others.push(cell);
                                } else if y == cell_y {
                                    line_others.push(cell);
                                }
                            }
                        }
                    }
                    self.moves += Board::handle_intersection(
                        intersection.try_into().expect("intersection has exactly 3 cells"),
                        quad_others.try_into().expect("quad_others has exactly 6 cells"),
                        line_others.try_into().expect("line_others has exactly 6 cells")
                    );
                }
            }
        }
    }

    fn handle_intersection(intersection: [&mut Cell; 3], mut quad_others: [&mut Cell; 6], mut line_others: [&mut Cell; 6]) -> u16
    {
        let mut changes = 0;

        // a number that is only possible in intersection clears its possibilities from the other set
        let mut intersection_possibles = [false; 9];
        for i in 0..3 {
            for (p1,p2) in intersection_possibles.iter_mut().zip(intersection[i].possible) {
                *p1 |= p2;
            }
        }

        let mut unique_to_quad = intersection_possibles.clone();
        for i in 0..6 {
            for (p1,p2) in unique_to_quad.iter_mut().zip(quad_others[i].possible) {
                *p1 &= !p2;
            }
        }
        changes += Board::remove_these_possibilities(unique_to_quad, &mut line_others);

        let mut unique_to_line = intersection_possibles.clone();
        for i in 0..6 {
            for (p1,p2) in unique_to_line.iter_mut().zip(line_others[i].possible) {
                *p1 &= !p2;
            }
        }
        changes += Board::remove_these_possibilities(unique_to_line, &mut quad_others);
        changes
    }

    fn remove_these_possibilities(remove: [bool; 9], cell: &mut [&mut Cell;6]) -> u16 {
        let mut changes = 0;
        for i in 0..9 {
            if remove[i] {
                for c in 0..6 {
                    if cell[c].is_possible(i+1) {
                        cell[c].not_possible(i+1);
                        changes+=1;
                    }
                }
            }
        }
        changes
    }

    fn read_board() -> Board {
        let mut board = Board::new();
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                let mut line = String::new();
                io::stdin().read_line(&mut line).expect("bad input");
                for quad_y in 0..3 {
                    for cell_y in 0..3 {
                        let d = line.chars().nth(quad_y*3 + cell_y).and_then(|c| c.to_digit(10));
                        if let Some(i) = d && i != 0 {
                            board.set_value(quad_x, quad_y, cell_x, cell_y, i as usize);
                        }
                    }
                }
            }
        }
        board
    }
    fn print_board(&self)
    {
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                for quad_y in 0..3 {
                    for cell_y in 0..3 {
                        let val = self.get_value(quad_x,quad_y,cell_x,cell_y);
                        if let Some(i)=val {
                            print!("{}", i);
                        } else {
                            print!(".");
                        }
                    }
                    if quad_y < 2 {
                        print!("|");
                    }
                }
                println!();
            }
            if quad_x < 2 {
                println!("---+---+---");
            }
        }
    }

    fn debug_board(&self)
    {
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                for quad_y in 0..3 {
                    for cell_y in 0..3 {
                        let val = self.get_value(quad_x,quad_y,cell_x,cell_y);
                        if let Some(i)=val {
                            print!(" *{}* ", i);
                        } else {
                            print!("[");
                            for i in 1..=9 {
                                if self.cells[quad_x][quad_y][cell_x][cell_y].is_possible(i) {
                                    print!("{}",i);
                                }
                            }
                            print!("]");
                        }
                    }
                    if quad_y < 2 {
                        print!("|");
                    }
                }
                println!();
            }
            if quad_x < 2 {
                println!("---------------+---------------+---------------");
            }
        }
    }
}

fn solve_puzzle(mut board: Board) {
    let mut i = 0;
    loop {
        board.print_board();
        let moves = board.moves;

        if let Err(message) = board.check_possibles() {
            println!("Validation error: {message}");
            break;
        }
        board.find_intersections();

        if board.is_solved() {
            println!(">>>>>>>>>>> SOLVED after {i} iterations and {moves} changes");
            board.print_board();
            break;
        }
        if board.moves==moves {
            println!(">>>>>>>>>>> stuck after {i} iterations and {moves} changes");
            board.debug_board();
            break;
        }

        println!(">>>>>>>>>>> {i} iterations and {moves} changes so far");
        i+=1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_puzzle() {
        let board = Board::from_grid([
            [0,0,8,0,7,0,0,0,0],
            [0,0,5,9,0,0,3,0,0],
            [0,4,0,0,0,1,0,6,9],
            [0,0,6,0,0,0,0,9,0],
            [1,0,0,0,0,0,0,0,5],
            [0,8,0,0,0,0,2,0,0],
            [3,6,0,4,0,0,0,7,0],
            [0,0,7,0,0,8,6,0,0],
            [0,0,0,0,2,0,4,0,0]
        ]);
        solve_puzzle(board);
    }

    #[test]
    fn harder_puzzle() {
        let board = Board::from_grid([
            [0,0,0,5,0,0,0,3,1],
            [4,0,0,0,8,0,7,0,0],
            [9,0,6,0,1,0,0,0,0],
            [0,0,0,0,0,3,0,6,0],
            [0,0,4,0,0,0,9,0,0],
            [0,5,0,4,0,0,0,0,0],
            [0,0,0,0,4,0,1,0,8],
            [0,0,8,0,3,0,0,0,7],
            [2,7,0,0,0,5,0,0,0]
        ]);
        solve_puzzle(board);
    }
}
