use dsi_bitstream::{
    codes::GammaRead,
    impls::{BufBitReader, BufBitWriter, WordAdapter},
    traits::{BitRead, BitWrite, Endianness, WordRead, BE, LE},
};
use mmap_rs::*;

use std::{fs::File, io::BufWriter, iter::FusedIterator, path::Path};
use webgraph::utils::MmapHelper;

///Table used to speed up the writing of gamma codes
const WRITE_BE: &[u16] = &[
    1, 2, 6, 4, 12, 20, 28, 8, 24, 40, 56, 72, 88, 104, 120, 16, 48, 80, 112, 144, 176, 208, 240,
    272, 304, 336, 368, 400, 432, 464, 496, 32, 96, 160, 224, 288, 352, 416, 480, 544, 608, 672,
    736, 800, 864, 928, 992, 1056, 1120, 1184, 1248, 1312, 1376, 1440, 1504, 1568, 1632, 1696,
    1760, 1824, 1888, 1952, 2016, 64,
];

///Table used to speed up the writing of gamma codes
const WRITE_LEN_BE: &[u16] = &[
    1, 3, 3, 5, 5, 5, 5, 7, 7, 7, 7, 7, 7, 7, 7, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11,
    11, 11, 11, 11, 11, 11, 11, 11, 13,
];

///Table used to speed up the writing of gamma codes
const WRITE_LE: &[u16] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
];

///Table used to speed up the writing of gamma codes
const WRITE_LEN_LE: &[u16] = &[
    1, 3, 3, 5, 5, 5, 5, 7, 7, 7, 7, 7, 7, 7, 7, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11, 11,
    11, 11, 11, 11, 11, 11, 11, 11, 13,
];

/// Write a value using an encoding table.
///
/// If the result is `Some` the encoding was successful, and
/// length of the code is returned.
#[inline(always)]
pub fn write_table_le<B: BitWrite<LE>>(
    backend: &mut B,
    value: u64,
) -> Result<Option<usize>, B::Error> {
    Ok(if let Some(bits) = WRITE_LE.get(value as usize) {
        let len = WRITE_LEN_LE[value as usize] as usize;
        backend.write_bits(*bits as u64, len)?;
        Some(len)
    } else {
        None
    })
}

/// Write a value using an encoding table.
///
/// If the result is `Some` the encoding was successful, and
/// length of the code is returned.
#[inline(always)]
pub fn write_table_be<B: BitWrite<BE>>(
    backend: &mut B,
    value: u64,
) -> Result<Option<usize>, B::Error> {
    Ok(if let Some(bits) = WRITE_BE.get(value as usize) {
        let len = WRITE_LEN_BE[value as usize] as usize;
        backend.write_bits(*bits as u64, len)?;
        Some(len)
    } else {
        None
    })
}

/// Trait for writing reverse γ codes.
pub trait GammaRevWrite<E: Endianness>: BitWrite<E> {
    fn write_rev_gamma(&mut self, n: u64) -> Result<usize, Self::Error>;
}

impl<B: BitWrite<BE>> GammaRevWrite<BE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_rev_gamma(&mut self, n: u64) -> Result<usize, Self::Error> {
        if let Some(len) = write_table_be(self, n)? {
            return Ok(len);
        }
        default_rev_write_gamma(self, n)
    }
}

impl<B: BitWrite<LE>> GammaRevWrite<LE> for B {
    #[inline]
    #[allow(clippy::collapsible_if)]
    fn write_rev_gamma(&mut self, n: u64) -> Result<usize, Self::Error> {
        if let Some(len) = write_table_le(self, n)? {
            return Ok(len);
        }
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

pub struct RevBuffer<P: AsRef<Path>> {
    path: P,
    bit_writer: BufBitWriter<BE, WordAdapter<u64, BufWriter<File>>>,
    len: u64,
}

impl<P: AsRef<Path>> RevBuffer<P> {
    pub fn new(path: P) -> anyhow::Result<Self> {
        let bit_writer = BufBitWriter::new(WordAdapter::new(BufWriter::new(File::create(
            path.as_ref(),
        )?)));

        Ok(Self {
            path,
            bit_writer,
            len: 0,
        })
    }

    pub fn push(&mut self, x: u64) -> anyhow::Result<()> {
        self.bit_writer.write_rev_gamma(x)?;
        self.len += 1;
        Ok(())
    }

    pub fn flush(&mut self) -> anyhow::Result<Iterable> {
        Ok(Iterable {
            len: self.len,
            padding: (u64::BITS as usize - self.bit_writer.flush()?) % usize::BITS as usize,
            mmap: MmapHelper::mmap(&self.path, MmapFlags::SEQUENTIAL)?,
        })
    }
}

pub struct Iterable {
    len: u64,
    padding: usize,
    mmap: MmapHelper<u32>,
}

impl<'a> IntoIterator for &'a Iterable {
    type Item = u64;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let mut bit_reader = BufBitReader::<LE, _>::new(RevReader::new(self.mmap.as_ref()));
        bit_reader.skip_bits(self.padding).unwrap();

        IntoIter {
            pos: 0,
            len: self.len,
            bit_reader,
        }
    }
}

pub struct IntoIter<'a> {
    pos: u64,
    len: u64,
    bit_reader: BufBitReader<LE, RevReader<'a>>,
}

impl Iterator for IntoIter<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.len {
            None
        } else {
            self.pos += 1;
            // RevReader is infallible
            Some(self.bit_reader.read_gamma().unwrap())
        }
    }
}

impl FusedIterator for IntoIter<'_> {}

struct RevReader<'a> {
    data: &'a [u32],
    position: usize,
}

impl<'a> RevReader<'a> {
    pub fn new(data: &'a [u32]) -> Self {
        Self {
            data,
            position: data.len(),
        }
    }
}

impl WordRead for RevReader<'_> {
    type Error = std::convert::Infallible;
    type Word = u32;

    fn read_word(&mut self) -> Result<u32, Self::Error> {
        if self.position == 0 {
            Ok(0)
        } else {
            self.position -= 1;
            let w = self.data[self.position].to_be();
            Ok(w)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rev() -> anyhow::Result<()> {
        use rand::rngs::SmallRng;
        use rand::RngCore;
        use rand::SeedableRng;
        let tmp = tempfile::NamedTempFile::new()?;
        let mut rev_writer = RevBuffer::new(tmp)?;

        let mut v = vec![];

        let mut r = SmallRng::seed_from_u64(42);

        for _ in 0..100000 {
            let x = r.next_u64() % 1024;
            v.push(x);
            rev_writer.push(x);
        }

        let iterable = rev_writer.flush()?;
        let mut into_iter = iterable.into_iter();

        for &x in v.iter().rev() {
            let y = into_iter.next().unwrap();
            assert_eq!(y, x);
        }

        let mut into_iter = iterable.into_iter();

        for &x in v.iter().rev() {
            let y = into_iter.next().unwrap();
            assert_eq!(y, x);
        }

        Ok(())
    }

    #[test]
    fn test_no_flush() -> anyhow::Result<()> {
        let tmp = tempfile::NamedTempFile::new()?;
        let mut rev_writer = RevBuffer::new(tmp)?;
        for _ in 0..42 {
            rev_writer.push(1)?;
        }

        for _ in 0..2 {
            rev_writer.push(0)?;
        }

        let iterable = rev_writer.flush()?;
        let mut into_iter = iterable.into_iter();
        for _ in 0..2 {
            assert_eq!(into_iter.next().unwrap(), 0);
        }

        for _ in 0..42 {
            assert_eq!(into_iter.next().unwrap(), 1);
        }

        Ok(())
    }
}
