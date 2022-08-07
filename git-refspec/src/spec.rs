use crate::types::Push;
use crate::{Fetch, Instruction, Mode, Operation, RefSpec, RefSpecRef};

/// Access
impl RefSpecRef<'_> {
    /// Return the refspec mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Transform the state of the refspec into an instruction making clear what to do with it.
    pub fn instruction(&self) -> Instruction<'_> {
        match self.op {
            Operation::Fetch => match (self.mode, self.src, self.dst) {
                (Mode::Normal | Mode::Force, Some(src), None) => Instruction::Fetch(Fetch::Only { src }),
                (Mode::Normal | Mode::Force, Some(src), Some(dst)) => Instruction::Fetch(Fetch::AndUpdate {
                    src,
                    dst,
                    allow_non_fast_forward: matches!(self.mode, Mode::Force),
                }),
                (Mode::Negative, Some(src), None) => Instruction::Fetch(Fetch::Exclude { src }),
                (mode, src, dest) => {
                    unreachable!(
                        "BUG: fetch instructions with {:?} {:?} {:?} are not possible",
                        mode, src, dest
                    )
                }
            },
            Operation::Push => match (self.mode, self.src, self.dst) {
                (Mode::Normal | Mode::Force, Some(src), None) => Instruction::Push(Push::Matching {
                    src,
                    dst: src,
                    allow_non_fast_forward: matches!(self.mode, Mode::Force),
                }),
                (Mode::Normal | Mode::Force, None, Some(dst)) => {
                    Instruction::Push(Push::Delete { ref_or_pattern: dst })
                }
                (Mode::Normal | Mode::Force, None, None) => Instruction::Push(Push::AllMatchingBranches {
                    allow_non_fast_forward: matches!(self.mode, Mode::Force),
                }),
                (Mode::Normal | Mode::Force, Some(src), Some(dst)) => Instruction::Push(Push::Matching {
                    src,
                    dst,
                    allow_non_fast_forward: matches!(self.mode, Mode::Force),
                }),
                (mode, src, dest) => {
                    unreachable!(
                        "BUG: push instructions with {:?} {:?} {:?} are not possible",
                        mode, src, dest
                    )
                }
            },
        }
    }
}

/// Conversion
impl RefSpecRef<'_> {
    /// Convert this ref into a standalone, owned copy.
    pub fn to_owned(&self) -> RefSpec {
        RefSpec {
            mode: self.mode,
            op: self.op,
            src: self.src.map(ToOwned::to_owned),
            dst: self.dst.map(ToOwned::to_owned),
        }
    }
}
