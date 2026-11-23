use std::io;

fn main() {
    println!("Hello, world!");
    let mut board = Board::read_board();
    board.print_board();
    match board.check_possibles() {
        Ok(_) => println!("No errors"),
        Err(message) => println!("Validation error: {message}")
    }
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
        self.possible[usize::from(value-1)]
    }

    // value must be 1..9
    fn not_possible(&mut self, value: usize)
    {
        self.possible[usize::from(value-1)] = false;
    }
}

struct Board {
    cells: [[[[Cell; 3]; 3]; 3]; 3]
}

impl Board {
    fn new() -> Self {
        Self {
            cells: std::array::from_fn(|_| {
                    std::array::from_fn(|_| {
                     std::array::from_fn(|_| std::array::from_fn(|_| Cell::new(None)))
                })
            }),
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
        for quad_y in 0..3 {
            for quad_x in 0..3 {
                let quad: [&mut Cell; 9] = self.cells[quad_x][quad_y]
                    .iter_mut()
                    .flatten()
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a quadrant has exactly 9 cells");
                Board::check_set(quad, format!("quad {}", quad_y*3+quad_x+1))?;
            }
        }
        for quad_x in 0..3 {
            for cell_x in 0..3 {
                let column: [&mut Cell; 9] = self.cells[quad_x]
                    .iter_mut()
                    .flat_map(|col| col[cell_x].iter_mut())
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a column has exactly 9 cells");
                Board::check_set(column, format!("column {}", quad_x*3+cell_x+1))?;
            }
        }
        for quad_y in 0..3 {
            for cell_y in 0..3 {
                let row: [&mut Cell; 9] = self.cells
                    .iter_mut()
                    .flat_map(|r| {
                        r[quad_y].iter_mut()
                            .map(|r| &mut r[cell_y])
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("a row has exactly 9 cells");
                Board::check_set(row, format!("row {}", quad_y*3+cell_y+1))?;
            }
        }
        Ok(())
    }

    fn check_set(cells: [&mut Cell; 9], set_name: String) -> Result<(), String> {
        for cell in 0..9 {
            for other in (0..9).filter(|&o| o != cell) {
                if let Some(c1) = cells[cell].value && let Some(c2) = cells[other].value && c1==c2 {
                    return Err(format!("In {set_name}, cell {} and cell {} are both {c1}",cell+1,other+1));
                }
            }
        }
        Ok(())
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
}
