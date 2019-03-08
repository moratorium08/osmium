#![no_main]
#![no_std]

#[macro_use]
extern crate misc;

use core::str;
use misc::syscall;
use misc::uart;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum State {
    Blank,
    Black,
    White,
}
impl State {
    pub fn print(&self, num: usize) {
        match self {
            State::Black => print!("{}", 'x'),
            State::White => print!("{}", 'o'),
            State::Blank => print!("{}", num),
        }
    }
}

struct Board {
    board: [[State; 3]; 3],
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Status {
    BlackWin,
    WhiteWin,
    Draw,
    Running,
}

#[derive(Copy, Clone)]
pub enum Turn {
    Black,
    White,
}

impl Turn {
    fn next(self) -> Turn {
        match self {
            Turn::Black => Turn::White,
            Turn::White => Turn::Black,
        }
    }
    fn to_state(self) -> State {
        match self {
            Turn::Black => State::Black,
            Turn::White => State::White,
        }
    }
}
impl Board {
    fn new() -> Board {
        Board { board: [[State::Blank; 3]; 3] }
    }
    fn put(&mut self, x: usize, y: usize, turn: Turn) -> bool {
        println!("{} {}", x, y);
        if self.board[x][y] != State::Blank {
            false
        } else {
            self.board[x][y] = turn.to_state();
            true
        }

    }
    fn row(&self, i: usize) -> bool {
        self.board[i][0] == self.board[i][1] && self.board[i][1] == self.board[i][2] &&
            self.board[i][0] != State::Blank
    }
    fn column(&self, i: usize) -> bool {
        self.board[0][i] == self.board[1][i] && self.board[1][i] == self.board[2][i] &&
            self.board[0][i] != State::Blank
    }
    fn naname(&self) -> bool {
        self.board[1][1] != State::Blank &&
            ((self.board[0][0] == self.board[1][1] && self.board[1][1] == self.board[2][2]) ||
                 (self.board[2][0] == self.board[1][1] && self.board[1][1] == self.board[0][2]))
    }
    fn status(&self) -> Status {
        for i in 0..3 {
            if self.row(i) {
                if self.board[i][0] == State::Black {
                    return Status::BlackWin;
                } else {
                    return Status::WhiteWin;
                }
            }
            if self.column(i) {
                if self.board[0][i] == State::Black {
                    return Status::BlackWin;
                } else {
                    return Status::WhiteWin;
                }
            }
            if self.naname() {
                if self.board[1][1] == State::Black {
                    return Status::BlackWin;
                } else {
                    return Status::WhiteWin;
                }
            }
        }
        Status::Running
    }

    fn print(&self) {
        for j in 0..3 {
            for i in 0..3 {
                self.board[i][j].print(j * 3 + i);
            }
            println!();
        }
    }
}

fn get_pos() -> u8 {
    let mut buf = [0u8; 256];
    loop {
        let (i, b) = uart::buffered_readline(&mut buf);
        if !b {
            println!("please try again.");
            continue;
        } else {
            if i != 1 {
                println!("illegal input. try again");
                continue;
            }
            if buf[0] == b'9' {
                println!("illegal input. try again");
                continue;
            }
            return buf[0] - b'0';
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut board = Board::new();
    let mut turn = Turn::Black;
    for i in 0..9 {
        board.print();

        print!("Enter a number which you want to put on> ");
        let p = get_pos();
        let x = p % 3;
        let y = p / 3;
        if !board.put(x as usize, y as usize, turn) {
            println!("You cannot put there.");
            println!("{} wins", 
            match turn {
                Turn::Black => "white",
                Turn::White => "black"
            });
            return syscall::sys_exit(0);
        };
        match board.status() {
            Status::Running => {
            },
            Status::Draw => {
                board.print();
                println!("Draw.");
            },
            Status::WhiteWin => {
                board.print();
                println!("White win.");
                break;
            },
            Status::BlackWin=> {
                board.print();
                println!("Black win.");
                break;
            },
        }
        turn = turn.next();
    }
    if board.status() == Status::Running {
        println!("Draw.")
    }
    syscall::sys_exit(0)
}