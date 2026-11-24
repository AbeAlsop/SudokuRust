use std::io;

fn main() {
    println!("Enter your starting Sudoku board, 9 digits per line");
    let mut board = Board::read_board();
    board.print_board();
    let mut moves = board.moves;
    match board.check_possibles() {
        Ok(_) => println!("No errors"),
        Err(message) => {
            board.print_board();
            println!("Validation error: {message}")
        }
    }
    while moves != board.moves {
        moves = board.moves;
        println!(">>>>>>>>>>> {moves} moves so far");
        board.print_board();
        if let Err(message) = board.check_possibles() {
            println!("Validation error: {message}");
        }
    }
    println!(">>>>>>>>>>> Final:");
    board.debug_board();
}

#[derive(Debug)]
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

    fn check_possibles(&mut self) -> Result<(), String> {
        // check quads
        for quad_x in 0..3 {
            for quad_y in 0..3 {
                let quad: [&mut Cell; 9] = self.cells[quad_x][quad_y]
                    .iter_mut()
                    .flatten()
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a quadrant has exactly 9 cells");
                self.moves += Board::check_set(quad, format!("quad {}", quad_x*3+quad_y+1))?;
            }
        }
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                let r: [&mut Cell; 9] = self.cells[quad_x]
                    .iter_mut()
                    .flat_map(|col| col[cell_x].iter_mut())
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a row has exactly 9 cells");
                self.moves += Board::check_set(r, format!("row {}", quad_x*3+cell_x+1))?;
            }
        }
        for quad_y in 0..3 {
            for cell_y in 0..3 {
                let c: [&mut Cell; 9] = self.cells
                    .iter_mut()
                    .flat_map(|r| {
                        r[quad_y].iter_mut()
                            .map(|r| &mut r[cell_y])
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a column has exactly 9 cells");
                self.moves += Board::check_set(c, format!("column {}", quad_y*3+cell_y+1))?;
            }
        }
        Ok(())
    }

    // returns number of changes made, or error if inconsistency
    fn check_set(cells: [&mut Cell; 9], set_name: String) -> Result<u16, String> {
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
                println!("---+---+---");
            }
        }
    }
}
