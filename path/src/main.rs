#[derive(Debug, Clone, Copy)]
enum Dir {
    N, S, E, W
}

#[derive(Debug, Clone, Copy)]
enum Instr {
    Add, Mul, Sub, Num(isize)
}

use Dir::*;
use Instr::*;

fn dir_full(dir: Dir) -> &'static str {
    match dir {
        N => "north",
        S => "south",
        E => "east",
        W => "west",
    }
}

const MAZE: [[Instr; 4]; 4] =
    [[Mul, Num(8), Sub, Num(1)],
     [Num(4), Mul, Num(11), Mul],
     [Add, Num(4), Sub, Num(18)],
     [Num(0), Sub, Num(9), Mul]];

const DIRS: [[&'static [Dir]; 4]; 4] =
    [[&[S, E],    &[S, E, W],    &[S, E, W],    &[]],
     [&[N, S, E], &[N, S, E, W], &[N, S, E, W], &[N, S, W]],
     [&[N, E],    &[N, S, E, W], &[N, S, E, W], &[N, S, W]],
     [&[N, E],    &[N, E],       &[N, E, W],    &[N, W]]];

fn main() {
    let mut stack = Vec::new();
    stack.push((3, 0, N, Num(0), 22, vec![]));
    stack.push((3, 0, E, Num(0), 22, vec![]));

    let mut victory = Vec::new();
    while let Some((mut py, mut px, dir, lastinstr, mut w, mut steps)) = stack.pop() {
        match dir {
            N => py -= 1,
            S => py += 1,
            E => px += 1,
            W => px -= 1,
        }
        steps.push(dir);
        let instr = MAZE[py][px];
        if let Num(operand) = instr {
            match lastinstr {
                Add => w += operand,
                Mul => w *= operand,
                Sub => w -= operand,
                _   => unreachable!(),
            }
            if py == 0 && px == 3 && w == 30 {
                victory = steps;
                break;
            }
            if w <= 0 || steps.len() >= 12 {
                continue;
            }
        }
        for newdir in DIRS[py][px] {
            stack.push((py, px, *newdir, instr, w, steps.clone()));
        }
    }
    if !victory.is_empty() {
        println!("Found the path:\n");
        for dir in &victory {
            println!("go {}", dir_full(*dir));
        }
    } else {
        println!("Found no path :(")
    }
}
