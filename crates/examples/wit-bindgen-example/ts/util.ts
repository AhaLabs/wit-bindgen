import * as path from "node:path";
import * as fs from "node:fs/promises";
import { fileURLToPath } from "node:url"; // the node package 'url'

function dirname(meta: any) {
  return path.dirname(fileURLToPath(meta.url));
}

// call with import.meta
export const __dirname = dirname(import.meta);

export function getWasm(): Promise<Buffer> {
  return fs.readFile(`${__dirname}/rust.wasm`);
}