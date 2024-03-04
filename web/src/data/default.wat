(module
  (func (export "next")
    ;; The parameters are the neighborhood values in row-major order. $p11 is
    ;; the previous value of the current cell.
    ;;
    ;; The bits of each value are the RGBA (8 bits for each channel).
    (param $p00 i32) (param $p01 i32) (param $p02 i32)
    (param $p10 i32) (param $p11 i32) (param $p12 i32)
    (param $p20 i32) (param $p21 i32) (param $p22 i32)

    (result i32)

    (local.get $p11)
    (local.get $p00) i32.xor ;; UL
    (local.get $p02) i32.xor ;; UR
    (local.get $p20) i32.xor ;; DL
    (local.get $p22) i32.xor ;; DR
  )
)
