use std::io;

enum Solution {
    Invalid(String, u16),
    Stuck(u16),
    Success(u16)
}

fn main() {
    println!("Enter your starting Sudoku board, 9 digits per line");
    let mut board = Board::read_board();

    match board.solve_puzzle() {
        Solution::Invalid(message, 0) => {
            println!("Invalid puzzle: {message}");
            board.print_board()
        }
        Solution::Invalid(message, iterations) => {
            println!("Possible bug: invalid puzzle after {iterations} iterations: {message}");
            board.debug_board();
        }
        Solution::Stuck(iterations) => {
            println!("Stuck after {iterations} iterations");
            board.debug_board();
        }
        Solution::Success(iterations) => {
            println!("Solved puzzle after {iterations} iterations");
            board.print_board();
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Clone)]
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

    fn solve_puzzle(&mut self) -> Solution {
        let mut i = 0;
        loop {
            self.print_board();
            let moves = self.moves;

            if let Err(message) = self.apply_to_sets(Self::check_set) {
                return Solution::Invalid(message, i);
            }
            i+=1;

            if let Err(message) = self.apply_to_sets(Self::pairs_and_triplets) {
                return Solution::Invalid(message, i);
            }

            if self.is_solved() {
                return Solution::Success(i)
            }
            if self.moves > moves {
                println!(">>>>>>>>>>> Making progress after {i} iterations and {} changes", self.moves);
                continue;
            }

            // less common operations
            self.find_intersections();
            if self.moves > moves {
                println!(">>>>>>>>>>> Required find_intersections after {i} iterations to make {} moves", self.moves - moves);
                if self.is_solved() {
                    return Solution::Success(i)
                }
                continue;
            }

            if let Err(message) = self.apply_to_sets(Self::naked_subsets) {
                return Solution::Invalid(message, i);
            }
            if self.moves > moves {
                println!(">>>>>>>>>>> Required naked_subsets function after {i} iterations to make {} moves", self.moves - moves);
                if self.is_solved() {
                    return Solution::Success(i)
                }
                continue;
            }

            if let Ok(solution) = self.guess_and_check() {
                *self = solution;
                println!(">>>>>>>>>>> Required guessing and checking to make {} moves", self.moves - moves);
                return Solution::Success(i)
            } else {
                return Solution::Stuck(i);
            }
        }
    }

    fn apply_to_sets(&mut self, rule: fn(&mut [&mut Cell; 9], String) -> Result<u16, String>) -> Result<(), String>
    {
        // check quads
        for quad_x in 0..3 {
            for quad_y in 0..3 {
                let mut quad: [&mut Cell; 9] = self.cells[quad_x][quad_y]
                    .iter_mut()
                    .flatten()
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a quadrant has exactly 9 cells");
                self.moves += rule(&mut quad, format!("quad {}", quad_x*3+quad_y+1))?;
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
                self.moves += rule(&mut r, format!("row {}", quad_x*3+cell_x+1))?;
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
                self.moves += rule(&mut c, format!("column {}", quad_y*3+cell_y+1))?;
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

    // look for naked pairs or triples
    fn pairs_and_triplets(cells: &mut [&mut Cell; 9], _ : String) -> Result<u16, String> {
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
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    // AI-generated function to generalize pairs_and_triplets
    // It's educational, but doesn't seem to solve any additional puzzles
    fn naked_subsets(cells: &mut [&mut Cell; 9], _ : String) -> Result<u16, String> {
        let mut changes = 0;

        // Try subsets of 2 through 6 cells, smaller subsets first.
        for size in 2..=6 {
            for subset in 0u16..(1 << 9) {
                if subset.count_ones() != size {
                    continue;
                }

                let mut possible = [false; 9];
                let mut valid = true;

                // Find the union of candidates for the selected cells.
                for (i, cell) in cells.iter().enumerate() {
                    if subset & (1 << i) == 0 {
                        continue;
                    }

                    if cell.value.is_some() {
                        valid = false;
                        break;
                    }

                    for (union, candidate) in possible.iter_mut().zip(cell.possible) {
                        *union |= candidate;
                    }
                }

                if !valid || possible.iter().filter(|&&p| p).count() != size as usize {
                    continue;
                }

                // Remove those candidates from all other unsolved cells.
                for (i, cell) in cells.iter_mut().enumerate() {
                    if subset & (1 << i) != 0 || cell.value.is_some() {
                        continue;
                    }

                    for (candidate, &remove) in possible.iter().enumerate() {
                        if remove && cell.possible[candidate] {
                            cell.not_possible(candidate + 1);
                            changes += 1;
                        }
                    }
                }
            }
        }

        Ok(changes)
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

    fn guess_and_check(&mut self) -> Result<Board, ()> {
        use std::sync::mpsc;
        use std::thread;
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                for quad_y in 0..3 {
                    for cell_y in 0..3 {
                        if self.get_value(quad_x,quad_y,cell_x,cell_y).is_none() {
                            let (guesser, receiver) = mpsc::channel();
                            for attempt in 1..=9 {
                                if self.cells[quad_x][quad_y][cell_x][cell_y].is_possible(attempt) {
                                    let guesser = guesser.clone();
                                    let mut board_guess = self.clone();
                                    thread::spawn(move || {
                                        println!(">>>>>>>>>>> Guessing {} in quadrant {} cell {}", attempt, quad_x*3+quad_y+1, cell_x*3+cell_y+1);
                                        board_guess.set_value(quad_x, quad_y, cell_x, cell_y, attempt);
                                        match board_guess.solve_puzzle() {
                                            Solution::Success(_) => {
                                                println!(">>>>>>>>>>> Guessing {} in quadrant {} cell {} succeeded", attempt, quad_x*3+quad_y+1, cell_x*3+cell_y+1);
                                                let _ = guesser.send(Ok(board_guess));
                                            }
                                            Solution::Invalid(_,_) => {
                                                println!(">>>>>>>>>>> Disproved {} in quadrant {} cell {}", attempt, quad_x*3+quad_y+1, cell_x*3+cell_y+1);
                                                let _ = guesser.send(Err(()));
                                            }
                                            Solution::Stuck(_) => {
                                                let _ = guesser.send(Err(()));
                                            }
                                        }
                                    });
                                }
                            }
                            drop(guesser);
                            let mut new_board: Option<Board> = None;
                            for result in receiver {
                                if let Ok(result_board) = result {
                                    new_board = Some(result_board);
                                }
                            };
                            if let Some(new_board) = new_board {
                                return Ok(new_board);
                            }
                        }
                    }
                }
            }
        }
        Err(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_puzzle() {
        let mut board = Board::from_grid([
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
        assert!(matches!(board.solve_puzzle(), Solution::Success(_)));
    }

    #[test]
    fn harder_puzzle() {
        let mut board = Board::from_grid([
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
        assert!(matches!(board.solve_puzzle(), Solution::Success(_)));
    }
}
