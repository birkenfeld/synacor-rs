#![feature(box_syntax)]

use std::thread;
use std::sync::mpsc;

type Tab<T> = Box<[T; M*N]>;

const M: usize = 6;
const N: usize = 32768;
const THREADS: u16 = 4;

fn ack(m: u16, n: u16, r: u16, done: &mut Tab<bool>, tab: &mut Tab<u16>) -> u16 {
    if done[N*m as usize + n as usize] {
        tab[N*m as usize + n as usize]
    } else if m == 0 {
        (n + 1) % 32768
    } else if n == 0 {
        ack(m - 1, r, r, done, tab)
    } else {
        let tmp = ack(m, n - 1, r, done, tab);
        let res = ack(m - 1, tmp, r, done, tab);
        tab[N*m as usize + n as usize] = res;
        done[N*m as usize + n as usize] = true;
        res
    }
}

fn main() {
    let (sender, receiver) = mpsc::channel();
    for j in 0..THREADS {
        let sender = sender.clone();
        thread::Builder::new().stack_size(1 << 24).spawn(move || {
            let mut done = box [false; M*N];
            let mut tab = box [0; M*N];
            for r in j*(32768 / THREADS)..(j+1)*(32768 / THREADS) {
                for loc in done.iter_mut() {
                    *loc = false;
                }
                let result = ack(4, 1, r, &mut done, &mut tab);
                if result == 6 {
                    sender.send(r).unwrap();
                }
                if r % 100 == 0 {
                    println!("{:5} -> {:5}", r, result);
                }
            }
        }).unwrap();
    }
    drop(sender);
    while let Ok(n) = receiver.recv() {
        println!("Found magic number: {}", n);
    }
}
