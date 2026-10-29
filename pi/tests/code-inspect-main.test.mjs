import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import test from "node:test";

const entry = fileURLToPath(new URL("../dist/runtime/code-inspect-main.js",import.meta.url));
const release = "0.0.1:code-inspect-test";
const rev = (source) => "sha256:" + createHash("sha256").update(Buffer.from(source)).digest("hex");
function request(source = '// 😀\r\nconst çay: number = 1;', language = "ts") {
  return {protocol_version:1,release_id:release,job_id:"op-code-inspect-1",source_revision:rev(source),source_utf8:source,
    request:{language,pointer:"",expected_revision:rev(source),start_child:0,limit:100,max_response_bytes:30000}};
}
function invoke(input, {raw,args=["--release-id",release],hold=false}={}) {
  return new Promise((accept,reject) => {
    const child = spawn(process.execPath,[entry,...args],{stdio:["pipe","pipe","pipe"]});
    let stdout = "", stderr = "";
    const timer = setTimeout(() => {child.kill("SIGKILL");reject(new Error("code inspection adapter exceeded test deadline"));},8000);
    child.on("error",reject);
    child.stdin.on("error",() => {});
    child.stdout.setEncoding("utf8").on("data",(chunk) => {stdout += chunk; assert.ok(Buffer.byteLength(stdout) <= 65536);});
    child.stderr.setEncoding("utf8").on("data",(chunk) => {stderr += chunk; assert.ok(Buffer.byteLength(stderr) <= 16384);});
    child.on("close",(code,signal) => {
      clearTimeout(timer);
      try {accept({code,signal,stderr,messages:stdout.trim().split("\n").filter(Boolean).map(JSON.parse)});} catch(error) {reject(error);}
    });
    if (hold) child.stdin.write(raw); else child.stdin.end(raw ?? JSON.stringify(input)+"\n");
  });
}

test("finite code transport binds exact release/job/source and returns one byte-based TypeScript page",async() => {
  const input = request();
  const result = await invoke(input);
  assert.equal(result.code,0);
  assert.equal(result.stderr,"");
  assert.equal(result.messages.length,1);
  const frame = result.messages[0];
  assert.equal(frame.protocol_version,1);
  assert.equal(frame.release_id,release);
  assert.equal(frame.job_id,input.job_id);
  assert.equal(frame.result.status,"page");
  assert.equal(frame.result.revision,input.source_revision);
  assert.deepEqual(frame.result.children[0].location,{start_byte:0,end_byte:Buffer.byteLength(input.source_utf8)});
  assert.ok(Buffer.byteLength(JSON.stringify(frame.result)) <= input.request.max_response_bytes);
});
test("byte-only transport handles NUL refusal and full 1MiB source",async() => {
  for(const input of [request("const x = \"\0\";"),request(" ".repeat(1048576))]) {
    const result = await invoke(input);
    assert.equal(result.code,0);
    assert.equal(result.messages[0].result.revision,input.source_revision);
    assert.equal(result.messages[0].result.status,input.source_utf8.includes("\0")?"refused":"page");
  }
});
test("same finite transport dispatches all four explicit code script kinds and rejects unknown language forms",async() => {
  for (const [language,source] of [["js","function çay() { return 1; }"],["ts","const çay: number = 1;"],
    ["jsx","const çay = <Box/>;"],["tsx","const çay = <Box/>;"]]) {
    const input = request("\ufeff// 😀\r\n"+source+"\r\n",language), result = await invoke(input);
    assert.equal(result.code,0); assert.equal(result.messages[0].result.status,"page");
    assert.deepEqual(result.messages[0].result.backend,{name:"typescript",version:"5.9.3"});
    assert.equal(result.messages[0].result.revision,input.source_revision);
    assert.equal(result.messages[0].result.structure_complete,true);
  }
  for (const language of ["html","css","python","bash","json","javascript",["ts"]]) {
    const result = await invoke(request("let x;",language));
    assert.equal(result.code,1); assert.equal(result.messages[0].error,"invalid_request");
  }
});
test("spoofed release/revision and path/plugin/execution fields refuse before inspection",async() => {
  for(const mutate of [r=>({...r,release_id:"stale"}),r=>({...r,source_revision:rev("different")}),
    r=>({...r,source_path:"/never-read"}),r=>({...r,request:{...r.request,plugins:["never-execute"]}}),
    r=>({...r,request:{...r.request,expected_revision:rev("different")}}),r=>({...r,source_utf8:"\ud800"})]) {
    const result = await invoke(mutate(request()));
    assert.equal(result.code,1);
    assert.equal(result.messages.length,1);
    assert.ok(result.messages[0].error);
    assert.equal(result.messages[0].result,undefined);
  }
});
test("incomplete, oversized, trailing and invalid UTF-8 request frames refuse finitely",async() => {
  for(const raw of ['{"incomplete":',"x".repeat(2097153),JSON.stringify(request())+"\n{}\n",Buffer.from([0xc3,0x28,10]),JSON.stringify(request())]) {
    const result = await invoke(null,{raw});
    assert.equal(result.code,1);
    assert.ok(result.messages[0].error);
    assert.equal(result.messages[0].result,undefined);
  }
});
test("partial input has a finite read deadline and no result",async() => {
  const result = await invoke(null,{raw:"{",hold:true});
  assert.equal(result.code,1);
  assert.equal(result.messages[0].error,"request_timeout");
});
test("startup requires explicit release identity",async() => {
  const result = await invoke(null,{args:[]});
  assert.equal(result.code,1);
  assert.equal(result.messages[0].error,"invalid_startup");
});
