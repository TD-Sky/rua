#[derive(Debug)]
pub enum ByteCode {
    GetGlobal(u8, u8),       // A  Bx   R[A] := G[K[Bx]]
    Move(u8, u8),            // A  B    R[A] := R[B]
    LoadConst(u8, u8),       // A  Bx   R[A] := K[Bx]
    LoadNil(u8),             // A  B    R[A], R[A+1], ..., R[A+B] := nil
    LoadBool(u8, bool),      // A  B    R[A] := B
    LoadInt(u8, i16),        // A  B    R[A] := B
    Call(u8, u8),            // A  B    R[A] := R[A](R[A+1], ... ,R[A+B-1])
    SetGlobalConst(u8, u8),  // Ax Bx   G[K[Ax]] := K[Bx]
    SetGlobalLocal(u8, u8),  // Ax B    G[K[Ax]] := R[B]
    SetGlobalGlobal(u8, u8), // Ax Bx   G[K[Ax]] := G[K[Bx]]
}

pub struct ByteCodeStack<'a>(pub &'a [ByteCode]);

impl std::fmt::Display for ByteCodeStack<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for code in self.0 {
            writeln!(f, "    {code:?}")?;
        }
        Ok(())
    }
}
