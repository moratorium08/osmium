#[derive(Debug)]
pub enum Error {
    Full,
    Empty,
}

// instant implmentation of bounded buffer
// this is not thread safe
const SIZE: usize = 8;
pub struct BoundedBuffer<T: Sized + Copy> {
    buffer: [T; SIZE],
    next_in: usize,
    next_out: usize,
}

fn count_up(v: usize) -> usize {
    let v = v + 1;
    if v == SIZE {
        0
    } else {
        v
    }
}

impl<T: Sized + Copy> BoundedBuffer<T> {
    pub fn new(init: T) -> BoundedBuffer<T> {
        BoundedBuffer {
            buffer: [init; SIZE],
            next_in: 0,
            next_out: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.next_in == self.next_out
    }
    pub fn is_full(&self) -> bool {
        count_up(self.next_in) == self.next_out
    }
    pub fn enqueue(&mut self, val: T) -> Result<(), Error> {
        if self.is_full() {
            return Err(Error::Full);
        }
        self.buffer[self.next_in] = val;
        self.next_in += 1;
        Ok(())
    }
    pub fn dequeue(&mut self) -> Result<T, Error> {
        if self.is_empty() {
            return Err(Error::Empty);
        }
        let idx = self.next_out;
        self.next_out = count_up(idx);
        Ok(self.buffer[idx])
    }
    pub fn len(&self) -> usize {
        if self.next_out <= self.next_in {
            self.next_in - self.next_out
        } else {
            self.next_in + SIZE - self.next_out
        }
    }
}

#[test]
fn test_bb() {
    let bb = &mut BoundedBuffer::new(0);
    assert!(bb.is_empty());
    bb.enqueue(1).unwrap();
    println!("ok");
    for i in 1..SIZE - 1 {
        bb.enqueue(i + 1).unwrap();
    }
    assert!(!bb.is_empty());
    assert!(bb.is_full());
    assert_eq!(bb.dequeue().unwrap(), 1);
    assert_eq!(bb.dequeue().unwrap(), 2);
    assert_eq!(bb.len(), SIZE - 3);
}
