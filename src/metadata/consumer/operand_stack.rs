use metadata::consumer::operand::Operand;

pub struct OperandStack<'k, 'p, 's>(Vec<Operand<'k, 'p, 's>>);
