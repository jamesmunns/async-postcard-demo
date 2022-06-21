fn main() {
    println!("Hello, world!");
}

use std::ops::DerefMut;

use postcard::ser_flavors::Flavor as SerFlavor;
use postcard::Result as PCResult;
use serde::{Serialize, Deserialize};

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Outer {
    foo: (u8, u32),
    bar: Vec<String>,
    baz: Vec<Inner>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Inner {
    a: u32,
    b: u64,
    c: Vec<u8>,
}

#[cfg(test)]
mod test {
    use super::*;
    use postcard::serialize_with_flavor;
    use postcard::ser_flavors::StdVec;
    use postcard::de_flavors::Slice;

    #[test]
    fn smoke() {
        let data = Outer {
            foo: (1, 2),
            bar: vec!["Hello".into(), "World!".into()],
            baz: vec![
                Inner { a: 3, b: 4, c: vec![5, 6, 7] },
                Inner { a: 8, b: 9, c: vec![0, 1, 2] },
                Inner { a: 0, b: 1, c: vec![2, 3, 4] },
            ],
        };

        let serdata = serialize_with_flavor::<Outer, LenFramedSer<StdVec>, Vec<u8>>(
            &data,
            LenFramedSer::new(StdVec::new()).unwrap(),
        ).unwrap();

        println!("{}, {:?}", serdata.len(), serdata);

        // Sender task psuedocode:
        //
        // tcp.send(&data).await?;

        // ...

        // Receiver task psuedocode (I don't know how async streams work...):
        //
        // loop {
        //     let len = u32::from_le_bytes(tcp.recv(4).await) as usize;
        //     let serdata = tcp.recv(len).await.to_vec();
        //     yield the_code_below()?;
        // }


        let flav = Slice::new(&serdata[4..]);
        let mut deser = postcard::Deserializer::from_flavor(flav);
        let dedata = Outer::deserialize(&mut deser).unwrap();
        assert_eq!(data, dedata);
    }
}


// TODO: I thought I needed this, but I really don't. With this simple data, it's easier to
// just await the len first

// use std::marker::PhantomData;
// use postcard::de_flavors::Flavor as DeFlavor;

// pub struct LenFramedDe<'de, F>
// where
//     F: DeFlavor<'de> + 'de
// {
//     inner: F,
//     _plt: PhantomData<&'de ()>,
// }

// impl<'de, F> LenFramedDe<'de, F>
// where
//     F: DeFlavor<'de>
// {
//     pub fn new(mut flav: F) -> PCResult<(usize, Self)> {
//         let mut buf = [0u8; 4];
//         buf.copy_from_slice(flav.try_take_n(4)?);
//         let len = u32::from_le_bytes(buf) as usize;
//         let me = Self { inner: flav, _plt: PhantomData };
//         Ok((len, me))
//     }
// }

// impl<'de, F> DeFlavor<'de> for LenFramedDe<'de, F>
// where
//     F: DeFlavor<'de>
// {
//     type Remainder = F::Remainder;
//     type Source = F;

//     fn pop(&mut self) -> PCResult<u8> {
//         self.inner.pop()
//     }

//     fn try_take_n(&mut self, ct: usize) -> PCResult<&'de [u8]> {
//         self.inner.try_take_n(ct)
//     }

//     fn finalize(self) -> PCResult<Self::Remainder> {
//         self.inner.finalize()
//     }
// }
