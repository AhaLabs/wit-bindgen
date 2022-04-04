(component
  (type (;0;) (func (param "x" string)))
  (type (;1;) (record (field "a" u8)))
  (type (;2;) (func (param "x" (type 1))))
  (type (;3;) 
    (instance
      (alias outer 1 0 (type (;0;)))
      (export "bar1" (type 0))
      (alias outer 1 1 (type (;1;)))
      (export "x" (type 1))
      (alias outer 1 2 (type (;2;)))
      (export "bar2" (type 2))
    )
  )
  (type (;4;) (list string))
  (type (;5;) (func (param "x" (type 4))))
  (type (;6;) (func))
  (type (;7;) s8)
  (type (;8;) (func (param "x" (type 7))))
  (type (;9;) 
    (instance
      (alias outer 1 5 (type (;0;)))
      (export "baz1" (type 0))
      (alias outer 1 6 (type (;1;)))
      (export "baz2" (type 1))
      (alias outer 1 7 (type (;2;)))
      (export "x" (type 2))
      (alias outer 1 8 (type (;3;)))
      (export "baz3" (type 3))
    )
  )
  (type (;10;) (func (param "x" u8)))
  (type (;11;) (func (param "x" float32)))
  (type (;12;) 
    (instance
      (alias outer 1 6 (type (;0;)))
      (export "foo1" (type 0))
      (alias outer 1 10 (type (;1;)))
      (export "foo2" (type 1))
      (alias outer 1 11 (type (;2;)))
      (export "foo3" (type 2))
    )
  )
  (module (;0;)
    (type (;0;) (func))
    (type (;1;) (func (param i32)))
    (type (;2;) (func (param f32)))
    (type (;3;) (func (param i32 i32)))
    (type (;4;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;5;) (func (param i32 i32 i32)))
    (import "foo" "foo1" (func (;0;) (type 0)))
    (import "foo" "foo2" (func (;1;) (type 1)))
    (import "foo" "foo3" (func (;2;) (type 2)))
    (import "bar" "bar1" (func (;3;) (type 3)))
    (import "bar" "bar2" (func (;4;) (type 1)))
    (import "baz" "baz1" (func (;5;) (type 3)))
    (import "baz" "baz2" (func (;6;) (type 0)))
    (import "baz" "baz3" (func (;7;) (type 1)))
    (func (;8;) (type 4) (param i32 i32 i32 i32) (result i32)
      unreachable
    )
    (func (;9;) (type 5) (param i32 i32 i32)
      unreachable
    )
    (memory (;0;) 1)
    (export "memory" (memory 0))
    (export "canonical_abi_realloc" (func 8))
    (export "canonical_abi_free" (func 9))
  )
  (import "bar" (instance (;0;) (type 3)))
  (import "baz" (instance (;1;) (type 9)))
  (import "foo" (instance (;2;) (type 12)))
  (module (;1;)
    (type (;0;) (func (param i32 i32)))
    (func (;0;) (type 0) (param i32 i32)
      local.get 0
      local.get 1
      i32.const 0
      call_indirect (type 0)
    )
    (func (;1;) (type 0) (param i32 i32)
      local.get 0
      local.get 1
      i32.const 1
      call_indirect (type 0)
    )
    (table (;0;) 2 2 funcref)
    (export "0" (func 0))
    (export "1" (func 1))
    (export "$imports" (table 0))
  )
  (module (;2;)
    (type (;0;) (func (param i32 i32)))
    (import "" "0" (func (;0;) (type 0)))
    (import "" "1" (func (;1;) (type 0)))
    (import "" "$imports" (table (;0;) 2 2 funcref))
    (elem (;0;) (i32.const 0) func 0 1)
  )
  (instance (;3;) (instantiate (module 1)))
  (alias export (instance 3) "0" (func (;0;)))
  (alias export (instance 0) "bar2" (func (;1;)))
  (alias export (instance 3) "1" (func (;2;)))
  (alias export (instance 1) "baz2" (func (;3;)))
  (alias export (instance 1) "baz3" (func (;4;)))
  (alias export (instance 2) "foo1" (func (;5;)))
  (alias export (instance 2) "foo2" (func (;6;)))
  (alias export (instance 2) "foo3" (func (;7;)))
  (func (;8;) (canon.lower (func 1)))
  (func (;9;) (canon.lower (func 3)))
  (func (;10;) (canon.lower (func 4)))
  (func (;11;) (canon.lower (func 5)))
  (func (;12;) (canon.lower (func 6)))
  (func (;13;) (canon.lower (func 7)))
  (instance (;4;) core (export "bar1" (func 0)) (export "bar2" (func 8)))
  (instance (;5;) core (export "baz1" (func 2)) (export "baz2" (func 9)) (export "baz3" (func 10)))
  (instance (;6;) core (export "foo1" (func 11)) (export "foo2" (func 12)) (export "foo3" (func 13)))
  (instance (;7;) (instantiate (module 0) (with "bar" (instance 4)) (with "baz" (instance 5)) (with "foo" (instance 6))))
  (alias export (instance 3) "$imports" (table (;0;)))
  (alias export (instance 0) "bar1" (func (;14;)))
  (alias export (instance 1) "baz1" (func (;15;)))
  (func (;16;) (canon.lower utf8 (into (instance 7)) (func 14)))
  (func (;17;) (canon.lower utf8 (into (instance 7)) (func 15)))
  (instance (;8;) core (export "$imports" (table 0)) (export "0" (func 16)) (export "1" (func 17)))
  (instance (;9;) (instantiate (module 2) (with "" (instance 8))))
)