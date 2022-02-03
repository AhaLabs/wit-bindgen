export function addJavascriptToImports(imports: any, obj: Javascript, get_export: (name: string) => WebAssembly.ExportValue): void;
export interface Javascript {
  print(msg: string): void;
}
