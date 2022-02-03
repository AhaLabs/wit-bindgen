import path from "path";
import fs from "fs";

// inspired by https://github.com/muturgan/typescript-esm-example/blob/7f0543ed2b3e365aae17ccc8b9cc3d442e7a7956/buildtools/fix-imports.js

const START_PATH = path.join(process.cwd(), "dist");
const IS_NODE_IMPORT = /.*['"]node:.*["']/
const ALREADY_FIXED_REGEXP =
  /^(import|export)( [^';]* from )(['"])([^'"]+\.js)['"]/g;
const IMPORT_REGEXP = /^(import|export)( [^';]* from )(['"])(?<importPath>[^'"]+)['"]/g;
const JUST_ADD_AN_EXTENSION = "$1$2$3$4.js$3";
const JS_EXT = ".js";

function fixImportsAtFolder(rootPath) {
  const entries = fs.readdirSync(rootPath);
  entries.forEach((entry) => {
    const entryPath = path.join(rootPath, entry);
    if (entry.endsWith(JS_EXT)) {
      fixImportsAtFile(entryPath);
    } else {
      const extName = path.extname(entry);
      if (!extName) {
        const stat = fs.statSync(entryPath);
        if (stat.isDirectory()) {
          fixImportsAtFolder(entryPath);
        }
      }
    }
  });
}

function fixImportsAtFile(filePath) {
  const content = fs.readFileSync(filePath).toString("utf8");
  const lines = content.split("\n");
  const fixedLines = lines.map((l) => {
    let match = l.match(IMPORT_REGEXP);
    if (!match || l.match(ALREADY_FIXED_REGEXP) || l.match(IS_NODE_IMPORT)) {
      return l;
    }
    const {importPath} = match.groups;
    if (!importPath.startsWith(".")) { return l;}
    const fullPath = path.join(filePath, '..', importPath);
    const exists = fs.existsSync(fullPath);
    if (exists === false) {
    return l.replace(IMPORT_REGEXP, JUST_ADD_AN_EXTENSION);
    }

    const stat = fs.statSync(fullPath);
    const isDirectory = stat.isDirectory();
    if (isDirectory === true) {
      return l.replace(IMPORT_REGEXP, ADD_INDEX_FILE);
    }

    return l;
  });
  const withFixedImports = fixedLines.join("\n");
  fs.writeFileSync(filePath, withFixedImports);
}

fixImportsAtFolder(START_PATH);
console.log("imports fixed...");
console.log("================");
