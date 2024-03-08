(module
  (func (export "next")
    (param $p00 i32) (param $p01 i32) (param $p02 i32)
    (param $p10 i32) (param $p11 i32) (param $p12 i32)
    (param $p20 i32) (param $p21 i32) (param $p22 i32)

    (result i32)

    (local.get $p11)         ;; self
    (local.get $p00) i32.xor ;; XOR UL
    (local.get $p02) i32.xor ;; XOR UR
    (local.get $p20) i32.xor ;; XOR DL
    (local.get $p22) i32.xor ;; XOR DR
  )
)
