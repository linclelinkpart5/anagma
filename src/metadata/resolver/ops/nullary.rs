use crate::metadata::resolver::Error;
use crate::metadata::resolver::ops::Op;
use crate::metadata::resolver::ops::Operand;
use crate::metadata::resolver::ops::OperandStack;
use crate::metadata::resolver::context::ResolverContext;
use crate::metadata::resolver::streams::Stream;
use crate::metadata::stream::block::FileMetaBlockStream;
use crate::metadata::stream::value::MetaValueStream;
use crate::util::file_walkers::ParentFileWalker;
use crate::util::file_walkers::ChildFileWalker;

#[derive(Clone, Copy, Debug)]
pub enum NullaryOp {
    // () -> Stream<V>
    Parents,
    // () -> Stream<V>
    Children,
}

impl Op for NullaryOp {
    fn process<'no>(&self, rc: &ResolverContext<'no>, stack: &mut OperandStack<'no>) -> Result<(), Error> {
        let mb_stream = match self {
            &Self::Parents => FileMetaBlockStream::new(ParentFileWalker::new(rc.current_item_file_path), rc.meta_format, rc.selection, rc.sort_order),
            &Self::Children => FileMetaBlockStream::new(ChildFileWalker::new(rc.current_item_file_path), rc.meta_format, rc.selection, rc.sort_order),
        };

        let stream = Stream::Raw(MetaValueStream::new(rc.current_key_path.clone(), mb_stream));

        stack.push(Operand::Stream(stream));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::NullaryOp;

    use crate::metadata::resolver::ops::Op;
    use crate::metadata::resolver::ops::Operand;
    use crate::metadata::resolver::ops::OperandStack;
    use crate::metadata::resolver::context::ResolverContext;

    use crate::metadata::types::MetaKeyPath;

    use crate::config::selection::Selection;
    use crate::config::sort_order::SortOrder;
    use crate::config::meta_format::MetaFormat;

    use crate::test_util::TestUtil;

    #[test]
    fn test_process() {
        let temp_dir = TestUtil::create_meta_fanout_test_dir("test_process", 3, 3, TestUtil::flag_set_by_default);
        let root_dir = temp_dir.path();

        let current_key_path = MetaKeyPath::new();

        let current_item_file_path = root_dir.join("0").join("0_1").join("0_1_2");
        let selection = Selection::default();

        let rc = ResolverContext {
            current_key_path,
            current_item_file_path: &current_item_file_path,
            meta_format: MetaFormat::Json,
            selection: &selection,
            sort_order: SortOrder::Name,
        };

        let op = NullaryOp::Parents;
        let mut stack: OperandStack = OperandStack::new();

        op.process(&rc, &mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Stream(_) => {},
            _ => { panic!("unexpected operand found on stack"); }
        }

        let op = NullaryOp::Children;
        let mut stack: OperandStack = OperandStack::new();

        op.process(&rc, &mut stack).expect("process failed");

        assert_eq!(1, stack.len());
        match stack.pop().expect("stack is empty") {
            Operand::Stream(_) => {},
            _ => { panic!("unexpected operand found on stack"); }
        }
    }
}
