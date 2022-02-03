import * as path from "path";
import * as fs from "fs/promises";
import { fileURLToPath } from "url"; // the node package 'url'
function dirname(meta) {
    return path.dirname(fileURLToPath(meta.url));
}
// call with import.meta
export const __dirname = dirname(import.meta);
export function getWasm() {
    return fs.readFile(`${__dirname}/rust.wasm`);
}
