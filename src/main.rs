use std::{collections::HashSet, fmt, fs::File, io::Write};

#[derive(Clone, Copy)]
struct Cell {
    number: Option<u8>,
    given: bool,
    candidates: [u8; 9],
}
struct Puzzle {
    iteration: usize,
    grid: [[Cell; 9]; 9],
}

impl Cell {
    fn candidates_as_vec(&self) -> Vec<u8> {
        let mut r: Vec<u8> = Vec::new();
        for i in 0..9 {
            if self.candidates[i] > 0 {
                r.push(self.candidates[i]);
            }
        }
        r
    }
}

impl Puzzle {
    fn parse(input: &str) -> Puzzle {
        // println!("Parsing <{}>", input);
        let mut grid: [[Cell; 9]; 9] = [[Cell {
            number: None,
            given: false,
            candidates: [0; 9],
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
                            candidates: [0; 9],
                        }
                    }
                    None => {}
                }
            }
        }

        Puzzle { iteration: 0, grid }
    }

    fn is_solved(&self) -> bool {
        for row in 0..9 {
            for col in 0..9 {
                match self.grid[row][col].number {
                    Some(_) => {}
                    None => return false,
                }
            }
        }

        true
    }

    fn is_ill_defined(&self) -> bool {
        let mut r = false;
        for row in 0..9 {
            for col in 0..9 {
                let cell = self.grid[row][col];
                match cell.number {
                    Some(_) => {}
                    None => {
                        if cell.candidates_as_vec().len() == 0 {
                            println!(
                                "🚨 Found ill-defined puzzle. No possible candidates for cell ({},{}).",
                                row, col
                            );
                            r = true;
                        }
                    }
                }
            }
        }

        r
    }

    fn solve(&mut self) {
        loop {
            let progress = self.step();

            print!(
                "Step {} progressed by {}. Current internals:\n{}",
                self.iteration,
                progress,
                self.internals()
            );

            if progress == 0 || self.is_solved() || self.is_ill_defined() {
                break;
            }
        }
    }

    fn step(&mut self) -> usize {
        self.iteration += 1;

        println!("Starting step #{}", self.iteration);
        self.assign_candidates();
        self.write_iteration(format!("step-{}-candidates", self.iteration));

        let progress = self.consolidate_candidates();
        self.write_iteration(format!("step-{}-consolidated", self.iteration));

        progress
    }

    fn write_iteration(&self, filename: String) {
        std::fs::create_dir_all("tmp").unwrap();
        let full_filename = format!("tmp/{}", filename);

        let mut ofile = File::create(full_filename).expect("unable to create file");
        ofile
            .write_all(self.display().as_bytes())
            .expect("unable to write");
    }

    /// Review every cell and assign the possible legal candidates based on lines and blocks only.
    fn assign_candidates(&mut self) {
        for cell_index in 0..81 {
            let col = cell_index % 9;
            let row = (cell_index - col) / 9;
            let cell = self.grid[row][col];
            let block = col / 3 + (row / 3) * 3;

            let debug = row == 0 && col == 2 && block == 0;

            if debug {
                println!("@assign_candidates DEBUGGING\n\n\n");
                println!(
                    "   looking at cell #{:02} ({},{}) in block {}: {:?}",
                    cell_index, row, col, block, cell.number
                );
            }

            if let Some(_) = cell.number {
                continue;
            }

            let mut cset: HashSet<u8> = HashSet::new();
            for p in 1..10 {
                cset.insert(p);
            }

            // Narrow candidates by block
            let mut forbidden = self.numbers_in_block(block);
            if debug {
                println!("Numbers in block #{}: {:?}", block, forbidden);
            }
            for f in forbidden.iter() {
                cset.remove(f);
            }

            // Narrow candidates by row
            forbidden = self.numbers_in_row(row);
            if debug {
                println!("Numbers in row #{}: {:?}", row, forbidden);
            }
            for f in forbidden.iter() {
                cset.remove(f);
            }

            // Narrow candidates by column
            forbidden = self.numbers_in_column(col);
            if debug {
                println!("Numbers in column #{}: {:?}", col, forbidden);
            }
            for f in forbidden.iter() {
                cset.remove(f);
            }

            let mut candidates: [u8; 9] = [0; 9];
            let mut sorted: Vec<u8> = cset.drain().collect();
            sorted.sort();
            for (c, canidate) in sorted.iter().enumerate() {
                candidates[c] = *canidate;
            }
            self.grid[row][col].candidates = candidates;
        }
    }

    /// Review the candidates for each cell and infer ways to reduce them or assign a number to the cell. Returns the number of consolidation steps performed.
    fn consolidate_candidates(&mut self) -> usize {
        let mut progress = 0;

        // Start with the trivial: resolve any cell with only one candidate
        for i in 0..9 {
            for j in 0..9 {
                let cell = self.grid[i][j];
                let candidates = cell.candidates_as_vec();

                if candidates.len() == 1 {
                    self.grid[i][j].number = Some(candidates[0]);
                    self.grid[i][j].candidates = [0; 9];
                    progress += 1;
                }
            }
        }

        // Save higher order logic for when we need it
        if progress > 0 {
            return progress;
        }

        // Review all candidates within a _block_ and infer reductions based on uniqueness. For example, a block with only candidates [3, 5], [1, 3], and [2, 3, 5] remaining would require that the last cell be 2 since it's the only valid place for it.

        for b in 0..9 {
            let block = self.block(b);
            println!("⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️⭐️");
            for row in 0..3 {
                for col in 0..3 {
                    let candidates = block[row][col].candidates;

                    println!("   looking @ candidates {:?}", candidates);

                    for candidate in candidates {
                        if candidate > 0 {
                            let count = self.count_candidates_in_block_for(b, candidate);
                            if count == 1 {
                                println!(
                                    "➡️➡️➡️➡️ Inferred that block {}'s row {} @ column {} must be {}",
                                    b, row, col, candidate
                                );
                                self.update_block(b, row, col, candidate);
                                progress += 1;
                                break;
                            }
                        }
                    }
                }
            }
        }

        progress
    }

    /// The corresponding block in our grid. 0 thru 8, starting in top left.
    fn block(&self, b: usize) -> [[Cell; 3]; 3] {
        assert!(b < 9, "Invalid block number: {}", b);

        let origin_x = b % 3;
        let origin_y = (b - origin_x) / 3;

        let mut result: [[Cell; 3]; 3] = [[Cell {
            number: None,
            given: false,
            candidates: [0; 9],
        }; 3]; 3];

        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.grid[origin_y * 3 + i][origin_x * 3 + j];
            }
        }

        result
    }

    fn update_block(&mut self, block_num: usize, row: usize, column: usize, number: u8) {
        let origin_row = block_num / 3;
        let origin_col = block_num % 3;

        self.grid[origin_row * 3 + row][origin_col * 3 + column].number = Some(number);
        self.grid[origin_row * 3 + row][origin_col * 3 + column].candidates = [0; 9];
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

    fn count_candidates_in_block_for(&self, block_num: usize, needle: u8) -> usize {
        let block = self.block(block_num);
        let mut count = 0;

        for i in 0..3 {
            for j in 0..3 {
                match block[i][j].number {
                    Some(_) => {}
                    None => {
                        let candidates = block[i][j].candidates;

                        for candidate in candidates {
                            if candidate == needle {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        count
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
                            let mut candidates: Vec<u8> = Vec::new();
                            for c in 0..9 {
                                if cell.candidates[c] > 0 {
                                    candidates.push(cell.candidates[c]);
                                }
                            }
                            r.push_str(format!("{:?}", candidates).as_str())
                        }
                    }
                    r.push_str("\n");
                }
            }
            r.push_str("\n");
        }

        r
    }

    fn display(&self) -> String {
        let mut r = String::new();

        r.push_str(
            "\n-------------------------------------------------------------------------------------------------------------------------\n"
        );
        for row in 0..9 {
            r.push_str("|");

            for col in 0..9 {
                let cell = self.grid[row][col];

                if let Some(n) = cell.number {
                    let string = n.to_string();
                    let display = string.as_str();
                    r.push_str(format!("{: <13}", display.to_string()).as_str());
                } else {
                    let mut candidates: Vec<u8> = Vec::new();
                    for c in 0..9 {
                        if cell.candidates[c] > 0 {
                            candidates.push(cell.candidates[c]);
                        }
                    }
                    let mut display = String::new();
                    let mut iter = candidates.iter().peekable();
                    display.push_str("[");
                    loop {
                        let c = iter.next();

                        match c {
                            Some(c) => {
                                display.push_str(c.to_string().as_str());
                            }
                            None => {}
                        }

                        match &iter.peek() {
                            Some(_) => display.push_str(","),
                            None => {
                                display.push_str("]");
                                break;
                            }
                        }
                    }
                    r.push_str(format!("{: <13}", display).as_str());
                }

                if (col + 1) % 3 == 0 {
                    r.push_str("|");
                }
            }

            if (row + 1) % 3 == 0 {
                r.push_str(
                    "\n-------------------------------------------------------------------------------------------------------------------------\n"
                );
            } else {
                r.push_str("\n");
            }
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
    puzzle.solve();

    println!(
        "🎁 🎁 🎁 🎁 🎁    FINAL     🎁 🎁 🎁 🎁 🎁\n{}",
        puzzle.internals()
    );

    // TODO: if not solved, we need to pick one of the opposing candidate pairs (e.g. a block with candidates [2,3] and [2, 3]) and work out if a solution can be found. Clone the puzzle, make a guess, and try solving again. If a contradiction is found, throw it away.

    println!("{}", puzzle.display());

    if puzzle.is_solved() {
        println!("Solved! 🙌");
    } else if puzzle.is_ill_defined() {
        println!("💥 Ill-defined puzzle. You probably took a bad guess while solving; try a different candidate for that cell.");
    } else {
        println!("⁉️  Couldn't reduce any further. Need more smarts.");
    }

    Ok(())
}

mod test {
    #[allow(unused_imports)] // wtf?
    use super::*;
    use std::collections::HashSet;

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

        puzzle.assign_candidates();

        // Block 0
        assert!(eq_slice(&puzzle.grid[0][0].candidates, &[1, 3, 8]));
        assert!(eq_slice(&puzzle.grid[0][2].candidates, &[1, 3, 8]));
        assert!(eq_slice(&puzzle.grid[1][2].candidates, &[3, 5, 8]));
        assert!(eq_slice(&puzzle.grid[2][1].candidates, &[2, 3, 5]));
        assert!(eq_slice(&puzzle.grid[2][2].candidates, &[2, 3, 5]));

        println!("Internals:\n{}", puzzle.internals());
    }
}
