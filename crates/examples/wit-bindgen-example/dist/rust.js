import { data_view, UTF8_DECODER, utf8_encode, UTF8_ENCODED_LEN } from './intrinsics.js';
export class Rust {
  addToImports(imports) {
  }
  
  async instantiate(module, imports) {
    imports = imports || {};
    this.addToImports(imports);
    
    if (module instanceof WebAssembly.Instance) {
      this.instance = module;
    } else if (module instanceof WebAssembly.Module) {
      this.instance = await WebAssembly.instantiate(module, imports);
    } else if (module instanceof ArrayBuffer || module instanceof Uint8Array) {
      const { instance } = await WebAssembly.instantiate(module, imports);
      this.instance = instance;
    } else {
      const { instance } = await WebAssembly.instantiateStreaming(module, imports);
      this.instance = instance;
    }
    this._exports = this.instance.exports;
  }
  capitalize(arg0) {
    const memory = this._exports.memory;
    const realloc = this._exports["canonical_abi_realloc"];
    const free = this._exports["canonical_abi_free"];
    const ptr0 = utf8_encode(arg0, realloc, memory);
    const len0 = UTF8_ENCODED_LEN;
    const ret = this._exports['capitalize'](ptr0, len0);
    const ptr1 = data_view(memory).getInt32(ret + 0, true);
    const len1 = data_view(memory).getInt32(ret + 8, true);
    const list1 = UTF8_DECODER.decode(new Uint8Array(memory.buffer, ptr1, len1));
    free(ptr1, len1, 1);
    return list1;
  }
}
