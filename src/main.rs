use std::{collections::HashSet, fmt};

#[derive(Clone, Copy)]
struct Cell {
    number: Option<u8>,
    given: bool,
    canidates: [u8; 9],
}
struct Puzzle {
    grid: [[Cell; 9]; 9],
}

impl Puzzle {
    fn parse(input: &str) -> Puzzle {
        // println!("Parsing <{}>", input);
        let mut grid: [[Cell; 9]; 9] = [[Cell {
            number: None,
            given: false,
            canidates: [0; 9],
        }; 9]; 9];

        for (i, line_str) in input.trim().split("\n").enumerate() {
            let trimmed = line_str.trim();
            if trimmed.len() == 0 {
                continue;
            }

            for (j, c) in trimmed.chars().enumerate() {
                match c.to_digit(10) {
                    Some(d) => {
                        grid[i][j] = Cell {
                            number: Some(d.try_into().unwrap()),
                            given: true,
                            canidates: [0; 9],
                        }
                    }
                    None => {}
                }
            }
        }

        Puzzle { grid }
    }

    /// Review every cell and assign the possible legal canidates based on lines and blocks only.
    fn init_canidates(&mut self) {
        println!(">init_canidates");
        for cell_index in 0..81 {
            let col = cell_index % 9;
            let row = (cell_index - col) / 9;
            let cell = self.grid[row][col];
            let block = col / 3 + (row / 3) * 3;

            let debug = row == 0 && col == 2 && block == 0;

            if debug {
                println!("DEBUGGING ENABLED\n\n\n");
            }

            println!(
                "   looking at cell #{:02} ({},{}) in block {}: {:?}",
                cell_index, row, col, block, cell.number
            );

            if cell.given {
                continue;
            }

            let mut cset: HashSet<u8> = HashSet::new();
            for p in 1..10 {
                cset.insert(p);
            }

            // Narrow canidates by block
            let mut forbidden = self.numbers_in_block(block);
            if debug {
                println!("Numbers in block #{}: {:?}", block, forbidden);
            }
            for f in forbidden.iter() {
                cset.remove(f);
            }

            // Narrow canidates by row
            forbidden = self.numbers_in_row(row);
            if debug {
                println!("Numbers in row #{}: {:?}", row, forbidden);
            }
            for f in forbidden.iter() {
                cset.remove(f);
            }

            // Narrow canidates by column
            forbidden = self.numbers_in_column(col);
            if debug {
                println!("Numbers in column #{}: {:?}", col, forbidden);
            }
            for f in forbidden.iter() {
                cset.remove(f);
            }

            let mut canidates: [u8; 9] = [0; 9];
            let mut sorted: Vec<u8> = cset.drain().collect();
            sorted.sort();
            for (c, canidate) in sorted.iter().enumerate() {
                canidates[c] = *canidate;
            }
            self.grid[row][col].canidates = canidates;
        }
        println!("<init_canidates");
    }

    /// The corresponding block in our grid. 0 thru 8, starting in top left.
    fn block(&self, b: usize) -> [[Cell; 3]; 3] {
        assert!(b < 9, "Invalid block number: {}", b);

        let origin_x = b % 3;
        let origin_y = (b - origin_x) / 3;

        println!(
            "Block {} backed by grid origin @ ({}, {})",
            b, origin_x, origin_y
        );

        let mut result: [[Cell; 3]; 3] = [[Cell {
            number: None,
            given: false,
            canidates: [0; 9],
        }; 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.grid[origin_y * 3 + i][origin_x * 3 + j];
            }
        }

        result
    }

    fn numbers_in_block(&self, b: usize) -> HashSet<u8> {
        let mut r: HashSet<u8> = HashSet::new();
        let block = self.block(b);

        for i in 0..3 {
            for j in 0..3 {
                match block[i][j].number {
                    Some(n) => {
                        r.insert(n);
                    }
                    None => {}
                }
            }
        }

        r
    }

    fn numbers_in_row(&self, row: usize) -> HashSet<u8> {
        let mut r: HashSet<u8> = HashSet::new();

        for i in 0..9 {
            match self.grid[row][i].number {
                Some(n) => {
                    r.insert(n);
                }
                None => {}
            }
        }

        r
    }

    fn numbers_in_column(&self, col: usize) -> HashSet<u8> {
        let mut r: HashSet<u8> = HashSet::new();

        for i in 0..9 {
            match self.grid[i][col].number {
                Some(n) => {
                    r.insert(n);
                }
                None => {}
            }
        }

        r
    }

    fn internals(&self) -> String {
        let mut r = String::new();

        for b in 0..9 {
            let block = self.block(b);
            r.push_str(format!("Block {}:\n", b).as_str());

            for i in 0..3 {
                for j in 0..3 {
                    let cell = block[i][j];

                    r.push_str(format!("    ({},{}) → ", i, j).as_str());

                    match cell.number {
                        Some(n) => r.push_str(n.to_string().as_str()),
                        None => {
                            let mut canidates: Vec<u8> = Vec::new();
                            for c in 0..9 {
                                if cell.canidates[c] > 0 {
                                    canidates.push(cell.canidates[c]);
                                }
                            }
                            r.push_str(format!("{:?}", canidates).as_str())
                        }
                    }
                    r.push_str("\n");
                }
            }
            r.push_str("\n");
        }

        r
    }
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display = "".to_string();

        for (i, row) in self.grid.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                match cell.number {
                    Some(num) => {
                        display.push_str(&num.to_string());
                    }
                    None => display.push_str("·"),
                }

                if j != 0 && (j + 1) % 3 == 0 {
                    display.push_str("  ");
                }
            }
            display.push_str("\n");

            if i != 0 && (i + 1) % 3 == 0 {
                display.push_str("\n");
            }
        }

        write!(f, "{}", display)
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut display = "".to_string();

        match self.number {
            Some(num) => {
                display.push_str(&num.to_string());
            }
            None => display.push_str("unknown"),
        }

        write!(f, "{}", display)
    }
}

pub fn read_stdin() -> Result<String, std::io::Error> {
    let mut buf = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
    Ok(buf)
}

fn main() -> Result<(), std::io::Error> {
    let input = &read_stdin()?;
    let mut puzzle = Puzzle::parse(input);
    puzzle.init_canidates();

    println!("Internals:\n{}", puzzle.internals());

    Ok(())
}

mod test {
    #[allow(unused_imports)] // wtf?
    use super::*;
    use std::{collections::HashSet, hash::Hash};

    #[allow(dead_code)] // wtf?
    const SAMPLE: &str = r#"
.4.5.2...
76....1.2
9...18.64
..429...8
.8.3.6.7.
6...754..
21.68...3
4.6....27
...4.9.1.
    "#;

    fn assert_eq_set(a: &HashSet<u8>, b: &[u8]) {
        let a: HashSet<_> = a.iter().collect();
        let b: HashSet<_> = b.iter().collect();

        assert!(a == b, "Sets do not match. Expected {:?}, found {:?}", b, a);
    }

    fn eq_slice(a: &[u8], b: &[u8]) -> bool {
        let mut a: HashSet<&u8> = a.iter().collect();
        let mut b: HashSet<&u8> = b.iter().collect();

        a.remove(&0);
        b.remove(&0);

        if a != b {
            println!("Slices do not match. Expected {:?}, found {:?}.", b, a);
        }

        a == b
    }

    #[test]
    fn baby_steps() {
        let mut puzzle = super::Puzzle::parse(SAMPLE);

        println!("Parsed puzzle:\n{}", puzzle);
        assert_eq_set(&puzzle.numbers_in_row(0), &[4, 5, 2]);
        assert_eq_set(&puzzle.numbers_in_row(1), &[7, 6, 1, 2]);
        assert_eq_set(&puzzle.numbers_in_row(2), &[9, 1, 8, 6, 4]);
        assert_eq_set(&puzzle.numbers_in_row(3), &[4, 2, 9, 8]);
        assert_eq_set(&puzzle.numbers_in_row(4), &[8, 3, 6, 7]);
        assert_eq_set(&puzzle.numbers_in_row(5), &[6, 7, 5, 4]);
        assert_eq_set(&puzzle.numbers_in_row(6), &[2, 1, 6, 8, 3]);
        assert_eq_set(&puzzle.numbers_in_row(7), &[4, 6, 2, 7]);
        assert_eq_set(&puzzle.numbers_in_row(8), &[4, 9, 1]);

        assert_eq_set(&puzzle.numbers_in_column(0), &[7, 9, 6, 2, 4]);
        assert_eq_set(&puzzle.numbers_in_column(1), &[4, 6, 8, 1]);
        assert_eq_set(&puzzle.numbers_in_column(2), &[4, 6]);
        assert_eq_set(&puzzle.numbers_in_column(3), &[5, 2, 3, 6, 4]);
        assert_eq_set(&puzzle.numbers_in_column(4), &[1, 9, 7, 8]);
        assert_eq_set(&puzzle.numbers_in_column(5), &[2, 8, 6, 5, 9]);
        assert_eq_set(&puzzle.numbers_in_column(6), &[1, 4]);
        assert_eq_set(&puzzle.numbers_in_column(7), &[6, 7, 2, 1]);
        assert_eq_set(&puzzle.numbers_in_column(8), &[2, 4, 8, 3, 7]);

        assert_eq_set(&puzzle.numbers_in_block(0), &[4, 7, 6, 9]);
        assert_eq_set(&puzzle.numbers_in_block(1), &[5, 2, 1, 8]);
        assert_eq_set(&puzzle.numbers_in_block(2), &[1, 2, 6, 4]);
        assert_eq_set(&puzzle.numbers_in_block(3), &[4, 8, 6]);
        assert_eq_set(&puzzle.numbers_in_block(4), &[2, 9, 3, 6, 7, 5]);
        assert_eq_set(&puzzle.numbers_in_block(5), &[8, 7, 4]);
        assert_eq_set(&puzzle.numbers_in_block(6), &[2, 1, 4, 6]);
        assert_eq_set(&puzzle.numbers_in_block(7), &[6, 8, 4, 9]);
        assert_eq_set(&puzzle.numbers_in_block(8), &[3, 2, 7, 1]);

        puzzle.init_canidates();

        // Block 0
        assert!(eq_slice(&puzzle.grid[0][0].canidates, &[1, 3, 8]));
        assert!(eq_slice(&puzzle.grid[0][2].canidates, &[1, 3, 8]));
        assert!(eq_slice(&puzzle.grid[1][2].canidates, &[3, 5, 8]));
        assert!(eq_slice(&puzzle.grid[2][1].canidates, &[2, 3, 5]));
        assert!(eq_slice(&puzzle.grid[2][2].canidates, &[2, 3, 5]));

        println!("Internals:\n{}", puzzle.internals());
    }
}
