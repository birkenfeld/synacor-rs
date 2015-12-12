// Direction to go between the rooms
#[derive(Debug, Clone, Copy)]
enum Dir {
    N, S, E, W
}

// An instruction from one of the room floors
#[derive(Debug, Clone, Copy)]
enum Instr {
    Add, Mul, Sub, Num(isize)
}

use Dir::*;
use Instr::*;

// For display purposes
fn dir_full(dir: Dir) -> &'static str {
    match dir {
        N => "north",
        S => "south",
        E => "east",
        W => "west",
    }
}

// The room layout: entry is at the bottom left, exit at the top right
const MAZE: [[Instr; 4]; 4] =
    [[Mul, Num(8), Sub, Num(1)],
     [Num(4), Mul, Num(11), Mul],
     [Add, Num(4), Sub, Num(18)],
     [Num(0), Sub, Num(9), Mul]];

// Directions we are allowed to go from each room (note that we are
// not allowed to go back to the starting room)
const DIRS: [[&'static [Dir]; 4]; 4] =
    [[&[S, E],    &[S, E, W],    &[S, E, W],    &[]],
     [&[N, S, E], &[N, S, E, W], &[N, S, E, W], &[N, S, W]],
     [&[N, E],    &[N, S, E, W], &[N, S, E, W], &[N, S, W]],
     [&[N, E],    &[N, E],       &[N, E, W],    &[N, W]]];

fn main() {
    // This uses a simple depth-first search with a stack of work-to-do
    let mut stack = Vec::new();
    // It's easiest to add the first possible steps manually
    stack.push((3, 0, N, Num(0), 22, vec![]));
    stack.push((3, 0, E, Num(0), 22, vec![]));

    let mut victory = Vec::new();
    while let Some((mut py, mut px, dir, lastinstr, mut w, mut steps)) = stack.pop() {
        // Execute the step in the chosen direction
        match dir {
            N => py -= 1,
            S => py += 1,
            E => px += 1,
            W => px -= 1,
        }
        // ... and record the step for retracing successful attempts
        steps.push(dir);
        let instr = MAZE[py][px];
        // We stepped into a number room?
        if let Num(operand) = instr {
            // Execute operation
            match lastinstr {
                Add => w += operand,
                Mul => w *= operand,
                Sub => w -= operand,
                _   => unreachable!(),
            }
            // Victory condition: we are in the top right room with the
            // correct weight
            if py == 0 && px == 3 && w == 30 {
                victory = steps;
                break;
            }
            // Abort conditions: weight zero (orb disappears), too many
            // steps (hourglass runs out)
            if w <= 0 || steps.len() >= 12 {
                continue;
            }
        }
        // Now put all possible next steps onto the stack
        for newdir in DIRS[py][px] {
            stack.push((py, px, *newdir, instr, w, steps.clone()));
        }
    }
    // We should have a solution here...
    if !victory.is_empty() {
        println!("Found the path:\n");
        for dir in &victory {
            println!("go {}", dir_full(*dir));
        }
    } else {
        println!("Found no path :(")
    }
}
