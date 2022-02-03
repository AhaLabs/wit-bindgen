import { Rust } from "../dist/rust.js";
import { addJavascriptToImports } from "../dist/javascript.js";
import { getWasm } from "./util.js";
async function instantiateBinary() {
    // Create initial class for rust wasm binary
    const rust = new Rust();
    // Implement the import required by the binary
    const js = {
        print: console.log,
    };
    // Create empty imports
    const imports = {};
    // call generated function that wraps the print function to allow passing string back from wasm side.
    addJavascriptToImports(imports, js, (name) => rust.instance.exports[name]);
    // Instantiate the wasm binary with the required imports
    await rust.instantiate(await getWasm(), imports);
    return rust;
}
async function main() {
    const rust = await instantiateBinary();
    const str = "hello world";
    const capitalized = rust.capitalize(str);
    if (capitalized !== str.toUpperCase()) {
        throw new Error(`String was not capitalized.  Expected ${str.toUpperCase()}, got ${capitalized}`);
    }
    console.log(capitalized);
}
void main();
