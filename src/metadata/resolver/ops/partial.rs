/// Unary operators as partially-applied binary operators.

use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::binary::BinaryOp;

pub enum PartialBinaryOp<'o> {
    First(BinaryOp, Operand<'o>),
    Second(BinaryOp, Operand<'o>),
}

impl<'o> PartialBinaryOp<'o> {
    pub fn process(&self, operand: Operand<'o>) -> Operand<'o> {
        operand
    }
}
