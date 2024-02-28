use dsi_bitstream::{
    impls::{BufBitReader, BufBitWriter, WordAdapter},
    traits::{BitRead, BitWrite, Endianness, WordRead, BE, LE},
};
use mmap_rs::*;

use std::{fs::File, io::BufWriter, path::Path};
use webgraph::utils::MmapBackend;

/// Trait for writing reverse γ codes.
pub trait GammaRevWrite<E: Endianness>: BitWrite<E> {
    fn write_rev_gamma(&mut self, n: u64) -> Result<usize, Self::Error>;
}

impl<B: BitWrite<BE>> GammaRevWrite<BE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_rev_gamma(&mut self, n: u64) -> Result<usize, Self::Error> {
        default_rev_write_gamma(self, n)
    }
}

impl<B: BitWrite<LE>> GammaRevWrite<LE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_rev_gamma(&mut self, n: u64) -> Result<usize, Self::Error> {
        default_rev_write_gamma(self, n)
    }
}

#[inline(always)]
fn default_rev_write_gamma<E: Endianness, B: BitWrite<E>>(
    backend: &mut B,
    mut n: u64,
) -> Result<usize, B::Error> {
    n += 1;
    let number_of_bits_to_write = n.ilog2();

    Ok(backend.write_bits(n, number_of_bits_to_write as _)?
        + backend.write_bits(1, 1)?
        + backend.write_bits(0, number_of_bits_to_write as _)?)
}

pub struct RevBitWriter<E: Endianness, P: AsRef<Path>> {
    path: P,
    bit_writer: BufBitWriter<E, WordAdapter<u64, BufWriter<File>>>,
}

impl<E: Endianness, P: AsRef<Path>> RevBitWriter<E, P> {
    pub fn new(path: P) -> anyhow::Result<Self>
    where
        BufBitWriter<E, WordAdapter<u64, BufWriter<std::fs::File>>>: BitWrite<E>,
    {
        let bit_writer = BufBitWriter::new(WordAdapter::new(BufWriter::new(File::create(
            path.as_ref(),
        )?)));

        Ok(Self { path, bit_writer })
    }

    pub fn push(&mut self, x: u64) -> anyhow::Result<usize>
    where
        BufBitWriter<E, WordAdapter<u64, BufWriter<std::fs::File>>>: BitWrite<E> + GammaRevWrite<E>,
    {
        Ok(self.bit_writer.write_rev_gamma(x)?)
    }

    pub fn flush(mut self) -> anyhow::Result<BufBitReader<LE, RevReader>>
    where
        BufBitReader<LE, RevReader>: BitRead<LE>,
        BufBitWriter<E, WordAdapter<u64, BufWriter<std::fs::File>>>: BitWrite<E>,
    {
        let padding = u64::BITS as usize - self.bit_writer.flush()?;
        let mut rev_reader = BufBitReader::<LE, _, _>::new(RevReader::new(self.path)?);
        rev_reader.skip_bits(padding as usize)?;
        Ok(rev_reader)
    }
}

pub struct RevReader {
    mmap: MmapBackend<u32>,
    position: usize,
}

impl RevReader {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mmap = MmapBackend::<u32>::load(path, MmapFlags::empty())?;
        let position = mmap.as_ref().len();
        Ok(Self { mmap, position })
    }
}

impl WordRead for RevReader {
    type Word = u32;
    type Error = std::io::Error;
    fn read_word(&mut self) -> std::io::Result<u32> {
        if self.position == 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "No more data to read",
            ))
        } else {
            self.position -= 1;
            let w = self.mmap.as_ref()[self.position].to_be();
            Ok(w)
        }
    }
}

#[test]
fn test_rev() -> anyhow::Result<()> {
    use dsi_bitstream::codes::GammaRead;
    use rand::rngs::SmallRng;
    use rand::RngCore;
    use rand::SeedableRng;

    let mut rev_writer = RevBitWriter::<BE, _>::new("test.rev")?;

    let mut v = vec![];

    let mut r = SmallRng::seed_from_u64(42);

    for _ in 0..100 {
        let x = r.next_u64();
        v.push(x);
        rev_writer.push(x);
    }

    let mut rev_reader = rev_writer.flush()?;

    for &x in v.iter().rev() {
        let y = rev_reader.read_gamma()?;
        assert_eq!(y, x);
    }

    Ok(())
}
