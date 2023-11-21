(module
  (import "" "call-host" (func $call-host (param i32)))
  (func (export "run") (result i32)
    i32.const 2
    call $call-host
    i32.const 42
  )
  (func (export "run2")
    i32.const -2
    call $call-host
  )
)
