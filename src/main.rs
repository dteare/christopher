use std::{collections::HashSet, fmt, fs::File, io::Write};

#[derive(Clone, Copy, Debug)]
struct Cell {
    number: Option<u8>,
    given: bool,
    candidates: [u8; 9],
}
struct Puzzle {
    iteration: usize,
    grid: [[Cell; 9]; 9],
    last_consolidation: Vec<Consolidation>,
}

// The type of consolidation performed during a step towards the solution
#[derive(Clone, Debug, PartialEq)]
enum Consolidation {
    SingleCandidateForCell(CellAssignment),
    OnlyOnePossibleCandidateForBlock(CellAssignment),
    OnlyOnePossibleCandidateForRow(CellAssignment),
    OnlyOnePossibleCandidateForColumn(CellAssignment),
}

#[derive(Clone, Debug, PartialEq)]
struct CellAssignment {
    number: u8,
    block: usize,
    row: usize,
    col: usize,
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

        Puzzle {
            iteration: 0,
            grid,
            last_consolidation: vec![],
        }
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

        // Bad if any row repeats a number

        // Bad if any col repeats a number

        // Bad if any block repeats a number

        true
    }

    fn is_ill_defined(&self) -> bool {
        let mut r = false;

        // Bad if cell has no number assigned and no candidates
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

        // Bad if any row repeats a number

        // Bad if any col repeats a number

        // Bad if any block repeats a number

        r
    }

    fn solve(&mut self) {
        loop {
            let progress = self.step();

            print!(
                "Step {} progressed by {:?}. Current board layout:\n{}",
                self.iteration,
                progress,
                self.display()
            );

            if progress.len() == 0 || self.is_solved() || self.is_ill_defined() {
                break;
            }
        }
    }

    fn step(&mut self) -> Vec<Consolidation> {
        self.iteration += 1;

        println!("Starting step #{}", self.iteration);
        self.assign_candidates();
        self.write_iteration(format!("s{}-candidates", self.iteration));

        self.last_consolidation = self.consolidate_candidates();
        self.write_iteration(format!("s{}-consolidated", self.iteration));

        self.last_consolidation.clone()
    }

    fn write_iteration(&self, filename: String) {
        std::fs::create_dir_all("tmp").unwrap();
        let full_filename = format!("tmp/{}", filename);

        let contents = format!(
            "{}\n\nLast consolidation: {:?}",
            self.display(),
            self.last_consolidation
        );

        let mut ofile = File::create(full_filename).expect("unable to create file");
        ofile
            .write_all(contents.as_bytes())
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

        loop {
            let reductions = self.reduce_candidates_with_sara_flex();

            println!("Sara flex reduced candidates by {}", reductions);

            if reductions == 0 {
                break;
            }
        }
    }

    // Sara flex: combine the following rules to reduce potential candidates:
    //
    //    * Each row and column and block must have 9 unique digits
    //    * Cells "pinned" to certain values force further reductions
    //
    // These two rules yield incredible results, especially as each reduction can trigger further reductions.
    //
    // Returns the number of reductions. Should be called repeatedly until no further simplifcations can be made.
    fn reduce_candidates_with_sara_flex(&mut self) -> usize {
        let mut reductions = 0;

        // Rows
        for i in 0..9 {
            let mut candidates: Vec<Vec<u8>> = Vec::new();
            for c in self.row(i) {
                candidates.push(c.candidates_as_vec());
            }

            let reduced = reduce_candidates_by_uniqueness(candidates);
            for j in 0..9 {
                let mut reduced_candidates: [u8; 9] = [0; 9];

                for (k, c) in reduced[j].iter().enumerate() {
                    reduced_candidates[k] = *c;
                }

                if reduced_candidates != self.grid[i][j].candidates {
                    self.grid[i][j].candidates = reduced_candidates;
                    reductions += 1;
                }
            }
        }

        // Columns
        for i in 0..9 {
            let mut candidates: Vec<Vec<u8>> = Vec::new();
            for c in self.column(i) {
                candidates.push(c.candidates_as_vec());
            }

            let reduced = reduce_candidates_by_uniqueness(candidates);

            for j in 0..9 {
                let mut reduced_candidates: [u8; 9] = [0; 9];

                for (k, c) in reduced[j].iter().enumerate() {
                    reduced_candidates[k] = *c;
                }

                if reduced_candidates != self.grid[j][i].candidates {
                    self.grid[j][i].candidates = reduced_candidates;
                    reductions += 1;
                }
            }
        }

        // Blocks
        for block_num in 0..9 {
            let block_cells = self.block_as_slice(block_num);
            let mut candidates: Vec<Vec<u8>> = Vec::new();
            for c in block_cells {
                candidates.push(c.candidates_as_vec());
            }

            let reduced = reduce_candidates_by_uniqueness(candidates);

            for j in 0..9 {
                let mut reduced_candidates: [u8; 9] = [0; 9];

                for (k, c) in reduced[j].iter().enumerate() {
                    reduced_candidates[k] = *c;
                }

                let modified =
                    self.update_block_candidates(block_num, j / 3, j % 3, reduced_candidates);
                if modified {
                    reductions += 1;
                }
            }
        }

        reductions
    }

    /// Review the candidates for each cell and infer ways to reduce them or assign a number to the cell. Returns the number of consolidation steps performed.
    fn consolidate_candidates(&mut self) -> Vec<Consolidation> {
        let mut progress: Vec<Consolidation> = Vec::new();

        // Start with the trivial: resolve any cell with only one candidate
        for block_num in 0..9 {
            let block = self.block(block_num);
            for row in 0..3 {
                for col in 0..3 {
                    let cell = block[row][col];
                    let candidates = cell.candidates_as_vec();

                    if candidates.len() == 1 {
                        self.update_block(block_num, row, col, candidates[0]);

                        let updated = Consolidation::SingleCandidateForCell(CellAssignment {
                            block: block_num,
                            row,
                            col,
                            number: candidates[0],
                        });
                        progress.push(updated);
                    }
                }
            }
        }

        // Save higher order logic for when we need it
        if progress.len() > 0 {
            return progress;
        }

        // Review all candidates within a _block_ and infer reductions based on uniqueness. For example, a block with only candidates [3, 5], [1, 3], and [2, 3, 5] remaining would require that the last cell be 2 since it's the only valid place for it.
        for b in 0..9 {
            let block = self.block(b);
            for row in 0..3 {
                for col in 0..3 {
                    let candidates = block[row][col].candidates_as_vec();

                    for candidate in candidates {
                        let count = self.count_candidates_in_block_for(b, candidate);
                        if count == 1 {
                            println!(
                                "➡️➡️➡️➡️ Inferred that block {}'s row {} @ column {} must be {}",
                                b, row, col, candidate
                            );
                            self.update_block(b, row, col, candidate);

                            return vec![Consolidation::OnlyOnePossibleCandidateForBlock(
                                CellAssignment {
                                    number: candidate,
                                    row,
                                    col,
                                    block: b,
                                },
                            )];
                        }
                    }
                }
            }
        }

        // Same uniqueness logic as above, but for rows
        for row_num in 0..9 {
            let row = self.row(row_num);
            for (col_num, cell) in row.iter().enumerate() {
                let candidates = cell.candidates_as_vec();

                for candidate in candidates {
                    let count = self.count_candidates_in_row(row_num, candidate);
                    if count == 1 {
                        println!(
                            "➡️➡️➡️➡️ Inferred that row {} @ column {} must be {} because it's the only one available in the ROW",
                            row_num, col_num, candidate
                        );
                        self.set_number(row_num, col_num, candidate);

                        return vec![Consolidation::OnlyOnePossibleCandidateForRow(
                            CellAssignment {
                                number: candidate,
                                row: row_num,
                                col: col_num,
                                block: block_num_for_row_col(row_num, col_num),
                            },
                        )];
                    }
                }
            }
        }

        // Same uniqueness logic as above, but for columns
        for col_num in 0..9 {
            let col = self.column(col_num);

            for (row_num, cell) in col.iter().enumerate() {
                let candidates = cell.candidates_as_vec();

                for candidate in candidates {
                    let count = self.count_candidates_in_col(col_num, candidate);

                    if count == 1 {
                        println!(
                            "➡️➡️➡️➡️ Inferred that row {} @ column {} must be {} because it's the only one in the COLUMN",
                            row_num, col_num, candidate
                        );
                        self.set_number(row_num, col_num, candidate);

                        return vec![Consolidation::OnlyOnePossibleCandidateForColumn(
                            CellAssignment {
                                number: candidate,
                                row: row_num,
                                col: col_num,
                                block: block_num_for_row_col(row_num, col_num),
                            },
                        )];
                    }
                }
            }
        }

        vec![]
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

    /// The corresponding block in our grid as a single slice of cells. Blocks are numbered 0 thru 8, starting in top left, proceeding left-to-right, top-to-bottom.
    fn block_as_slice(&self, b: usize) -> [Cell; 9] {
        assert!(b < 9, "Invalid block number: {}", b);

        let origin_x = b % 3;
        let origin_y = (b - origin_x) / 3;

        let mut result: [Cell; 9] = [Cell {
            number: None,
            given: false,
            candidates: [0; 9],
        }; 9];

        for i in 0..3 {
            for j in 0..3 {
                result[i * 3 + j] = self.grid[origin_y * 3 + i][origin_x * 3 + j];
            }
        }

        result
    }

    /// The corresponding row in our grid.
    fn row(&self, r: usize) -> [Cell; 9] {
        assert!(r < 9, "Invalid row number: {}", r);

        let mut result: [Cell; 9] = [Cell {
            number: None,
            given: false,
            candidates: [0; 9],
        }; 9];

        for i in 0..9 {
            result[i] = self.grid[r][i];
        }

        result
    }

    /// The corresponding column in our grid.
    fn column(&self, c: usize) -> [Cell; 9] {
        assert!(c < 9, "Invalid column number: {}", c);

        let mut result: [Cell; 9] = [Cell {
            number: None,
            given: false,
            candidates: [0; 9],
        }; 9];

        for i in 0..9 {
            result[i] = self.grid[i][c];
        }

        result
    }

    fn update_block(&mut self, block_num: usize, row: usize, col: usize, number: u8) {
        let origin_row = block_num / 3;
        let origin_col = block_num % 3;

        self.grid[origin_row * 3 + row][origin_col * 3 + col].number = Some(number);
        self.grid[origin_row * 3 + row][origin_col * 3 + col].candidates = [0; 9];
    }

    fn set_number(&mut self, row: usize, col: usize, number: u8) {
        self.grid[row][col].number = Some(number);
        self.grid[row][col].candidates = [0; 9];
    }

    // Updated cell candidates in block. Returns true if an update took place
    fn update_block_candidates(
        &mut self,
        block_num: usize,
        row: usize,
        col: usize,
        candidates: [u8; 9],
    ) -> bool {
        let origin_row = block_num / 3;
        let origin_col = block_num % 3;

        if self.grid[origin_row * 3 + row][origin_col * 3 + col].candidates != candidates {
            self.grid[origin_row * 3 + row][origin_col * 3 + col].candidates = candidates;
            return true;
        }

        false
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

    fn count_candidates_in_row(&self, row_num: usize, needle: u8) -> usize {
        let row = self.row(row_num);
        let mut count = 0;

        for i in 0..9 {
            match row[i].number {
                Some(_) => {}
                None => {
                    let candidates = row[i].candidates;

                    for candidate in candidates {
                        if candidate == needle {
                            count += 1;
                        }
                    }
                }
            }
        }

        count
    }

    fn count_candidates_in_col(&self, col_num: usize, needle: u8) -> usize {
        let col = self.column(col_num);
        let mut count = 0;

        for i in 0..9 {
            match col[i].number {
                Some(_) => {}
                None => {
                    let candidates = col[i].candidates;

                    for candidate in candidates {
                        if candidate == needle {
                            count += 1;
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

// Given 9 sets of candidate sets (from either a row, line, or block), look for numbers that are "pinned" to a particular set of sets. Then use this fact to eliminate those numbers from all other sets.
//
// For example, given:
//
//      [2,7], [2,5,7,8], 1, 3, 9, 4, 6, [5,8], [5,8]
//
// We know that 5 and 8 must be in the last two sets, and therefore cannot be anywhere else. This allows us to reduce [2,5,7,8] to [2,7].
//
// Sets need not be exact duplicates for this trick to work. For example, given:
//
//     [2,7], [2,5,7,8], 1, 9, 4, 6, [5,8], [3,8], [5,3]
//
// We can make a super set with the last three sets to form [3,5,8]. Since there are exactly 3 numbers possible for each of these 3 sets, the numbers within this super set are "pinned" and can be excluded from the rest of the line. In this example it would result in the 5 & 8 in the second set should be removed.
//
// Returns the consolidated sets in the same order they were provided.
//pub fn reduce_candidates_by_uniqueness(candidates: [[u8; 9]; 9]) -> [[u8; 9]; 9] {
pub fn reduce_candidates_by_uniqueness(candidates: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    use hashbag::HashBag;

    let mut reduced: Vec<HashSet<u8>> = Vec::new();
    let mut bag: HashBag<Vec<u8>> = HashBag::new();

    for i in 0..9 {
        bag.insert(candidates[i].clone());

        let mut r: HashSet<u8> = HashSet::new();
        for c in candidates[i].iter() {
            r.insert(*c);
        }
        reduced.push(r);
    }

    let mut pinned: Vec<Vec<u8>> = Vec::new();
    for (candidate, count) in bag.set_iter() {
        // println!("{} instance of {:?}", count, candidate);

        if count > 1 && count == candidate.len() {
            // Pinned pair, triplet, quadruplet, etc.
            pinned.push(candidate.clone());
        }
    }

    // TODO: Figure out how to find pinned supersets from N sets that contain N numbers. E.g. [5,8], [3,8], [5,3] => [3,5,8].

    // println!("Pinned pairs/triplets/quadruplets/etc: {:?}", pinned);

    // Remove contents of each pinned set from all _other_ sets.
    for pinned_numbers in pinned.iter() {
        for i in 0..9 {
            if *pinned_numbers == candidates[i] {
                // println!("Pinned set {:?} matched itself; skipping", pinned_numbers);
                continue;
            }

            for pinned_number in pinned_numbers {
                // Changing the
                reduced[i].remove(pinned_number);
            }
        }
    }

    let mut result: Vec<Vec<u8>> = Vec::new();
    for i in 0..9 {
        let mut entries: Vec<u8> = reduced[i].iter().map(|c| *c).collect();
        entries.sort();
        result.push(entries);
    }

    return result;
}

pub fn read_stdin() -> Result<String, std::io::Error> {
    let mut buf = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
    Ok(buf)
}

pub fn block_num_for_row_col(row: usize, col: usize) -> usize {
    (row / 3) * 3 + col / 3
}

fn main() -> Result<(), std::io::Error> {
    let input = &read_stdin()?;
    let mut puzzle = Puzzle::parse(input);
    puzzle.solve();

    println!(
        "🎁 🎁 🎁 🎁 🎁    FINAL     🎁 🎁 🎁 🎁 🎁\n{}",
        puzzle.display()
    );

    // TODO: if not solved, we need to pick one of the opposing candidate pairs (e.g. a block with candidates [2,3] and [2, 3]) and work out if a solution can be found. Clone the puzzle, make a guess, and try solving again. If a contradiction is found, throw it away.

    // Create vector of all possible guesses
    //  guesses = [[Block1, Row0, Col2 = 2], [Block1, Row0, Col2 = 5], ....]

    // solved_puzzle = solve_with_guess(puzzle, guesses)
    // solved_puzzle.display()

    // fn solve_with_guess(puzzle, mut guesses) -> Result<Puzzle, Error> {
    //
    //      let guess = guesses.pop()
    //
    //      let my_puzzle = puzzle.clone()
    //
    //      my_puzzle.grid[][] = guess
    //
    //
    //      my_puzzle.solve()
    //      if my_puzzle.is_solved() {
    //          return Ok(my_puzzle)
    //      }
    //      else if my_puzzle.is_ill_defined() {
    //          return Error(...)
    //      }
    //      else {
    //          return solve_with_guess(my_puzzle, guesses)
    //      }
    //
    // }

    // for guess in v.iter() {
    //      my_guess = puzzle.clone()
    //      my_guess.set_block_num(Block1, Row0, Col2)
    //      my_guess.solve()
    //      look for solved, bad, or more guesses needed
    //      Create NEW list of guesses. v2 = [...]
    // }

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
    fn helpers() {
        assert_eq!(0, block_num_for_row_col(0, 0));
        assert_eq!(0, block_num_for_row_col(2, 2));
        assert_eq!(2, block_num_for_row_col(1, 6));
        assert_eq!(6, block_num_for_row_col(7, 2));
        assert_eq!(8, block_num_for_row_col(8, 8));
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
        assert!(eq_slice(&puzzle.grid[1][2].candidates, &[5, 8]));
        assert!(eq_slice(&puzzle.grid[2][1].candidates, &[2, 3, 5]));
        assert!(eq_slice(&puzzle.grid[2][2].candidates, &[2, 3, 5]));

        println!("Internals:\n{}", puzzle.internals());
    }

    #[test]
    fn reduce_candidates_by_uniqueness() {
        let pinned_pair: Vec<Vec<u8>> = vec![
            vec![2, 7],
            vec![2, 5, 7, 8],
            vec![1],
            vec![3],
            vec![9],
            vec![4],
            vec![6],
            vec![5, 8],
            vec![5, 8],
        ];

        let mut reduced = super::reduce_candidates_by_uniqueness(pinned_pair);

        assert_eq!(reduced[0], vec![2, 7]);
        assert_eq!(reduced[1], vec![2, 7]);
        assert_eq!(reduced[2], vec![1]);
        assert_eq!(reduced[3], vec![3]);
        assert_eq!(reduced[4], vec![9]);
        assert_eq!(reduced[5], vec![4]);
        assert_eq!(reduced[6], vec![6]);
        assert_eq!(reduced[7], vec![5, 8]);
        assert_eq!(reduced[8], vec![5, 8]);

        let pinned_triplet: Vec<Vec<u8>> = vec![
            vec![6, 3, 8],
            vec![3, 4, 8],
            vec![3, 4, 8],
            vec![1],
            vec![2, 4],
            vec![5],
            vec![4, 8, 9],
            vec![7],
            vec![3, 4, 8],
        ];

        reduced = super::reduce_candidates_by_uniqueness(pinned_triplet);

        assert_eq!(reduced[0], vec![6]);
        assert_eq!(reduced[1], vec![3, 4, 8]);
        assert_eq!(reduced[2], vec![3, 4, 8]);
        assert_eq!(reduced[3], vec![1]);
        assert_eq!(reduced[4], vec![2]);
        assert_eq!(reduced[5], vec![5]);
        assert_eq!(reduced[6], vec![9]);
        assert_eq!(reduced[7], vec![7]);
        assert_eq!(reduced[8], vec![3, 4, 8]);
    }
}
