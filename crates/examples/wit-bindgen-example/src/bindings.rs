mod printer {
  pub fn print(msg: & str,){
    unsafe {
      let vec0 = msg;
      let ptr0 = vec0.as_ptr() as i32;
      let len0 = vec0.len() as i32;
      #[link(wasm_import_module = "printer")]
      extern "C" {
        #[cfg_attr(target_arch = "wasm32", link_name = "print")]
        #[cfg_attr(not(target_arch = "wasm32"), link_name = "printer_print")]
        fn wit_import(_: i32, _: i32, );
      }
      wit_import(ptr0, len0);
    }
  }
}
pub mod stringfuncs {
  #[export_name = "capitalize"]
  unsafe extern "C" fn __wit_bindgen_capitalize(arg0: i32, arg1: i32, ) -> i32{
    let len0 = arg1 as usize;
    let result1 = <super::Stringfuncs as Stringfuncs>::capitalize(String::from_utf8(Vec::from_raw_parts(arg0 as *mut _, len0, len0)).unwrap());
    let vec2 = (result1.into_bytes()).into_boxed_slice();
    let ptr2 = vec2.as_ptr() as i32;
    let len2 = vec2.len() as i32;
    core::mem::forget(vec2);
    let ptr3 = RET_AREA.as_mut_ptr() as i32;
    *((ptr3 + 8) as *mut i32) = len2;
    *((ptr3 + 0) as *mut i32) = ptr2;
    ptr3
  }
  pub trait Stringfuncs {
    fn capitalize(s: String,) -> String;
  }
  static mut RET_AREA: [i64; 2] = [0; 2];
}