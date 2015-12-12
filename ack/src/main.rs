#![feature(box_syntax)]

use std::thread;
use std::sync::mpsc;

// Type for memoization tables
type Tab<T> = Box<[T; M*N]>;
// Number of entries in the tables (for first/second argument of ack)
// Note, we're using arithmetic modulo 32768, so we're actually memoizing
// for all possible values of the second argument
const M: usize = 6;
const N: usize = 32768;

// Target result (from teleporter code)
const GOAL: u16 = 6;

// Number of threads to search in parallel
const THREADS: u16 = 4;

// The modified Ackermann function used by the teleporter.
fn ack(m: u16, n: u16, r: u16, done: &mut Tab<bool>, tab: &mut Tab<u16>) -> u16 {
    if done[N*m as usize + n as usize] {
        // Table hit
        tab[N*m as usize + n as usize]
    } else if m == 0 {
        // The easy case: don't forget the modulo operation (this is the only
        // location where it has to be applied)
        (n + 1) % 32768
    } else if n == 0 {
        // The (modified) simple recursive case
        ack(m - 1, r, r, done, tab)
    } else {
        // The complex case
        let tmp = ack(m, n - 1, r, done, tab);
        let res = ack(m - 1, tmp, r, done, tab);
        // Store in the table for future iterations
        tab[N*m as usize + n as usize] = res;
        done[N*m as usize + n as usize] = true;
        res
    }
}

fn main() {
    // We're using channels to send the lucky hit back to the main thread
    let (sender, receiver) = mpsc::channel();
    for j in 0..THREADS {
        let sender = sender.clone();
        // For some reason, we need a large stack size for the threads?
        thread::Builder::new().stack_size(1 << 22).spawn(move || {
            // Allocate memoization tables on the heap
            let mut done = box [false; M*N];
            let mut tab = box [0; M*N];
            for r in j*(32768 / THREADS)..(j+1)*(32768 / THREADS) {
                // Clear the boolean table (compiled down to a memset)
                for loc in done.iter_mut() {
                    *loc = false;
                }
                // Calculate result
                let result = ack(4, 1, r, &mut done, &mut tab);
                if result == GOAL {
                    sender.send(r).unwrap();
                }
                // Give some indication of progress
                if r % 100 == 0 {
                    println!("{:5} -> {:5}", r, result);
                }
            }
        }).unwrap();
    }
    // With this sender gone, the receiver's recv() will return an Err
    // as soon as all threads are finished (and their senders dropped)
    drop(sender);
    while let Ok(n) = receiver.recv() {
        // There should be only one, actually
        println!("Found a magic number: {}", n);
    }
}
