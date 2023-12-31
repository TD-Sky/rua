use std::borrow::Cow;
use std::mem;
use std::rc::Rc;

use tinyvec::TinyVec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LossyStr(Repr);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Repr {
    Inline {
        len: InlineSize,
        buf: [u8; LossyStr::INLINE_CAP],
    },
    Heap(Rc<[u8]>),
}

impl From<TinyVec<[u8; LossyStr::INLINE_CAP]>> for LossyStr {
    fn from(v: TinyVec<[u8; LossyStr::INLINE_CAP]>) -> Self {
        match v {
            TinyVec::Inline(v) => Self(Repr::Inline {
                len: unsafe { mem::transmute(v.len() as u8) },
                buf: v.into_inner(),
            }),
            TinyVec::Heap(v) => Self(Repr::Heap(Rc::from(v))),
        }
    }
}

impl LossyStr {
    pub const INLINE_CAP: usize = 23;

    pub fn to_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self(Repr::Inline { len, buf }) => &buf[..(*len as u8 as usize)],
            Self(Repr::Heap(v)) => v.as_ref(),
        }
    }
}

impl std::fmt::Display for LossyStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_str())
    }
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineSize {
    V0 = 0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    V10,
    V11,
    V12,
    V13,
    V14,
    V15,
    V16,
    V17,
    V18,
    V19,
    V20,
    V21,
    V22,
    V23,
}
