// Reusable disposable fixture and retained host-browser evidence checks.
// Retained host-browser captures are historical; this file neither launches a browser nor qualifies the optional Pi peer.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, realpath, stat } from "node:fs/promises";
import path from "node:path";
import { test } from "node:test";

const retainedEvidence = process.env.MIRKET_INTERACTIVE_QA_EVIDENCE_DIR;
const sha = value => `sha256:${createHash("sha256").update(value).digest("hex")}`;

export const fixtureHtml = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Interactive QA fixture</title><style>
@font-face{font-family:QALocal;src:url("./fixture-font.ttf") format("truetype");font-display:block}
*{box-sizing:border-box}html,body{margin:0;background:#f7f7f2;color:#18201b;font-family:QALocal,system-ui,sans-serif}
.shell{width:min(100% - 32px,640px);margin:24px auto;padding:24px;background:#fff;border:1px solid #d7ddd5;border-radius:18px}
.hero{display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap}.eyebrow{font-size:12px;letter-spacing:.12em;text-transform:uppercase;color:#47705d}
h1{font-size:clamp(24px,6vw,40px);line-height:1.05;margin:8px 0}.copy{max-width:48ch;line-height:1.55}
button{font:inherit;border:0;border-radius:10px;padding:10px 14px;background:#173c2b;color:#fff;cursor:pointer}
button:focus-visible{outline:3px solid #2563eb;outline-offset:3px}.panel{margin-top:20px;padding:16px;background:#eef4ed;border-radius:12px;animation:breath 1000ms infinite alternate}
@keyframes breath{from{transform:translateY(0)}to{transform:translateY(4px)}}
.status{min-height:24px;margin-top:16px}.error{color:#a52626;font-weight:600}.wide{white-space:normal;overflow-wrap:anywhere}#qa-result{display:none}
html[data-defect="focus"] button:focus-visible{outline:none;outline-offset:0}
html[data-defect="overflow"] .shell{width:2000px}
html[data-defect="reduced"] .panel{animation-duration:1000ms;animation-iteration-count:infinite}
@media(prefers-reduced-motion:reduce){.panel{animation-duration:1ms;animation-iteration-count:1;animation-play-state:paused}}
@media(prefers-reduced-motion:reduce){html[data-defect="reduced"] .panel{animation-duration:1000ms;animation-iteration-count:infinite;animation-play-state:running}}
@media(max-width:500px){.shell{margin:12px 8px;padding:16px}}
</style></head><body><main class="shell" id="shell"><div class="hero"><div><div class="eyebrow">Host-browser QA</div><h1>Readable interface</h1></div><button id="open" aria-expanded="false" aria-controls="panel">Open details</button></div>
<p class="copy">A scoped fixture for keyboard focus, responsive reflow, local fonts and reduced-motion behavior.</p><section id="panel" class="panel" hidden><strong>Details are open.</strong><p class="wide">This line remains inside the intended reading region.</p></section>
<div id="status" class="status" role="status" aria-live="polite"></div><pre id="qa-result"></pre></main><script>
const params=new URLSearchParams(location.search),defect=params.get("defect")||"clean",reduce=matchMedia("(prefers-reduced-motion: reduce)").matches;
document.documentElement.dataset.defect=defect;
const open=document.querySelector("#open"),panel=document.querySelector("#panel"),status=document.querySelector("#status");
open.addEventListener("click",()=>{const next=panel.hidden;panel.hidden=!next;open.setAttribute("aria-expanded",String(next));status.textContent=next?"Details opened":"Details closed"});
if(defect==="font")document.body.style.fontFamily="system-ui,sans-serif";
async function inspect(){
  if(defect==="font")await new Promise(resolve=>setTimeout(resolve,100));
  else await Promise.race([document.fonts.ready,new Promise(resolve=>setTimeout(resolve,250))]);
  open.focus();open.click();
  const style=getComputedStyle(open),faces=[...document.fonts].map(face=>({family:face.family,status:face.status}));
  const animation=getComputedStyle(document.querySelector("#panel")),duration=animation.animationDuration.endsWith("ms")?parseFloat(animation.animationDuration):parseFloat(animation.animationDuration)*1000;
  const result={defect,viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio},focus:{active:document.activeElement===open,outline_width:parseFloat(style.outlineWidth)||0,outline_style:style.outlineStyle,visible:(parseFloat(style.outlineWidth)||0)>=2&&style.outlineStyle!=="none"},overflow:{client_width:document.documentElement.clientWidth,scroll_width:document.documentElement.scrollWidth,horizontal:document.documentElement.scrollWidth>document.documentElement.clientWidth},font:{faces,loaded:faces.some(face=>face.family.includes("QALocal")&&face.status==="loaded")},reduced:{requested:reduce,duration_ms:duration,iterations:animation.animationIterationCount},interaction:{expanded:open.getAttribute("aria-expanded")==="true",panel_hidden:panel.hidden,status:status.textContent}};
  document.querySelector("#qa-result").textContent=encodeURIComponent(JSON.stringify(result));
}inspect();
</script></body></html>`;

function pngDimensions(bytes) {
  assert.deepEqual([...bytes.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10], "evidence must be a PNG");
  return {width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20)};
}

async function scopedCapture(root, name) {
  assert.equal(typeof name, "string");
  assert.ok(name.length > 0 && !path.isAbsolute(name) && !name.split(/[\\/]/).includes(".."), "capture must be relative and scoped");
  const base = await realpath(root);
  const target = await realpath(path.join(base, name));
  assert.ok(target.startsWith(base + path.sep), "capture must remain inside evidence root");
  return target;
}

test("retained capture paths reject external locations", async () => {
  for (const name of ["/tmp/outside.png", "../outside.png", "nested/../../outside.png", "..\\outside.png", ""]) {
    await assert.rejects(scopedCapture("/nonexistent-evidence-root", name), /capture must be relative and scoped/);
  }
});

test("retained host-browser QA evidence has independent captures for every case", { skip: !retainedEvidence }, async () => {
  const evidence = JSON.parse(await readFile(path.join(retainedEvidence, "qa-results-host.json"), "utf8"));
  assert.equal(evidence.source_revision, sha(Buffer.from(fixtureHtml)));
  assert.equal(await readFile(path.join(retainedEvidence, "index.html"), "utf8"), fixtureHtml);
  assert.ok(Array.isArray(evidence.cases));
  assert.equal(new Set(evidence.cases.map(item => item.id)).size, evidence.cases.length, "case IDs must be unique");
  assert.ok(evidence.browser_viewport && evidence.browser_viewport.width > 0 && evidence.browser_viewport.height > 0);
  // A recorded manual observation, not an authenticated input-event trace.
  assert.deepEqual(evidence.keyboard_manual, {before_expanded: true, key: "Enter", after_expanded: false, focus_retained: true});
  const expected = {
    clean: value => { assert.equal(value.focus.visible, true); assert.equal(value.overflow.horizontal, false); assert.equal(value.font.loaded, true); assert.equal(value.reduced.requested, false); },
    "missing-focus": value => assert.equal(value.focus.visible, false),
    "overflow-stress": value => { assert.equal(value.overflow.horizontal, true); assert.equal(value.browser_viewport?.width ?? value.viewport?.width, 390); },
    "font-failure": value => assert.equal(value.font.loaded, false),
    "reduced-motion": value => { assert.equal(value.reduced.requested, true); assert.equal(value.reduced.iterations, "infinite"); assert.ok(value.reduced.duration_ms > 1); },
    "reduced-motion-clean-control": value => { assert.equal(value.reduced.requested, true); assert.equal(value.reduced.iterations, "1"); assert.ok(value.reduced.duration_ms <= 1.5); },
    "clean-mobile": value => { assert.equal(value.overflow.horizontal, false); assert.equal(value.viewport.width, 390); },
  };
  for (const item of ["clean", "missing-focus", "overflow-stress", "font-failure", "reduced-motion", "reduced-motion-clean-control", "clean-mobile"]) {
    const value = evidence.cases.find(candidate => candidate.id === item);
    assert.ok(value, `missing ${item}`);
    const screenshotPath = await scopedCapture(retainedEvidence, value.screenshot);
    assert.ok((await stat(screenshotPath)).size > 0, `missing screenshot ${item}`);
    assert.match(value.screenshot_sha256, /^sha256:[a-f0-9]{64}$/);
    const screenshotBytes = await readFile(screenshotPath);
    assert.equal(value.screenshot_sha256, sha(screenshotBytes));
    const dimensions = pngDimensions(screenshotBytes);
    assert.equal(dimensions.width, value.viewport.width * value.viewport.dpr, `${item} screenshot width`);
    assert.equal(dimensions.height, value.viewport.height * value.viewport.dpr, `${item} screenshot height`);
    expected[item](value);
  }
});
