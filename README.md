# Sudoku Solver

A [Sudoku](https://en.wikipedia.org/wiki/Sudoku) puzzle solver written in [Rust](https://www.rust-lang.org). 

## Why?

During our 3rd family dinner over the Christmas holidays my dad and I got on the topic of how to programmatically solve a Sudoku puzzle. He had mentioned he had started an implementation years ago so we dug in. 

I woke up the next morning dreaming of a possible solution...

## Concepts

The puzzle is a grid of cells. 

Each puzzle has 9 rows, 9 columns, and 9 logical blocks.

All counting is zero-indexed, proceeds left-to-right, and "wraps" to the next item starting on the left, then repeats. Like a dotmatrix printer. 

Thus in the [sample puzzle](./sample.jpeg), block 8 is in the center on the bottom, abutting the "EASY" label. The 6 is at coordinates (0,0); 8 is at (0, 1); 4 is at (0, 2), and 9 is at (2, 2).

## Usage

`cargo run < sample.txt`

