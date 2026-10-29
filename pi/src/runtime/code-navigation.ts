/** Scope-aware navigation in an explicit, immutable, memory-only source model. */
import { createHash } from "node:crypto";
import ts from "typescript";

export const CODE_NAVIGATION_LIMITS = Object.freeze({ sources: 16, project_sources: 8, declarations: 8, imports: 128, specifier_bytes: 256, source_bytes: 262_144,
  total_bytes: 1_048_576, nodes: 32_768, depth: 64, locations: 100, response_bytes: 32_768 });
export type CodeLanguage = "js" | "ts" | "jsx" | "tsx" | "ts_declaration";
export interface NavigationSource { language: CodeLanguage; source_utf8: string; expected_revision: string }
export interface ImportBinding { source: number; specifier: string; target: number }
export interface TypeScriptModel { imports: ImportBinding[] }
export interface NavigationRequest {
  sources: NavigationSource[];
  typescript_model: TypeScriptModel;
  source: number;
  position_byte: number;
  operation: "definitions" | "references";
  limit: number;
  max_response_bytes: number;
}
interface Span { start_byte: number; end_byte: number }
interface Location { source: number; revision: string; location: Span; is_write?: boolean }
interface SourceIdentity { source: number; revision: string; source_bytes: number }
export interface NavigationResult {
  status: "locations" | "changed" | "refused" | "budget_blocked";
  backend: { name: "typescript-language-service"; version: "5.9.3" };
  sources: SourceIdentity[];
  model_revision: string;
  model_complete: false;
  coverage: string[];
  locations?: Location[];
  complete?: boolean;
  reason?: string;
}
const encoder = new TextEncoder();
const scriptKinds = Object.freeze({ js: ts.ScriptKind.JS, ts: ts.ScriptKind.TS,
  jsx: ts.ScriptKind.JSX, tsx: ts.ScriptKind.TSX, ts_declaration: ts.ScriptKind.TS });
const coverage = ["TypeScript 5.9.3 language service over only the supplied immutable source snapshots",
  "scope-aware definitions/references across explicitly selected project and declaration snapshots; complete describes only query truncation",
  "only explicit source-specific import bindings; no configuration/package discovery, plugins, default libraries, type acquisition or execution",
  "selected model is incomplete; missing/ambiguous/unused import bindings and unverified or unresolved definitions are refused"];
class Refusal extends Error { constructor(readonly reason: string) { super(reason); } }
function fail(reason: string): never { throw new Refusal(reason); }
function exact(value: unknown, keys: string[]): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value) && Object.keys(value).length === keys.length &&
    keys.every((key) => Object.hasOwn(value, key));
}
function size(value: unknown): number { return encoder.encode(JSON.stringify(value)).length; }
function offsets(text: string): Int32Array {
  const table = new Int32Array(text.length + 1).fill(-1);
  let utf16 = 0, utf8 = 0; table[0] = 0;
  for (const character of text) {
    utf8 += encoder.encode(character).length; utf16 += character.length; table[utf16] = utf8;
  }
  return table;
}
function span(table: Int32Array, textSpan: ts.TextSpan): Span {
  const start = textSpan.start, end = start + textSpan.length;
  if (!Number.isSafeInteger(start) || !Number.isSafeInteger(end) || start < 0 || end <= start || end >= table.length ||
      table[start]! < 0 || table[end]! < 0) fail("unproved_original_location");
  return { start_byte: table[start]!, end_byte: table[end]! };
}

/** No IO is performed here. The parent owns saved-file scope, revisions and process limits. */
export function navigateCode(request: NavigationRequest): NavigationResult {
  const result: NavigationResult = { status: "refused", backend: { name: "typescript-language-service", version: "5.9.3" },
    sources: [], model_revision: "", model_complete: false, coverage: [...coverage] };
  let service: ts.LanguageService | undefined;
  try {
    if (!exact(request, ["sources", "typescript_model", "source", "position_byte", "operation", "limit", "max_response_bytes"]) ||
        !Array.isArray(request.sources) || request.sources.length < 1 || request.sources.length > CODE_NAVIGATION_LIMITS.sources ||
        !exact(request.typescript_model, ["imports"]) || !Array.isArray(request.typescript_model.imports) ||
        request.typescript_model.imports.length > CODE_NAVIGATION_LIMITS.imports ||
        !Number.isSafeInteger(request.source) || request.source < 0 || request.source >= request.sources.length ||
        !Number.isSafeInteger(request.position_byte) || request.position_byte < 0 ||
        !["definitions", "references"].includes(request.operation) ||
        !Number.isSafeInteger(request.limit) || request.limit < 1 || request.limit > CODE_NAVIGATION_LIMITS.locations ||
        !Number.isSafeInteger(request.max_response_bytes) || request.max_response_bytes < 2048 ||
        request.max_response_bytes > CODE_NAVIGATION_LIMITS.response_bytes) fail("invalid_request");
    let total = 0, projectSources = 0, declarations = 0;
    const sources = request.sources.map((source, index) => {
      if (!exact(source, ["language", "source_utf8", "expected_revision"]) || typeof source.language !== "string" ||
          !Object.hasOwn(scriptKinds, source.language) || typeof source.source_utf8 !== "string" ||
          typeof source.expected_revision !== "string" || !/^sha256:[0-9a-f]{64}$/.test(source.expected_revision)) fail("invalid_source");
      const bytes = encoder.encode(source.source_utf8);
      if (bytes.length > CODE_NAVIGATION_LIMITS.source_bytes || (total += bytes.length) > CODE_NAVIGATION_LIMITS.total_bytes) fail("source_limit");
      if (new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes) !== source.source_utf8) fail("invalid_utf8");
      if (source.source_utf8.includes("\0")) fail("nul_source");
      if (source.language === "ts_declaration") declarations++; else projectSources++;
      const revision = "sha256:" + createHash("sha256").update(bytes).digest("hex");
      result.sources.push({ source: index, revision, source_bytes: bytes.length });
      return { text: source.source_utf8, revision, language: source.language,
        name: source.language === "ts_declaration" ? `/mirket/declaration-${index}.d.ts` : `/mirket/source-${index}.${source.language}`, table: offsets(source.source_utf8),
        snapshot: ts.ScriptSnapshot.fromString(source.source_utf8) };
    });
    if (projectSources < 1 || projectSources > CODE_NAVIGATION_LIMITS.project_sources || declarations > CODE_NAVIGATION_LIMITS.declarations) fail("source_kind_limit");
    const bindings = new Map<string, number>();
    const bindingKey = (source: number, specifier: string): string => JSON.stringify([source, specifier]);
    for (const binding of request.typescript_model.imports) {
      if (!exact(binding, ["source", "specifier", "target"]) || !Number.isSafeInteger(binding.source) ||
          binding.source < 0 || binding.source >= sources.length || !Number.isSafeInteger(binding.target) ||
          binding.target < 0 || binding.target >= sources.length || typeof binding.specifier !== "string" ||
          encoder.encode(binding.specifier).length < 1 || encoder.encode(binding.specifier).length > CODE_NAVIGATION_LIMITS.specifier_bytes ||
          /[\u0000-\u001f\u007f]/u.test(binding.specifier) ||
          new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(encoder.encode(binding.specifier)) !== binding.specifier) fail("invalid_import_binding");
      const key = bindingKey(binding.source, binding.specifier);
      if (bindings.has(key)) fail("ambiguous_import_binding");
      bindings.set(key, binding.target);
    }
    result.model_revision = "sha256:" + createHash("sha256").update(JSON.stringify([
      sources.map((source) => [source.language, source.revision]),
      request.typescript_model.imports.map((binding) => [binding.source, binding.specifier, binding.target]),
    ])).digest("hex");
    if (sources.some((source, index) => source.revision !== request.sources[index]!.expected_revision)) {
      result.status = "changed"; return result;
    }
    const selected = sources[request.source]!;
    const position = selected.table.indexOf(request.position_byte);
    if (position < 0 || position === selected.text.length) fail("invalid_position");
    const byName = new Map(sources.map((source) => [source.name, source]));
    const options: ts.CompilerOptions = { noLib: true, noResolve: true, noEmit: true, types: [], allowJs: true, checkJs: true,
      target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler, jsx: ts.JsxEmit.Preserve };
    const host: ts.LanguageServiceHost = {
      getCompilationSettings: () => options, getScriptFileNames: () => sources.map((source) => source.name),
      getScriptVersion: (name) => byName.get(name)?.revision ?? "absent",
      getScriptSnapshot: (name) => byName.get(name)?.snapshot,
      getScriptKind: (name) => { const source = byName.get(name); return source ? scriptKinds[source.language] : ts.ScriptKind.Unknown; },
      getCurrentDirectory: () => "/mirket", getDefaultLibFileName: () => "/mirket/disabled-default-library.d.ts",
      useCaseSensitiveFileNames: () => true, getNewLine: () => "\n",
      fileExists: (name) => byName.has(name), readFile: (name) => byName.get(name)?.text,
      directoryExists: () => false, readDirectory: () => [], getDirectories: () => [],
      resolveModuleNameLiterals: (names, containingFile, redirectedReference) => {
        if (redirectedReference) fail("unsupported_project_reference");
        const source = byName.get(containingFile);
        if (!source) fail("location_outside_selected_model");
        return names.map((name) => {
          const index = bindings.get(bindingKey(sources.indexOf(source), name.text));
          if (index === undefined) return { resolvedModule: undefined };
          const target = sources[index]!;
          const extension = { js: ts.Extension.Js, ts: ts.Extension.Ts, jsx: ts.Extension.Jsx,
            tsx: ts.Extension.Tsx, ts_declaration: ts.Extension.Dts }[target.language];
          return { resolvedModule: { resolvedFileName: target.name, extension,
            isExternalLibraryImport: target.language === "ts_declaration" } };
        });
      },
      resolveTypeReferenceDirectiveReferences: (names) => names.map(() => ({ resolvedTypeReferenceDirective: undefined })),
    };
    service = ts.createLanguageService(host, ts.createDocumentRegistry(true, "/mirket"));
    const program = service.getProgram();
    if (!program || program.getSourceFiles().length !== sources.length) fail("model_contract");
    let nodes = 0, selectedIdentifier = false;
    const requiredBindings = new Set<string>();
    const importedNames: ts.Identifier[] = [];
    for (const [index, source] of sources.entries()) {
      const file = program.getSourceFile(source.name);
      if (!file || service.getSyntacticDiagnostics(source.name).length) fail("syntax_errors");
      if (file.referencedFiles.length || file.typeReferenceDirectives.length || file.libReferenceDirectives.length) fail("unsupported_reference_directive");
      const pending: { node: ts.Node; depth: number }[] = [{ node: file, depth: 0 }];
      while (pending.length) {
        const { node, depth } = pending.pop()!;
        if (++nodes > CODE_NAVIGATION_LIMITS.nodes) fail("node_limit");
        if (depth > CODE_NAVIGATION_LIMITS.depth) fail("depth_limit");
        if (ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) {
          if (node.attributes) fail("unsupported_import_form");
          if (node.moduleSpecifier) {
            if (!ts.isStringLiteral(node.moduleSpecifier)) fail("unsupported_import_form");
            requiredBindings.add(bindingKey(index, node.moduleSpecifier.text));
          }
          if (ts.isImportDeclaration(node) && node.importClause) {
            if (node.importClause.name) importedNames.push(node.importClause.name);
            const named = node.importClause.namedBindings;
            if (named && ts.isNamespaceImport(named)) importedNames.push(named.name);
            if (named && ts.isNamedImports(named)) for (const element of named.elements) importedNames.push(element.name);
          }
          if (ts.isExportDeclaration(node) && node.moduleSpecifier && node.exportClause && ts.isNamedExports(node.exportClause)) {
            for (const element of node.exportClause.elements) if (ts.isIdentifier(element.name)) importedNames.push(element.name);
          }
        }
        if (ts.isImportEqualsDeclaration(node) || ts.isImportTypeNode(node) ||
            (ts.isModuleDeclaration(node) && ts.isStringLiteral(node.name)) ||
            (ts.isExportAssignment(node) && node.isExportEquals) ||
            (ts.isCallExpression(node) && (node.expression.kind === ts.SyntaxKind.ImportKeyword ||
              (ts.isIdentifier(node.expression) && node.expression.text === "require")))) fail("unsupported_import_form");
        if (source === selected && (node.kind === ts.SyntaxKind.Identifier || node.kind === ts.SyntaxKind.PrivateIdentifier) &&
            node.getStart(file) <= position && position < node.end) selectedIdentifier = true;
        ts.forEachChild(node, (child) => { pending.push({ node: child, depth: depth + 1 }); });
      }
    }
    if ([...requiredBindings].some((key) => !bindings.has(key))) fail("missing_import_binding");
    if ([...bindings.keys()].some((key) => !requiredBindings.has(key))) fail("unused_import_binding");
    const checker = program.getTypeChecker();
    for (const name of importedNames) {
      const alias = checker.getSymbolAtLocation(name);
      if (!alias || !(alias.flags & ts.SymbolFlags.Alias)) fail("unresolved_import");
      const declarations = checker.getAliasedSymbol(alias).getDeclarations();
      if (!declarations?.length) fail("unresolved_import");
      if (declarations.some((declaration) => !byName.has(declaration.getSourceFile().fileName))) fail("location_outside_selected_model");
    }
    if (!selectedIdentifier) fail("identifier_required");
    const definitions = service.getDefinitionAtPosition(selected.name, position);
    if (!definitions?.length) fail("unresolved_symbol");
    // The pinned 5.9.3 service reports failedAliasResolution in its runtime
    // DefinitionInfo result, but omits that field from its public declaration.
    if (definitions.some((definition) => (definition as ts.DefinitionInfo & { failedAliasResolution?: boolean }).failedAliasResolution)) fail("unresolved_import");
    if (definitions.some((definition) => definition.unverified)) fail("unverified_definition");
    const found = request.operation === "definitions" ? definitions : service.getReferencesAtPosition(selected.name, position) ?? [];
    const locations: Location[] = [];
    const seen = new Set<string>();
    for (const item of found) {
      const source = byName.get(item.fileName);
      if (!source) fail("location_outside_selected_model");
      const location = span(source.table, item.textSpan), index = sources.indexOf(source);
      const key = `${index}:${location.start_byte}:${location.end_byte}`;
      if (seen.has(key)) continue;
      seen.add(key);
      const exposed: Location = { source: index, revision: source.revision, location };
      if (request.operation === "references") exposed.is_write = (item as ts.ReferenceEntry).isWriteAccess;
      locations.push(exposed);
    }
    locations.sort((a, b) => a.source - b.source || a.location.start_byte - b.location.start_byte || a.location.end_byte - b.location.end_byte);
    result.status = "locations"; result.locations = locations.slice(0, request.limit); result.complete = locations.length <= request.limit;
    while (size(result) > request.max_response_bytes && result.locations.length) { result.locations.pop(); result.complete = false; }
    if (!result.locations.length && locations.length) {
      result.status = "budget_blocked"; delete result.locations; delete result.complete; result.reason = "location_budget";
    }
    if (size(result) > request.max_response_bytes) fail("response_budget");
    return result;
  } catch (error: unknown) {
    delete result.locations; delete result.complete;
    result.status = "refused"; result.reason = error instanceof Refusal ? error.reason : "language_service_failure";
    return result;
  } finally { service?.dispose(); }
}
