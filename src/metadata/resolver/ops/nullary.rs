use metadata::resolver::Error;
use metadata::resolver::ops::Op;
use metadata::resolver::ops::Operand;
use metadata::resolver::ops::OperandStack;
use metadata::resolver::context::ResolverContext;
use metadata::resolver::streams::Stream;
use metadata::stream::block::FileMetaBlockStream;
use metadata::stream::value::MetaValueStream;
use util::file_walkers::ParentFileWalker;
use util::file_walkers::ChildFileWalker;

#[derive(Clone, Copy, Debug)]
pub enum NullaryOp {
    // () -> Stream<V>
    Parents,
    // () -> Stream<V>
    Children,
}

impl Op for NullaryOp {
    fn process<'k, 'p, 's>(&self, rc: &ResolverContext<'k, 'p, 's>, stack: &mut OperandStack<'k, 'p, 's>) -> Result<(), Error> {
        let mb_stream = match self {
            &Self::Parents => FileMetaBlockStream::new(ParentFileWalker::new(rc.current_item_file_path), rc.meta_format, rc.selection, rc.sort_order),
            &Self::Children => FileMetaBlockStream::new(ChildFileWalker::new(rc.current_item_file_path), rc.meta_format, rc.selection, rc.sort_order),
        };

        let stream = Stream::Raw(MetaValueStream::new(rc.current_key_path.clone(), mb_stream));

        stack.push(Operand::Stream(stream));

        Ok(())
    }
}
