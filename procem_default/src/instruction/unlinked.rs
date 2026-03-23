use ars::range::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct UnlinkedInstruction {
    instr_idx: usize,
    label: Range,
}

impl UnlinkedInstruction {
    #[must_use]
    #[inline]
    pub(crate) const fn new(instr_idx: usize, label: Range) -> Self {
        Self { instr_idx, label }
    }
}
