// SPDX-License-Identifier: MIT OR Apache-2.0
/// Element types. Half precision and complex types are not offered yet (no vendor provider has them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Dtype {
    /// 32-bit float.
    F32,
    /// 64-bit float.
    F64,
}

/// Operations a provider may implement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Op {
    /// `C = alpha * op(A) * op(B) + beta * C`.
    Gemm,
}

/// Whether an operand is used as stored or transposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trans {
    /// Use the matrix as stored.
    No,
    /// Use the transpose.
    Yes,
}

/// One (operation, element type) a provider claims to implement.
///
/// A claim is data to print and to generate tests from; it is not evidence. The conformance suite
/// checks it, and consumers (Fuel) verify accuracy and determinism on their own terms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capability {
    /// The operation.
    pub op: Op,
    /// The element type.
    pub dtype: Dtype,
    /// Free-text caveats (threading, determinism, ...).
    pub notes: &'static str,
}

/// The configuration actually in force for a provider, not what was requested.
///
/// ULP error and bit-stability depend on thread count and instruction set, so consumers need to see them.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Config {
    /// Threads the provider will use, if known.
    pub threads: Option<usize>,
    /// Instruction-set level or kernel family in use, if known.
    pub isa: Option<String>,
}

/// Arguments for `C = alpha * op(A) * op(B) + beta * C`, row-major with explicit leading dimensions.
///
/// `op(A)` is `m x k` and `op(B)` is `k x n`. When `beta` is zero, `c` is not read (it may hold NaN).
/// Stored `A` is `m x k` if `trans_a` is [`Trans::No`], otherwise `k x m`; likewise for `B`.
#[derive(Debug)]
pub struct GemmArgs<'a, T> {
    /// Transposition of `A`.
    pub trans_a: Trans,
    /// Transposition of `B`.
    pub trans_b: Trans,
    /// Rows of `op(A)` and `C`.
    pub m: usize,
    /// Columns of `op(B)` and `C`.
    pub n: usize,
    /// Columns of `op(A)`, rows of `op(B)`.
    pub k: usize,
    /// Scale for the product.
    pub alpha: T,
    /// Matrix `A` as stored.
    pub a: &'a [T],
    /// Row stride of stored `A`.
    pub lda: usize,
    /// Matrix `B` as stored.
    pub b: &'a [T],
    /// Row stride of stored `B`.
    pub ldb: usize,
    /// Scale for the existing `C`.
    pub beta: T,
    /// Matrix `C`, `m x n`.
    pub c: &'a mut [T],
    /// Row stride of `C`.
    pub ldc: usize,
}

impl<T> GemmArgs<'_, T> {
    /// Check the leading dimensions and buffer lengths. Providers call this before touching memory.
    pub fn validate(&self) -> Result<(), crate::Error> {
        let (ar, ac) = match self.trans_a {
            Trans::No => (self.m, self.k),
            Trans::Yes => (self.k, self.m),
        };
        let (br, bc) = match self.trans_b {
            Trans::No => (self.k, self.n),
            Trans::Yes => (self.n, self.k),
        };
        check("A", ar, ac, self.lda, self.a.len())?;
        check("B", br, bc, self.ldb, self.b.len())?;
        check("C", self.m, self.n, self.ldc, self.c.len())
    }
}

fn check(name: &str, rows: usize, cols: usize, ld: usize, len: usize) -> Result<(), crate::Error> {
    if rows == 0 || cols == 0 {
        return Ok(());
    }
    if ld < cols {
        return Err(crate::Error::InvalidArgument(format!(
            "{name}: leading dimension {ld} < columns {cols}"
        )));
    }
    let needed = (rows - 1)
        .checked_mul(ld)
        .and_then(|v| v.checked_add(cols))
        .ok_or_else(|| crate::Error::InvalidArgument(format!("{name}: size overflows")))?;
    if len < needed {
        return Err(crate::Error::InvalidArgument(format!(
            "{name}: buffer holds {len} elements, {needed} needed"
        )));
    }
    Ok(())
}
