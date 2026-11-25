//! Pike VM: simulates the NFA over all threads in lock step, tracking
//! captures with leftmost-first (Perl-like) priority and O(n*m) time.

use crate::nfa::{Inst, Program};

pub type Slots = Vec<Option<usize>>;

struct Thread {
    pc: usize,
    slots: Slots,
}

/// Ordered thread list with a sparse "already added" set per step.
struct List {
    threads: Vec<Thread>,
    seen: Vec<bool>,
}

impl List {
    fn new(n: usize) -> Self {
        List { threads: Vec::new(), seen: vec![false; n] }
    }

    fn clear(&mut self) {
        self.threads.clear();
        self.seen.iter_mut().for_each(|s| *s = false);
    }
}

/// Runs `prog` against `chars`, trying start positions left to right.
/// Positions in the returned slots are char indices.
pub fn search(prog: &Program, chars: &[char], start: usize) -> Option<Slots> {
    let n = prog.insts.len();
    let mut clist = List::new(n);
    let mut nlist = List::new(n);
    let mut matched: Option<Slots> = None;

    for pos in start..=chars.len() {
        if matched.is_none() {
            // Lowest priority: begin a new attempt at this position.
            add(prog, &mut clist, 0, pos, chars, vec![None; prog.slot_count()]);
        }
        if clist.threads.is_empty() && matched.is_some() {
            break;
        }
        let ch = chars.get(pos).copied();
        for t in std::mem::take(&mut clist.threads) {
            let advance = match (&prog.insts[t.pc], ch) {
                (Inst::Char(c), Some(x)) => *c == x,
                (Inst::Any, Some(x)) => x != '\n',
                (Inst::Class(cl), Some(x)) => cl.matches(x),
                (Inst::Match, _) => {
                    matched = Some(t.slots);
                    break; // lower-priority threads are cut off
                }
                _ => false,
            };
            if advance {
                add(prog, &mut nlist, t.pc + 1, pos + 1, chars, t.slots);
            }
        }
        clist.clear();
        std::mem::swap(&mut clist, &mut nlist);
    }
    // A thread may have reached Match on the final iteration only via nlist.
    if matched.is_none() {
        for t in clist.threads {
            if prog.insts[t.pc] == Inst::Match {
                return Some(t.slots);
            }
        }
    }
    matched
}

/// Follows epsilon transitions from `pc`, appending runnable threads in
/// priority order.
fn add(prog: &Program, list: &mut List, pc: usize, pos: usize, chars: &[char], mut slots: Slots) {
    if list.seen[pc] {
        return;
    }
    list.seen[pc] = true;
    match &prog.insts[pc] {
        Inst::Jmp(x) => add(prog, list, *x, pos, chars, slots),
        Inst::Split(x, y) => {
            add(prog, list, *x, pos, chars, slots.clone());
            add(prog, list, *y, pos, chars, slots);
        }
        Inst::Save(n) => {
            slots[*n] = Some(pos);
            add(prog, list, pc + 1, pos, chars, slots);
        }
        Inst::AssertStart => {
            if pos == 0 {
                add(prog, list, pc + 1, pos, chars, slots);
            }
        }
        Inst::AssertEnd => {
            if pos == chars.len() {
                add(prog, list, pc + 1, pos, chars, slots);
            }
        }
        _ => list.threads.push(Thread { pc, slots }),
    }
}
