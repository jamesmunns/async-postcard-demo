fn main() {
    println!("Hello, world!");
}

use std::marker::PhantomData;
use std::ops::DerefMut;

use postcard::ser_flavors::Flavor as SerFlavor;
use postcard::de_flavors::Flavor as DeFlavor;
use postcard::Result as PCResult;

pub struct LenFramedSer<F> {
    inner: F,
}

impl<F> LenFramedSer<F>
where
    F: SerFlavor,
{
    pub fn new(mut flav: F) -> PCResult<Self> {
        // Placeholder for length
        flav.try_extend(&0u32.to_le_bytes())?;

        Ok(Self {
            inner: flav
        })
    }
}

impl<F> SerFlavor for LenFramedSer<F>
where
    F: SerFlavor,
    F::Output: DerefMut<Target = [u8]>,
{
    type Output = F::Output;

    fn try_push(&mut self, data: u8) -> PCResult<()> {
        self.inner.try_push(data)
    }

    fn finalize(self) -> PCResult<Self::Output> {
        let mut out = self.inner.finalize()?;
        // Fill in placeholder, MINUS the length of length itself
        let len = out.len();
        out[0..4].copy_from_slice(&((len - 4) as u32).to_le_bytes());
        Ok(out)
    }

    fn try_extend(&mut self, data: &[u8]) -> PCResult<()> {
        self.inner.try_extend(data)
    }
}

pub struct LenFramedDe<'de, F>
where
    F: DeFlavor<'de> + 'de
{
    inner: F,
    _plt: PhantomData<&'de ()>,
}

impl<'de, F> LenFramedDe<'de, F>
where
    F: DeFlavor<'de>
{
    pub fn new(mut flav: F) -> PCResult<(usize, Self)> {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(flav.try_take_n(4)?);
        let len = u32::from_le_bytes(buf) as usize;
        let me = Self { inner: flav, _plt: PhantomData };
        Ok((len, me))
    }
}

impl<'de, F> DeFlavor<'de> for LenFramedDe<'de, F>
where
    F: DeFlavor<'de>
{
    type Remainder = F::Remainder;
    type Source = F;

    fn pop(&mut self) -> PCResult<u8> {
        self.inner.pop()
    }

    fn try_take_n(&mut self, ct: usize) -> PCResult<&'de [u8]> {
        self.inner.try_take_n(ct)
    }

    fn finalize(self) -> PCResult<Self::Remainder> {
        self.inner.finalize()
    }
}
