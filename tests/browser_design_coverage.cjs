/* Optional independent effect checks for the eight scoped coverage lessons.
 * Uses an managed Browserless package and browser; never installs or
 * downloads anything. Serve assets/design-languages on a loopback URL first.
 * NODE_PATH=/existing/node_modules BROWSERLESS_WS=ws://127.0.0.1:3000 \
 *   node tests/browser_design_coverage.cjs BASE_URL OUTPUT_DIR [ID,...]
 * Captures require human/agent inspection; these checks do not certify native
 * runtime behavior, assistive technology, authorization or human acceptance. */
'use strict';
const { connect } = require('./browserless.cjs');
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');
const base = process.argv[2] || process.env.BASE_URL;
const output = path.resolve(process.argv[3] || process.env.OUTPUT_DIR || '.mirket/evidence/design-coverage');
const ids = (process.argv[4] || process.env.IDS || 'saleor-commerce,atlassian-collaboration,fiori-complex-workspace,penpot-direct-manipulation,vega-linked-analysis,android-adaptive-continuity,spectrum-bidirectional,aria-alternative-input').split(',').map(s=>s.trim()).filter(Boolean);
const focal = {'saleor-commerce':'.summary','atlassian-collaboration':'.composer','fiori-complex-workspace':'#item-form','penpot-direct-manipulation':'#properties','vega-linked-analysis':'#linked-bars','android-adaptive-continuity':'#detail-panel','spectrum-bidirectional':'.request','aria-alternative-input':'#queue>li[data-id="habitat"]'};
assert.ok(ids.length&&ids.every(id=>focal[id]),'Select known coverage lesson IDs.');
const baseURL = new URL(base || 'about:blank');
if (baseURL.protocol !== 'http:' || !['127.0.0.1','localhost'].includes(baseURL.hostname) || !baseURL.port || baseURL.username || baseURL.password || baseURL.search || baseURL.hash || !baseURL.pathname.endsWith('/')) throw new Error('BASE_URL must be a loopback URL with an explicit port and trailing /.');
const report = {scope:'Independent browser effects and render captures. No native, screen-reader, production-service or human-acceptance claim.',base,browser:null,results:[],failures:[]};
const text = (p,s) => p.locator(s).innerText();
const value = (p,s) => p.locator(s).inputValue();
const attr = (p,s,a) => p.locator(s).getAttribute(a);
async function key(p,s,key='Enter') { await p.locator(s).focus(); await p.keyboard.press(key); }
async function eqText(p,s,expected) { assert.equal((await text(p,s)).trim(),expected,s); }
async function focusIs(p,id) { await p.waitForFunction(expected=>document.activeElement?.id===expected,id);assert.equal(await p.evaluate(()=>document.activeElement?.id),id,'focus ownership'); }
async function capture(p,name,selector) { const file=`${name}.png`; if(selector) await p.locator(selector).screenshot({path:path.join(output,file)});else await p.screenshot({path:path.join(output,file),fullPage:true});return file; }
async function geometry(p) {const g=await p.evaluate(()=>({viewport:innerWidth,width:document.documentElement.scrollWidth})); assert.ok(g.width<=g.viewport+1,`page overflow ${JSON.stringify(g)}`);return g;}
async function enlargeText(p) {await p.evaluate(()=>{const nodes=[...document.querySelectorAll('body *:not(svg):not(svg *)')];const sizes=nodes.map(n=>[n,parseFloat(getComputedStyle(n).fontSize)]);for(const[n,s]of sizes)n.style.fontSize=`${s*2}px`;});}

const journeys = {
  async 'saleor-commerce'(p,row,bad) {
    await p.locator('#capacity').selectOption('750'); await p.locator('#quantity').fill('3'); await key(p,'#update-bag');
    for(const[s,v]of Object.entries({'#line-variant':'750 ml · 3 flasks','#subtotal':'$120.00','#shipping':'$6.00','#tax':'$10.08','#total':'$136.08'}))await eqText(p,s,v);
    if(bad){assert.notEqual(await text(p,'#wrong-headline'),await text(p,'#total'));row.discriminant='Headline $120.00 omits $16.08 of delivery/tax.';return;}
    await p.locator('#quantity').fill('0'); await key(p,'#update-bag'); await eqText(p,'#total','$136.08');assert.equal(await attr(p,'#quantity','aria-invalid'),'true');
    await p.locator('#quantity').fill('3');await key(p,'#update-bag');await key(p,'#start-review');await focusIs(p,'review-title');
    const address='Independent QA · 17 Long Garden Road';await p.locator('#address').fill(address);await key(p,'#check-review');assert.match(await text(p,'#review-status'),/rejected/);assert.equal(await value(p,'#address'),address);assert.equal(await p.locator('#ready').isVisible(),false);
    await key(p,'#use-available');await eqText(p,'#total','$92.88');assert.equal(await value(p,'#address'),address);assert.equal(await p.locator('#ready').isVisible(),false);
    await key(p,'#check-review');assert.equal(await p.locator('#ready').isVisible(),true);await p.locator('#email').fill('qa@example.invalid');assert.equal(await p.locator('#ready').isVisible(),false,'editing details invalidates approval');
    await key(p,'#check-review');await key(p,'#finish');assert.match(await text(p,'#completion-summary'),/750 ml · 2 flasks · \$92\.88 USD reviewed for qa@example.invalid/);await focusIs(p,'completion');row.effect=await text(p,'#completion-summary');row.focal=await capture(p,`${row.tag}-effect`,'#review');
    await key(p,'#edit-selection');await focusIs(p,'capacity');await p.locator('#quantity').fill('1');await key(p,'#update-bag');assert.equal(await p.locator('#completion').isVisible(),false,'applying a changed bag invalidates completed review');assert.equal(await p.locator('#ready').isVisible(),false);await eqText(p,'#total','$49.68');
  },
  async 'atlassian-collaboration'(p,row,bad) {
    if(bad){assert.equal(await p.locator('#access-list li').count(),3);assert.match(await text(p,'.presence-wrong'),/Everyone here can read/);assert.match(await text(p,'#access-summary'),/Sam has none/);row.discriminant='Presence claim contradicts the unchanged three-person access list.';return;}
    const draft='<b>Literal @Sam QA draft</b>';await p.locator('#comment-draft').fill(draft);await p.locator('#comment-scope').selectOption('editors');await p.locator('#local-role').selectOption('viewer');await key(p,'#post-comment');await eqText(p,'#comment-count','1 comment');assert.equal(await value(p,'#comment-draft'),draft);assert.equal(await value(p,'#comment-scope'),'editors');assert.match(await text(p,'#comment-status'),/rejected/);
    await key(p,'#close-draft');await focusIs(p,'resume-draft');await key(p,'#resume-draft');await focusIs(p,'comment-draft');assert.equal(await value(p,'#comment-draft'),draft);
    await p.locator('#share-person').selectOption('sam');await p.locator('#share-permission').selectOption('editor');await key(p,'#apply-access');assert.equal(await p.locator('#access-list li').count(),3);assert.match(await text(p,'#access-status'),/rejected/);
    await p.locator('#local-role').selectOption('owner');await key(p,'#apply-access');assert.equal(await p.locator('#access-list li').count(),4);assert.match(await text(p,'#composer-scope'),/Morgan, Jules, Sam/);await key(p,'#post-comment');await eqText(p,'#comment-count','2 comments');assert.equal(await p.locator('#comments b').count(),0,'markup must remain literal');assert.equal(await p.locator('#comments .comment').last().locator('p').innerText(),draft);row.effect=await p.locator('#comments .comment').last().innerText();row.focal=await capture(p,`${row.tag}-effect`,'.discussion');
    // Verify focused fixed skip link is not left covering the composer target.
    await p.locator('.skip').focus();await p.keyboard.press('Enter');await p.locator('#comment-draft').focus();await p.keyboard.press('Tab');assert.equal(await p.locator('.skip').evaluate(n=>document.activeElement===n),false);row.skip_after_content_focus=await p.locator('.skip').evaluate(n=>({top:n.getBoundingClientRect().top,bottom:n.getBoundingClientRect().bottom}));
  },
  async 'fiori-complex-workspace'(p,row,bad) {
    await p.locator('#item-quantity').fill('9');await key(p,'#apply-item');await eqText(p,'#order-total','$204.00');
    if(bad){await eqText(p,'#item-status','Saved');await eqText(p,'#order-status','Order draft · not saved');row.discriminant='Child claims Saved while parent remains an unsaved draft.';return;}
    await eqText(p,'#item-status','Staged · not saved');await key(p,'#save-order');assert.match(await text(p,'#save-error-text'),/PO-4201 \/ Item 20/);await key(p,'#open-error-item');assert.equal(await value(p,'#item-quantity'),'9');
    await p.locator('#item-quantity').fill('8');await key(p,'#apply-item');await key(p,'#save-order');await eqText(p,'#order-total','$192.00');await eqText(p,'#order-status','Locally saved baseline');
    const note='Independent parent-owned preparation note';await p.locator('#item-note').fill(note);await key(p,'#open-PO-4202');assert.equal(await p.locator('#switch-dialog').isVisible(),true);await focusIs(p,'keep-editing');await p.keyboard.press('Shift+Tab');await focusIs(p,'discard-switch');await p.keyboard.press('Tab');await focusIs(p,'keep-editing');await p.keyboard.press('Escape');await focusIs(p,'open-PO-4202');await eqText(p,'#order-id','PO-4201');
    await key(p,'#open-PO-4202');await key(p,'#keep-switch');await eqText(p,'#order-id','PO-4202');await p.locator('#item-note').fill('Second order owns this note');await key(p,'#apply-item');await key(p,'#save-order');
    await key(p,'#open-PO-4201');assert.equal(await value(p,'#item-note'),note);assert.equal(await value(p,'#item-quantity'),'8');await eqText(p,'#item-parent','PO-4201 / Item 20');
    row.effect={parent:await text(p,'#item-parent'),note:await value(p,'#item-note'),total:await text(p,'#order-total')};row.focal=await capture(p,`${row.tag}-effect`,'main');
    await key(p,'#open-PO-4202');await key(p,'#discard-switch');assert.equal(await value(p,'#item-note'),'Second order owns this note','discard must not alter other order');await key(p,'#open-PO-4201');assert.notEqual(await value(p,'#item-note'),note,'discard removes only named order buffer');await eqText(p,'#order-total','$192.00');
    await p.locator('#item-quantity').fill('0');await key(p,'#apply-item');assert.equal(await attr(p,'#item-quantity','aria-invalid'),'true');await key(p,'[data-item="10"]');assert.equal(await value(p,'#item-quantity'),'2');assert.notEqual(await attr(p,'#item-quantity','aria-invalid'),'true','another valid item cannot inherit the previous item’s validation error');await key(p,'[data-item="20"]');assert.equal(await value(p,'#item-quantity'),'0');assert.equal(await attr(p,'#item-quantity','aria-invalid'),'true','returning restores the invalid buffer’s own error');assert.match(await text(p,'#item-message'),/whole quantity/);await key(p,'#reset-item');assert.notEqual(await attr(p,'#item-quantity','aria-invalid'),'true');
  },
  async 'penpot-direct-manipulation'(p,row,bad) {
    await key(p,'#shape-title','ArrowRight');assert.equal(await attr(p,'#shape-title','data-x'),'61');await key(p,'#layer-stamp');await key(p,'#undo');
    if(bad){assert.equal(await attr(p,'#shape-title','data-x'),'61');assert.equal(await attr(p,'#shape-stamp','data-x'),'60');row.discriminant='Undo overwrites current stamp with former title state.';return;}
    assert.equal(await attr(p,'#shape-title','data-x'),'60');assert.equal(await attr(p,'#shape-stamp','data-x'),'360');assert.equal(await attr(p,'#studio-main','data-selected-id'),'stamp');await key(p,'#redo');assert.equal(await attr(p,'#shape-title','data-x'),'61');await key(p,'#undo');await key(p,'#shape-stamp','ArrowLeft');assert.equal(await p.locator('#redo').isDisabled(),true,'new edit clears redo');
    await p.locator('#object-label').fill('Independent stamp note');await p.locator('#object-x').fill('300');await key(p,'#properties button[type=submit]');await key(p,'#layer-title');await key(p,'#undo');assert.equal(await attr(p,'#shape-stamp','data-x'),'359');await eqText(p,'#shape-stamp','SAT · 17');
    const b=await p.locator('#shape-title').boundingBox();const before={x:await attr(p,'#shape-title','data-x'),depth:await attr(p,'#undo','data-depth')};await p.mouse.move(b.x+b.width/2,b.y+b.height/2);await p.mouse.down();await p.mouse.move(b.x+b.width/2+35,b.y+b.height/2+25);await p.keyboard.press('Escape');await p.mouse.up();assert.equal(await attr(p,'#shape-title','data-x'),before.x);assert.equal(await attr(p,'#undo','data-depth'),before.depth,'cancelled pointer gesture must not add history');row.effect={titleX:await attr(p,'#shape-title','data-x'),stampX:await attr(p,'#shape-stamp','data-x'),selected:await attr(p,'#studio-main','data-selected-id')};row.focal=await capture(p,`${row.tag}-effect`,'#studio-main');
  },
  async 'vega-linked-analysis'(p,row,bad) {
    await p.locator('#range-from').fill('0');await p.locator('#range-to').fill('20');await key(p,'#filters button[type=submit]');
    const selected=await p.locator('#station-table tbody tr:visible').evaluateAll(ns=>ns.map(n=>n.dataset.id));assert.deepEqual(selected,['a','b','c']);await eqText(p,'#station-total','3 of 12 stations');await eqText(p,'#observation-total','12 observations');assert.equal(await attr(p,'#selection-summary','data-denominator'),'12');
    if(bad){assert.equal(await attr(p,'#river-bar','data-value'),'40');assert.equal(await attr(p,'#park-bar','data-value'),'43');row.discriminant='3 selected rows/12 observations disagree with bars showing 40+43.';return;}
    assert.equal(await attr(p,'#river-bar','data-value'),'7');assert.equal(await attr(p,'#park-bar','data-value'),'5');const ratios=await p.locator('#linked-bars').evaluate(n=>{const width=n.getBoundingClientRect().width;return ['river','park'].map(id=>document.getElementById(id+'-bar').getBoundingClientRect().width/width)});assert.ok(Math.abs(ratios[0]-.14)<.01&&Math.abs(ratios[1]-.10)<.01,'bar lengths use one fixed 0–50 scale');
    row.focal=await capture(p,`${row.tag}-effect`,'.charts');await p.locator('#habitat').selectOption('River');await eqText(p,'#station-total','2 of 12 stations');assert.equal(await attr(p,'#river-bar','data-value'),'7');assert.equal(await attr(p,'#park-bar','data-value'),'0');assert.equal(await attr(p,'#selection-summary','data-denominator'),'12');
    await p.locator('#range-to').fill('5');await key(p,'#filters button[type=submit]');await eqText(p,'#station-total','0 of 12 stations');await eqText(p,'#observation-total','0 observations');assert.equal(await p.locator('#linked-empty').isVisible(),true);assert.equal(await attr(p,'#river-bar','data-value'),'0');assert.equal(await attr(p,'#park-bar','data-value'),'0');await key(p,'#reset');await eqText(p,'#station-total','12 of 12 stations');await eqText(p,'#observation-total','83 observations');
    await p.locator('#range-from').fill('30');await p.locator('#range-to').fill('20');await key(p,'#filters button[type=submit]');await eqText(p,'#station-total','12 of 12 stations');assert.equal(await p.locator('#range-to').evaluate(n=>n.validity.valid),false);await key(p,'#reset');
    const box=await p.locator('#scatterplot').boundingBox();const sx=box.width/540;await p.mouse.move(box.x+60*sx,box.y+100*sx);await p.mouse.down();await p.mouse.move(box.x+210*sx,box.y+100*sx);await p.mouse.up();await eqText(p,'#station-total','3 of 12 stations');const old=await attr(p,'#scatterplot','data-to');await p.mouse.down();await p.mouse.move(box.x+400*sx,box.y+100*sx);await p.keyboard.press('Escape');await p.mouse.up();assert.equal(await attr(p,'#scatterplot','data-to'),old);row.effect={selected,river:7,park:5,denominator:12,resetObservations:83};
  },
  async 'android-adaptive-continuity'(p,row,bad) {
    await key(p,'#select-orchard');await p.locator('#note-orchard').fill('Independent orchard annotation');await p.setViewportSize({width:390,height:844});
    await p.waitForFunction(()=>document.querySelector('#fieldlog').dataset.layout==='narrow');
    if(bad){assert.equal(await attr(p,'#fieldlog','data-selected-id'),'quay');assert.equal(await value(p,'#note-orchard'),'Independent orchard annotation');row.discriminant='Width change replaces orchard selection with quay while orchard note remains.';return;}
    assert.equal(await attr(p,'#fieldlog','data-selected-id'),'orchard');assert.equal(await attr(p,'#detail-panel','data-record-id'),'orchard');assert.equal(await value(p,'#note-orchard'),'Independent orchard annotation');await key(p,'#back-to-list');await focusIs(p,'select-orchard');await key(p,'#select-orchard');await focusIs(p,'title-orchard');await p.reload();assert.equal(await attr(p,'#fieldlog','data-selected-id'),'orchard');assert.equal(await value(p,'#note-orchard'),'Independent orchard annotation');await p.goBack();assert.equal(await attr(p,'#fieldlog','data-pane'),'list');await focusIs(p,'select-orchard');await p.goForward();assert.equal(await attr(p,'#detail-panel','data-record-id'),'orchard');await focusIs(p,'title-orchard');row.focal=await capture(p,`${row.tag}-effect`,'#detail-panel');await p.setViewportSize({width:1280,height:900});assert.equal(await attr(p,'#fieldlog','data-selected-id'),'orchard');assert.equal(await value(p,'#note-orchard'),'Independent orchard annotation');await key(p,'#select-library');await p.locator('#note-library').fill('Separate library annotation');await key(p,'#select-orchard');assert.equal(await value(p,'#note-orchard'),'Independent orchard annotation');row.effect={selected:'orchard',orchard:await value(p,'#note-orchard'),library:await value(p,'#note-library')};
  },
  async 'spectrum-bidirectional'(p,row,bad) {
    const transforms=await p.evaluate(()=>Object.fromEntries(['.flow-icon','#clock-icon','#verified-icon','#case-reference'].map(s=>[s,getComputedStyle(document.querySelector(s)).transform])));
    assert.equal(await attr(p,'#support-workspace','dir'),'rtl');await eqText(p,'#case-reference','SUP-2047');
    if(bad){assert.match(transforms['#clock-icon'],/^matrix\(-1,/);assert.match(transforms['#case-reference'],/^matrix\(-1,/);row.discriminant='Universal mirroring reverses clock and known Latin case identifier.';row.transforms=transforms;return;}
    assert.match(transforms['.flow-icon'],/^matrix\(-1,/);for(const s of ['#clock-icon','#verified-icon','#case-reference'])assert.equal(transforms[s],'none',`${s} must retain orientation`);
    const customer=await text(p,'#customer-name'),message=await text(p,'#customer-message'),draft='QA <b>literal</b> SUP-2047 / طلب 17';await p.locator('#note').fill(draft);await key(p,'#save-note');await eqText(p,'#saved-text',draft);assert.equal(await p.locator('#saved-text b').count(),0);
    await p.locator('#locale').selectOption('en');assert.equal(await attr(p,'#support-workspace','dir'),'ltr');assert.equal(await attr(p,'#support-workspace','lang'),'en');await eqText(p,'#case-reference','SUP-2047');assert.equal(await value(p,'#note'),draft);await eqText(p,'#customer-name',customer);await eqText(p,'#customer-message',message);assert.equal(await p.locator('.flow-icon').evaluate(n=>getComputedStyle(n).transform),'none');row.english=await capture(p,`${row.tag}-english`,'main');
    await p.locator('#note').fill('');await key(p,'#save-note');await focusIs(p,'note');assert.equal(await attr(p,'#note','aria-invalid'),'true');await eqText(p,'#saved-text',draft);await p.locator('#locale').selectOption('ar');assert.equal(await attr(p,'#support-workspace','dir'),'rtl');assert.equal(await p.locator('#note-error').isVisible(),true);await p.locator('#note-language').selectOption('ar');const corrected='يرجى تأكيد SUP-2047 يوم الخميس.';await p.locator('#note').fill(corrected);await key(p,'#save-note');await eqText(p,'#saved-text',corrected);assert.equal(await attr(p,'#saved-text','lang'),'ar');assert.notEqual(await attr(p,'#note','aria-invalid'),'true');
    const isolation=await p.locator('#customer-name,#case-reference,#saved-text').evaluateAll(ns=>ns.map(n=>({id:n.id,tag:n.tagName,direction:getComputedStyle(n).direction,bidi:getComputedStyle(n).unicodeBidi})));for(const n of isolation){assert.equal(n.tag,'BDI');assert.match(n.bidi,/isolate/);}row.effect={case:'SUP-2047',customer,saved:corrected,isolation};row.focal=await capture(p,`${row.tag}-effect`,'.request');
  },
  async 'aria-alternative-input'(p,row,bad) {
    assert.equal(await p.locator('main #order-proof').count(),0,'test-only ID proof belongs outside organizer flow');assert.equal(await p.locator('main').innerText().then(t=>t.includes('stable ID:')),false,'row facts must serve the organizer');
    const order=()=>p.locator('#queue>li').evaluateAll(ns=>ns.map(n=>n.dataset.id));await key(p,'#up-habitat');assert.deepEqual(await order(),['orientation','habitat','soil','water']);
    if(bad){await focusIs(p,'up-soil');await key(p,'#up-habitat');assert.deepEqual(await order(),['orientation','soil','habitat','water']);assert.match(await text(p,'#move-status'),/Find a habitat moved up to position 2/);row.discriminant='Repeated habitat command moves soil and reports habitat in a position it does not occupy.';return;}
    await focusIs(p,'up-habitat');await p.keyboard.press('Enter');assert.deepEqual(await order(),['orientation','habitat','soil','water']);await focusIs(p,'up-habitat');assert.equal(await attr(p,'#move-status','data-rejected'),'true');assert.match(await text(p,'#move-status'),/Orientation must stay first/);await p.keyboard.press('Tab');await focusIs(p,'down-habitat');await p.keyboard.press('Enter');assert.deepEqual(await order(),['orientation','soil','habitat','water']);await focusIs(p,'down-habitat');await key(p,'#undo');assert.deepEqual(await order(),['orientation','habitat','soil','water']);await focusIs(p,'undo');await key(p,'#undo');assert.deepEqual(await order(),['orientation','soil','habitat','water']);await key(p,'#undo');assert.deepEqual(await order(),['orientation','soil','habitat','water']);assert.equal(await attr(p,'#move-status','data-rejected'),'true');await key(p,'#down-water');assert.deepEqual(await order(),['orientation','soil','habitat','water']);await focusIs(p,'down-water');
    await key(p,'#up-water');assert.deepEqual(await order(),['orientation','soil','water','habitat']);await key(p,'#reset');assert.deepEqual(await order(),['orientation','soil','habitat','water']);assert.equal(await attr(p,'#route-planner','data-order'),'orientation,soil,habitat,water');await eqText(p,'#order-proof','orientation → soil → habitat → water');row.effect={order:await order(),boundary:'orientation remains first',focusAfterRejection:'same named button'};row.focal=await capture(p,`${row.tag}-effect`,'.queue-section');
  }
};

async function open(browser,id,variant,viewport,javaScriptEnabled=true) {
  const context=await browser.newContext({viewport,javaScriptEnabled,reducedMotion:'reduce',deviceScaleFactor:2});const p=await context.newPage();p.setDefaultTimeout(7000);
  const errors=[],external=[];p.on('pageerror',e=>errors.push(String(e)));await p.route('**/*',r=>{if(new URL(r.request().url()).origin===baseURL.origin)return r.continue();external.push(r.request().url());return r.abort();});
  const response=await p.goto(`${base}${id}/${variant?'?variant=misleading':''}`,{waitUntil:'load'});assert.equal(response.status(),200,'local specimen route');const html_sha256=crypto.createHash('sha256').update(await response.body()).digest('hex');await p.locator('main').waitFor();return {context,p,errors,external,html_sha256};
}
async function execute(browser,id) {
  for(const mode of ['wide','narrow','text2','nojs','misleading']) {
    const tag=`${id}-${mode}`;const row={id,tag,mode};report.results.push(row);let c;
    try {
      c=await open(browser,id,mode==='misleading',mode==='wide'||mode==='misleading'?{width:1280,height:900}:{width:390,height:844},mode!=='nojs');
      row.html_sha256=c.html_sha256;
      assert.equal(await attr(c.p,'html','data-design-language'),id);assert.ok((await text(c.p,'main')).length>80,'essential specimen content');
      if(mode==='text2')await enlargeText(c.p);row.geometry=await geometry(c.p);row.screenshot=await capture(c.p,tag);
      if(mode==='wide'||mode==='misleading') {assert.ok(journeys[id],`No effect journey for ${id}`);await journeys[id](c.p,row,mode==='misleading');}
      if(mode==='misleading')row.effect_screenshot=await capture(c.p,`${tag}-effect`);
      if(mode==='text2'){const first=c.p.locator('main button:visible:not([disabled]), main input:visible, main select:visible, main a[href]:visible').first();if(await first.count()){await first.focus();await c.p.keyboard.press('Tab');row.focus=await c.p.evaluate(()=>({id:document.activeElement?.id,tag:document.activeElement?.tagName,outline:getComputedStyle(document.activeElement).outlineStyle}));assert.equal(row.focus.outline,'solid','visible keyboard focus');row.focal=`${tag}-focus.png`;await c.p.screenshot({path:path.join(output,row.focal),fullPage:false});}}
      if(mode==='narrow'||mode==='text2'){if(id==='android-adaptive-continuity'){await key(c.p,'#select-orchard');await focusIs(c.p,'title-orchard');}if(focal[id])row.focal_detail=await capture(c.p,`${tag}-detail`,focal[id]);}
      assert.deepEqual(c.errors,[],'page errors');assert.deepEqual(c.external,[],'external requests');row.status='pass';
    }catch(e){row.status='fail';row.error=String(e);report.failures.push({tag,error:String(e)});if(c)try{row.failure_screenshot=await capture(c.p,`${tag}-failure`);}catch{}}
    finally {if(c)await c.context.close();}
  }
}
(async()=>{await fs.mkdir(output,{recursive:true});const browser=await connect();report.browser=browser.version();try{for(const id of ids)await execute(browser,id);}finally{await browser.close();await fs.writeFile(path.join(output,'report.json'),JSON.stringify(report,null,2)+'\n');}console.log(JSON.stringify({browser:report.browser,results:report.results.length,failures:report.failures,report:path.join(output,'report.json')},null,2));if(report.failures.length)process.exitCode=1;})().catch(e=>{console.error(e);process.exitCode=1;});
