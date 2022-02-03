import { UTF8_DECODER } from './intrinsics.js';
export function addJavascriptToImports(imports, obj, get_export) {
  if (!("javascript" in imports)) imports["javascript"] = {};
  imports["javascript"]["print"] = function(arg0, arg1) {
    const memory = get_export("memory");
    const ptr0 = arg0;
    const len0 = arg1;
    obj.print(UTF8_DECODER.decode(new Uint8Array(memory.buffer, ptr0, len0)));
  };
}