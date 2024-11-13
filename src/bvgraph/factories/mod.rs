pub mod bvgraph_decoder_factory;
pub mod bvgraphseq_decoder_factory;

/// The default version of EliasFano used by the [`crate::bvgraph::factories::bvgraph_decoder_factory::ANSBVGraphDecoderFactory`] to retrieve
/// the needed data to initialize a decoder on a specific node.
pub type EF = sux::dict::EliasFano<
    sux::rank_sel::SelectAdaptConst<sux::bits::BitVec<Box<[usize]>>, Box<[usize]>, 12, 4>,
    sux::bits::BitFieldVec<usize, Box<[usize]>>,
>;