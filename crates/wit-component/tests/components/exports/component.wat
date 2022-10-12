(component
  (type (;0;) (func))
  (type (;1;) (func (param "a" s8) (param "b" s16) (param "c" s32) (param "d" s64) (result string)))
  (type (;2;) (tuple s8 s16 s32 s64))
  (type (;3;) (func (result 2)))
  (type (;4;) (flags "a" "b" "c"))
  (type (;5;) (func (param "x" 4)))
  (type (;6;) (variant (case "a") (case "b" string) (case "c" s64)))
  (type (;7;) (func (param "x" string) (result 6)))
  (type (;8;) (func (param "x" 6) (result string)))
  (core module (;0;)
    (type (;0;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;1;) (func))
    (type (;2;) (func (param i32 i32 i32 i64) (result i32)))
    (type (;3;) (func (param i32)))
    (type (;4;) (func (result i32)))
    (type (;5;) (func (param i32 i32) (result i32)))
    (type (;6;) (func (param i32 i64 i32) (result i32)))
    (func (;0;) (type 0) (param i32 i32 i32 i32) (result i32)
      unreachable
    )
    (func (;1;) (type 1)
      unreachable
    )
    (func (;2;) (type 2) (param i32 i32 i32 i64) (result i32)
      unreachable
    )
    (func (;3;) (type 3) (param i32)
      unreachable
    )
    (func (;4;) (type 4) (result i32)
      unreachable
    )
    (func (;5;) (type 1)
      unreachable
    )
    (func (;6;) (type 5) (param i32 i32) (result i32)
      unreachable
    )
    (func (;7;) (type 3) (param i32)
      unreachable
    )
    (func (;8;) (type 6) (param i32 i64 i32) (result i32)
      unreachable
    )
    (func (;9;) (type 3) (param i32)
      unreachable
    )
    (func (;10;) (type 3) (param i32)
      unreachable
    )
    (memory (;0;) 1)
    (export "memory" (memory 0))
    (export "cabi_realloc" (func 0))
    (export "a" (func 1))
    (export "b" (func 2))
    (export "cabi_post_b" (func 3))
    (export "c" (func 4))
    (export "foo#a" (func 5))
    (export "foo#b" (func 6))
    (export "cabi_post_foo#b" (func 7))
    (export "foo#c" (func 8))
    (export "cabi_post_foo#c" (func 9))
    (export "bar#a" (func 10))
  )
  (core instance (;0;) (instantiate 0))
  (alias core export 0 "memory" (core memory (;0;)))
  (alias core export 0 "cabi_realloc" (core func (;0;)))
  (alias core export 0 "a" (core func (;1;)))
  (func (;0;) (type 0) (canon lift (core func 1)))
  (alias core export 0 "b" (core func (;2;)))
  (alias core export 0 "cabi_post_b" (core func (;3;)))
  (func (;1;) (type 1) (canon lift (core func 2) (memory 0) string-encoding=utf8 (post-return 3)))
  (alias core export 0 "c" (core func (;4;)))
  (func (;2;) (type 3) (canon lift (core func 4) (memory 0)))
  (export "a" (func 0))
  (export "b" (func 1))
  (export "c" (func 2))
  (alias core export 0 "bar#a" (core func (;5;)))
  (func (;3;) (type 5) (canon lift (core func 5)))
  (instance (;0;)
    (export "x" (type 4))
    (export "a" (func 3))
  )
  (export "bar" (instance 0))
  (alias core export 0 "foo#a" (core func (;6;)))
  (func (;4;) (type 0) (canon lift (core func 6)))
  (alias core export 0 "foo#b" (core func (;7;)))
  (alias core export 0 "cabi_post_foo#b" (core func (;8;)))
  (func (;5;) (type 7) (canon lift (core func 7) (memory 0) (realloc 0) string-encoding=utf8 (post-return 8)))
  (alias core export 0 "foo#c" (core func (;9;)))
  (alias core export 0 "cabi_post_foo#c" (core func (;10;)))
  (func (;6;) (type 8) (canon lift (core func 9) (memory 0) (realloc 0) string-encoding=utf8 (post-return 10)))
  (instance (;1;)
    (export "x" (type 6))
    (export "a" (func 4))
    (export "b" (func 5))
    (export "c" (func 6))
  )
  (export "foo" (instance 1))
)