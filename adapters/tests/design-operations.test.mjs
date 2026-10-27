import assert from "node:assert/strict";
import test from "node:test";
import { compileTokens, DesignOperationError, optimizeSvg } from "../dist/design-operations.js";

const encoder = new TextEncoder();
const bytes = (value) => encoder.encode(typeof value === "string" ? value : JSON.stringify(value));

function errorCode(error) {
  assert.equal(error instanceof DesignOperationError, true);
  return error.code;
}

test("compiles bounded aliases to deterministic CSS and nested JSON", async () => {
  const input = bytes({
    color: {
      brand: { value: "#1463ff" },
      accent: { value: "{color.brand}" },
    },
    spacing: { sm: { value: 8, type: "dimension" } },
  });
  const css = await compileTokens(input, "css");
  const json = await compileTokens(input, "json");
  assert.equal(css.format, "css");
  assert.equal(css.token_count, 3);
  assert.match(new TextDecoder().decode(css.output), /--color-brand: #1463ff;/u);
  assert.match(new TextDecoder().decode(css.output), /--color-accent: #1463ff;/u);
  assert.match(new TextDecoder().decode(css.output), /--spacing-sm: 8rem;/u);
  assert.match(new TextDecoder().decode(json.output), /"accent": "#1463ff"/u);
  assert.equal(css.output_bytes, css.output.byteLength);
  assert.equal(json.output_bytes, json.output.byteLength);
});

test("rejects malformed, missing and cyclic aliases before formatter work", async () => {
  await assert.rejects(() => compileTokens(bytes({ color: { a: { value: "{missing}" } } }), "json"), (error) => errorCode(error) === "alias_unresolved");
  await assert.rejects(() => compileTokens(bytes({ color: { a: { value: "{color.b}" }, b: { value: "{color.a}" } } }), "css"), (error) => errorCode(error) === "alias_cycle");
  await assert.rejects(() => compileTokens(bytes({ color: { a: { value: "prefix {color.b}" } } }), "json"), (error) => errorCode(error) === "alias_invalid");
  await assert.rejects(() => compileTokens(bytes({ color: { a: { value: "{../outside}" } } }), "json"), (error) => errorCode(error) === "alias_invalid");
});

test("rejects duplicate JSON keys at nested groups and token leaves", async () => {
  const duplicateGroup = '{"colors":{"brand":{"value":"#1463ff"},"brand":{"value":"#000000"}}}';
  const duplicateValue = '{"colors":{"brand":{"value":"#1463ff","value":"#000000"}}}';
  await assert.rejects(() => compileTokens(bytes(duplicateGroup), "json"), (error) => errorCode(error) === "invalid_tokens");
  await assert.rejects(() => compileTokens(bytes(duplicateValue), "css"), (error) => errorCode(error) === "invalid_tokens");
});

test("optimizes a valid SVG with stable bytes", () => {
  const input = bytes('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="#1463ff"/></svg>');
  const first = optimizeSvg(input);
  const second = optimizeSvg(input);
  assert.equal(first.format, "svg");
  assert.equal(first.element_count, 2);
  assert.match(new TextDecoder().decode(first.output), /<svg/u);
  assert.deepEqual([...first.output], [...second.output]);
});

test("accepts a bounded D2-like SVG with styles, text, markers and internal URLs", () => {
  const input = bytes(`<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 80">
  <style type="text/css"><![CDATA[.edge { marker-end: url(#arrow); } ]]></style>
  <defs>
    <marker id="arrow" markerWidth="6" markerHeight="6" refX="5" refY="3" orient="auto">
      <path d="M0,0 L6,3 L0,6 z" fill="#333"/>
    </marker>
  </defs>
  <path class="edge" d="M10,40 L100,40" stroke="#333" marker-end="url(#arrow)"/>
  <text x="12" y="28" font-family="Noto Sans" font-size="12">Node</text>
</svg>`);
  const result = optimizeSvg(input);
  assert.match(new TextDecoder().decode(result.output), /<svg/u);
  assert.ok(result.element_count >= 5);
});

test("refuses active, external and structurally unsafe SVG features", () => {
  const unsafe = [
    '<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>',
    '<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg"/>',
    '<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.test/a.png"/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><rect onclick="alert(1)"/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><rect style="fill:url(https://example.test/a)"/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><style>@import "theme.css";</style></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><style>@font-face { font-family: localfont; src: local(myfont) }</style></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><unknown/></svg>',
  ];
  for (const source of unsafe) {
    assert.throws(() => optimizeSvg(bytes(source)), (error) => errorCode(error) === "svg_rejected");
  }
  assert.throws(() => optimizeSvg(bytes("x".repeat(1024 * 1024 + 1))), (error) => errorCode(error) === "input_too_large");
});
