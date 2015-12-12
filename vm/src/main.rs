extern crate byteorder;
extern crate rand;

use std::char;
use std::collections::VecDeque;
use std::fmt;
use std::fs::File;
use std::io::{Cursor, Read, Write, stdin};
use byteorder::{LittleEndian, ReadBytesExt};
use rand::random;

type Addr = usize;
type Val = u16;
type Arg = u16;

#[derive(Debug)]
enum Op {
    Halt,
    Set(Arg, Arg),
    Push(Arg),
    Pop(Arg),
    Eq(Arg, Arg, Arg),
    Gt(Arg, Arg, Arg),
    Jmp(Arg),
    Jt(Arg, Arg),
    Jf(Arg, Arg),
    Add(Arg, Arg, Arg),
    Mul(Arg, Arg, Arg),
    Mod(Arg, Arg, Arg),
    And(Arg, Arg, Arg),
    Or(Arg, Arg, Arg),
    Not(Arg, Arg),
    Rmem(Arg, Arg),
    Wmem(Arg, Arg),
    Call(Arg),
    Ret,
    Out(Arg),
    In(Arg),
    Noop
}

fn fmt_arg(arg: Arg) -> String {
    if arg < 32768 {
        format!("{:x}", arg)
    } else {
        format!("r{}", arg - 32767)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Op::Halt => write!(fmt, "hlt"),
            Op::Set(t, v) => write!(fmt, "{} <- {}", fmt_arg(t), fmt_arg(v)),
            Op::Push(v) => write!(fmt, "push {}", fmt_arg(v)),
            Op::Pop(r) => write!(fmt, "{} <- pop", fmt_arg(r)),
            Op::Eq(r, v1, v2) => write!(fmt, "{} <- {} == {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::Gt(r, v1, v2) => write!(fmt, "{} <- {} > {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::Jmp(t) => write!(fmt, "jmp {}", fmt_arg(t)),
            Op::Jt(v, t) => write!(fmt, "jmp {} if {}", fmt_arg(t), fmt_arg(v)),
            Op::Jf(v, t) => write!(fmt, "jmp {} if not {}", fmt_arg(t), fmt_arg(v)),
            Op::Add(r, v1, v2) => write!(fmt, "{} <- {} + {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::Mul(r, v1, v2) => write!(fmt, "{} <- {} * {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::Mod(r, v1, v2) => write!(fmt, "{} <- {} % {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::And(r, v1, v2) => write!(fmt, "{} <- {} & {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::Or(r, v1, v2) => write!(fmt, "{} <- {} | {}", fmt_arg(r), fmt_arg(v1), fmt_arg(v2)),
            Op::Not(t, v) => write!(fmt, "{} <- ! {}", fmt_arg(t), fmt_arg(v)),
            Op::Rmem(t, v) => write!(fmt, "{} <- read mem at {}", fmt_arg(t), fmt_arg(v)),
            Op::Wmem(t, v) => write!(fmt, "write mem at {} <- {}", fmt_arg(t), fmt_arg(v)),
            Op::Call(a) => write!(fmt, "call {}", fmt_arg(a)),
            Op::Ret => write!(fmt, "ret"),
            Op::Out(v) => write!(fmt, "print {}", fmt_arg(v)),
            Op::In(t) => write!(fmt, "{} <- input", fmt_arg(t)),
            Op::Noop => write!(fmt, "---"),
        }
    }
}

struct VM {
    regs:   [Val; 8],
    stack:  Vec<Val>,
    mem:    [Val; 32768],
    lastip: Addr,
    inbuf:  VecDeque<u8>,
    outbuf: Vec<u8>,
    readin: bool,
    disasm: bool,
}

impl VM {
    fn new(readin: bool) -> VM {
        VM {
            regs:   [0, 0, 0, 0, 0, 0, 0, 0],
            stack:  Vec::with_capacity(1024),
            mem:    [0; 32768],
            lastip: 0,
            inbuf:  VecDeque::new(),
            outbuf: Vec::new(),
            readin: readin,
            disasm: false,
        }
    }

    fn load(&mut self, program: Vec<u8>) {
        let mut cursor = Cursor::new(program);
        let mut i = 0;
        while let Ok(v) = cursor.read_u16::<LittleEndian>() {
            self.mem[i] = v;
            i += 1;
        }
    }

    fn run(&mut self) {
        let mut ip = self.lastip;
        loop {
            self.lastip = ip;
            let op = self.decode(&mut ip);
            match self.exec(op, ip) {
                Some(newip) => ip = newip,
                None => break,
            }
        }
    }

    fn next(&self, ip: &mut Addr) -> Val {
        let res = self.mem[*ip];
        *ip += 1;
        res
    }

    fn decode(&self, ip: &mut Addr) -> Op {
        match self.next(ip) {
            0  => Op::Halt,
            1  => Op::Set(self.next(ip), self.next(ip)),
            2  => Op::Push(self.next(ip)),
            3  => Op::Pop(self.next(ip)),
            4  => Op::Eq(self.next(ip), self.next(ip), self.next(ip)),
            5  => Op::Gt(self.next(ip), self.next(ip), self.next(ip)),
            6  => Op::Jmp(self.next(ip)),
            7  => Op::Jt(self.next(ip), self.next(ip)),
            8  => Op::Jf(self.next(ip), self.next(ip)),
            9  => Op::Add(self.next(ip), self.next(ip), self.next(ip)),
            10 => Op::Mul(self.next(ip), self.next(ip), self.next(ip)),
            11 => Op::Mod(self.next(ip), self.next(ip), self.next(ip)),
            12 => Op::And(self.next(ip), self.next(ip), self.next(ip)),
            13 => Op::Or(self.next(ip), self.next(ip), self.next(ip)),
            14 => Op::Not(self.next(ip), self.next(ip)),
            15 => Op::Rmem(self.next(ip), self.next(ip)),
            16 => Op::Wmem(self.next(ip), self.next(ip)),
            17 => Op::Call(self.next(ip)),
            18 => Op::Ret,
            19 => Op::Out(self.next(ip)),
            20 => Op::In(self.next(ip)),
            21 => Op::Noop,
            v  => panic!("unknown opcode {}", v),
        }
    }

    fn reg(&mut self, arg: Arg) -> &mut Val {
        if 32768 <= arg && arg < 32776 {
            &mut self.regs[(arg - 32768) as usize]
        } else {
            panic!("invalid register argument {}", arg)
        }
    }

    fn val(&self, arg: Arg) -> Val {
        if arg < 32768 {
            arg
        } else if arg < 32776 {
            self.regs[(arg - 32768) as usize]
        } else {
            panic!("invalid register or immediate argument {}", arg)
        }
    }

    fn exec(&mut self, op: Op, mut nextip: Addr) -> Option<Addr> {
        if self.disasm {
            println!("\t[{:06x}] {}", self.lastip, op);
        }
        match op {
            Op::Halt => return None,
            Op::Set(r, v) => *self.reg(r) = self.val(v),
            Op::Push(v) => { let v = self.val(v); self.stack.push(v) },
            Op::Pop(r) => *self.reg(r) = self.stack.pop().unwrap(),
            Op::Eq(r, v1, v2) => *self.reg(r) = if self.val(v1) == self.val(v2) { 1 } else { 0 },
            Op::Gt(r, v1, v2) => *self.reg(r) = if self.val(v1) > self.val(v2) { 1 } else { 0 },
            Op::Jmp(t) => nextip = self.val(t) as Addr,
            Op::Jt(v, t) => if self.val(v) != 0 { nextip = self.val(t) as Addr; },
            Op::Jf(v, t) => if self.val(v) == 0 { nextip = self.val(t) as Addr; },
            Op::Add(r, v1, v2) => *self.reg(r) = (self.val(v1) + self.val(v2)) % 32768,
            Op::Mul(r, v1, v2) => *self.reg(r) = (self.val(v1).wrapping_mul(self.val(v2))) % 32768,
            Op::Mod(r, v1, v2) => *self.reg(r) = self.val(v1) % self.val(v2),
            Op::And(r, v1, v2) => *self.reg(r) = self.val(v1) & self.val(v2),
            Op::Or(r, v1, v2) => *self.reg(r) = self.val(v1) | self.val(v2),
            Op::Not(r, v) => *self.reg(r) = (!self.val(v)) & 0x7FFF,
            Op::Rmem(r, a) => *self.reg(r) = self.mem[self.val(a) as Addr],
            Op::Wmem(a, v) => self.mem[self.val(a) as Addr] = self.val(v),
            Op::Call(t) => { self.stack.push(nextip as Val); nextip = self.val(t) as Addr; },
            Op::Ret => { let a = self.stack.pop(); if a.is_none() { return None }
                         nextip = a.unwrap() as Addr; },
            Op::Out(v) => { let c = self.val(v); self.putc(c) },
            Op::In(r) => { if self.inbuf.is_empty() && !self.readin { return None; }
                           *self.reg(r) = self.getc(); },
            Op::Noop => (),
        }
        Some(nextip)
    }

    fn putc(&mut self, c: Val) {
        let c = char::from_u32(c as u32).unwrap();
        print!("{}", c);
        write!(self.outbuf, "{}", c).unwrap();
    }

    fn getc(&mut self) -> Val {
        match self.inbuf.pop_front() {
            None => {
                let mut line = String::new();
                stdin().read_line(&mut line).unwrap();
                for c in line.chars() {
                    self.inbuf.push_back(c as u8);
                }
                self.getc()
            }
            Some(b'#') => {
                self.mem[0x1571] = 21;  // bypass confirmation
                self.mem[0x1572] = 21;
                self.mem[0x1573] = 21;
                self.mem[0x1574] = 21;
                self.mem[0x1575] = 21;
                self.mem[0x1576] = 21;
                self.regs[7] = 25734;   // set eigth register correctly
                self.getc()
            }
            Some(byte) => byte as Val
        }
    }
}

#[allow(unused)]
fn maze_step(vm: &mut VM, steps: &mut Vec<&'static str>) {
    let direction = ["north", "west", "south", "east"][random::<usize>() % 4];
    let command = format!("go {}\n", direction);
    println!("{}", command);
    vm.inbuf.extend(command.as_bytes());
    vm.outbuf.clear();
    vm.run();
    let outstr = String::from_utf8_lossy(&vm.outbuf).into_owned();
    if outstr.find("I don't understand").is_some() {
        return;
    }
    steps.push(direction);
    if outstr.find("== Twisty passages ==").is_none() {
        panic!("FOUND A WAY OUT?! {:?}", steps);
    }
    if outstr.find("- ladder").is_some() {
        steps.clear();
    }
}

#[allow(unused)]
fn disassemble(vm: &VM, start: Addr, count: usize) {
    let mut ip = start;
    for _ in 0..count {
        let addr = ip;
        let op = vm.decode(&mut ip);
        println!("[{:06x}] {}", addr, op);
    }
}


fn main() {
    let mut vm = VM::new(false);
    let mut prog = Vec::new();
    File::open("../challenge.bin").unwrap().read_to_end(&mut prog).unwrap();
    vm.inbuf.extend(b"take tablet
use tablet
go doorway
go north
go north
go bridge
go continue
go down
go east
take empty lantern
go west
go west
go passage
go ladder
go west
go north
go south
go north
take can
use can
use lantern
go west
go west
go south
go north
go west
go ladder
go darkness
go continue
go west
go west
go west
go west
go north
take red coin
go north
go west
take blue coin
go up
take shiny coin
go down
go east
go east
take concave coin
go down
take corroded coin
go up
go west
use blue coin
use red coin
use shiny coin
use concave coin
use corroded coin
go north
take teleporter
use teleporter
take strange book
look strange book
#use teleporter
go north
go north
go north
go north
go north
go north
go north
go east
take journal
look journal
go west
go north
go north
take orb
go north
go east
go east
go north
go west
go south
go east
go east
go west
go north
go north
go east
go vault
take mirror
use mirror
".iter());

    // vm.disasm = true;
    vm.load(prog);
    vm.run();
}
