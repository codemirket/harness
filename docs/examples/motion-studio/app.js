const main = document.querySelector('main');
const notice = document.querySelector('#notice');
const reduce = matchMedia('(prefers-reduced-motion: reduce)');
const studies = [
  { slug: 'field-notes', number: '01', title: 'Field notes', subtitle: 'A folio for close observation' },
  { slug: 'tidelines', number: '02', title: 'Tidelines', subtitle: 'An atlas of changing edges' }
];
let currentURL, request = 0, pending, activeTransition, scene, saveFrame;
let restoring = false;
history.scrollRestoration = 'manual';

// This study has one collection and two folio entries, not a general router.
const entryFor = url => studies.find(s => url.pathname === `/studies/${s.slug}`);
const isCollection = url => url.pathname === '/';
const isStudyURL = url => isCollection(url) || Boolean(entryFor(url));
const safe = text => String(text).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

function paperObject(study, shared = false) {
  return `${shared ? `<div class="object-presentation" data-shared="${study.slug}">` : ''}<div class="object-camera"><div class="book-travel" data-motion="travel"><div class="book-turn" data-motion="tilt"><div class="book-block">
    <div class="paper-edge"></div><div class="inner-sheet"><div class="sheet-offset" data-motion="sheet"><span class="small-label">Paper room / ${safe(study.number)}</span><div class="inside-heading">${safe(study.inside || 'A field for<br>observation.').replaceAll('&lt;br&gt;', '<br>')}</div><div class="inside-rule"></div><div class="inside-lines"></div><span class="tiny">A DIGITAL STUDY IN PAPER</span></div></div>
    <div class="cover-hinge" data-motion="hinge"><div class="cover-face cover-front ${study.slug}"><div class="cover-graphic"><div class="field-orb"></div><div class="field-orbit"></div></div><span class="cover-overline">PAPER ROOM / STUDIES IN FORM</span><span class="cover-title">${study.slug === 'field-notes' ? 'Field<br>notes' : 'Tide<br>lines'}</span><span class="cover-number">No. ${safe(study.number)} — ${study.slug === 'field-notes' ? 'OBSERVATION' : 'CHANGING EDGES'}</span><span class="cover-spine"></span></div><div class="cover-face cover-back"><span class="back-label">A FIELD<br>FOR WHAT COMES NEXT.<br><br>PAPER ROOM — ${safe(study.number)}</span></div></div>
  </div></div></div></div>${shared ? '</div>' : ''}`;
}

function indexView() {
  return `<section class="index-intro"><div><p class="eyebrow">Studies in form / 01—02</p><h1 tabindex="-1" data-focus-key="heading">A surface.<br>A small world.</h1></div><div class="index-deck"><p>Two paper objects, studied through proportion, colour and the way a cover opens.</p><span class="small-label">Original digital compositions / October 2026</span></div></section>
  <section class="collection" aria-label="The collection">${studies.map(s => `<a class="study-link" href="/studies/${s.slug}" data-focus-key="study-${s.slug}"><div class="art-field" aria-hidden="true"><span class="field-corner">FIG. ${s.number}</span>${paperObject(s, true)}<span class="field-scale">240 × 320</span></div><div class="study-caption"><div><h2>${s.title}</h2><p>${s.subtitle}</p></div><span class="arrow" aria-hidden="true">↗</span></div></a>`).join('')}</section>
  <section class="approach" id="approach"><h2 tabindex="-1" data-focus-key="approach">A study in<br>ordinary things.</h2><div><p>A cover, a seam, a page. This development study uses a familiar object to explore how a digital interface can carry identity from a collection into a closer view.</p><p>Each entry includes a short, optional opening sequence. The designs and dimensions are study material; no manufactured product or endorsement is implied.</p></div></section>`;
}

function detailView(study) {
  const next = studies.find(s => s.slug === study.next);
  return `<div class="detail-top"><a href="/" data-focus-key="back">← Back to the collection</a><span>Object study ${study.number} / 02</span></div>
  <section class="detail-hero"><div class="art-field hero-portrait" aria-hidden="true"><span class="field-corner">FIG. ${study.number} / FRONT FACE</span>${paperObject(study, true)}<span class="field-scale">240 × 320</span></div><div class="detail-copy"><p class="eyebrow">${safe(study.subtitle)}</p><h1 tabindex="-1" data-focus-key="heading">${safe(study.title)}</h1><p class="deck">${safe(study.deck)}</p><dl class="detail-facts"><div><dt>Proportion</dt><dd>${safe(study.format)}</dd></div><div><dt>Colour</dt><dd>${safe(study.palette)}</dd></div></dl><a href="#construction" class="text-link">Explore the construction <span aria-hidden="true">↓</span></a></div></section>
  <section class="assembly-section" id="construction"><div class="assembly-copy"><p class="eyebrow">The construction</p><h2 tabindex="-1" data-focus-key="construction">One edge.<br>A wider field.</h2><p>${safe(study.construction)}</p><div class="motion-controls"><button class="primary" type="button" data-cover-toggle aria-pressed="false">Open cover</button><button type="button" data-replay>Replay</button><button type="button" data-pause disabled>Pause</button></div><p class="motion-status" aria-live="polite" data-motion-status>Closed — the front face is in view.</p></div><figure class="assembly-figure"><div class="assembly-stage" data-assembly data-open="false" aria-hidden="true"><span class="small-label">${safe(study.title)} / an opening study</span>${paperObject(study)}<div class="annotation-group" data-motion="note">01 — SINGLE EDGE<br>02 — INSIDE FACE</div><span class="stage-end">THE OBJECT, UNFOLDED</span></div><figcaption>The cover rotates around its left seam. The object, paper and annotation have separate movements.</figcaption></figure></section>
  <section class="reading-note"><h3>Notes on the composition</h3><p>${safe(study.note)}</p></section><div class="next-study"><div><span class="small-label">Continue the collection</span><a href="/studies/${next.slug}" data-focus-key="next">${next.title}<span aria-hidden="true">↗</span></a></div><a href="/" class="small-label" data-focus-key="all-studies">All studies ↑</a></div>`;
}

function saveEntry() {
  if (restoring || !currentURL || currentURL.pathname !== location.pathname) return;
  const focus = document.activeElement?.dataset.focusKey || null;
  const state = { ...history.state, paperRoom: { scroll: [scrollX, scrollY], focus, path: location.pathname } };
  history.replaceState(state, '', location.href);
}

function clearNames() {
  main.querySelectorAll('[data-shared]').forEach(node => node.style.removeProperty('view-transition-name'));
}
function nameShared(slug) {
  clearNames();
  const node = main.querySelector(`[data-shared="${slug}"]`);
  if (node) node.style.viewTransitionName = `paper-${slug}`;
}

function showNotice(message, retry) {
  notice.replaceChildren(document.createTextNode(message));
  if (retry) {
    const button = document.createElement('button');
    button.textContent = 'Try again';
    button.addEventListener('click', retry, { once: true });
    notice.append(button);
  }
  notice.hidden = false;
}

async function navigate(url, mode = 'push', restoredState = null) {
  const id = ++request;
  pending?.abort();
  activeTransition?.skipTransition();
  activeTransition = null;
  clearNames();
  pending = new AbortController();
  if (mode === 'push') saveEntry();
  if (mode === 'pop' && currentURL?.pathname === url.pathname) {
    notice.hidden = true;
    currentURL = url;
    restorePosition(url, mode, restoredState);
    return;
  }
  const entry = entryFor(url);
  try {
    let content;
    if (entry) {
      showNotice(`Opening ${entry.title}…`);
      const response = await fetch(`/content/${entry.slug}.json`, { signal: pending.signal });
      if (!response.ok) throw new Error(`Study data: ${response.status}`);
      const data = await response.json();
      if (data.slug !== entry.slug || !data.deck || !data.note || !studies.some(s => s.slug === data.next)) throw new Error('Incomplete study data');
      content = detailView(data);
    } else content = isCollection(url) ? indexView() : '<section class="entry"><p class="eyebrow">Study not found</p><h1 tabindex="-1" data-focus-key="heading">A missing page.</h1><p>This address is not part of the collection.</p><a href="/" class="text-link">Return to the collection →</a></section>';
    if (id !== request) return;
    // Data is ready before capture. The callback is synchronous and commits once.
    const sharedSlug = entry?.slug || entryFor(currentURL || url)?.slug;
    let invoked = false;
    const commit = () => {
      if (id !== request || invoked) return;
      invoked = true;
      scene?.dispose(); scene = null;
      restoring = true;
      main.innerHTML = content;
      main.dataset.route = url.pathname;
      document.title = entry ? `${entry.title} — Paper room` : isCollection(url) ? 'Paper room — Studies in paper' : 'Study not found — Paper room';
      if (mode === 'push') history.pushState({ paperRoom: { scroll: [0, 0], focus: 'heading', path: url.pathname } }, '', url);
      else if (mode === 'initial') history.replaceState({ ...history.state, paperRoom: { scroll: [0, 0], focus: 'heading', path: url.pathname } }, '', url);
      currentURL = url;
      if (sharedSlug && activeTransition) nameShared(sharedSlug);
      scene = createScene();
      restorePosition(url, mode, restoredState);
      restoring = false;
      notice.hidden = true;
    };
    if (mode === 'initial' || reduce.matches || typeof document.startViewTransition !== 'function') commit();
    else {
      if (sharedSlug) nameShared(sharedSlug);
      const transition = document.startViewTransition(commit);
      activeTransition = transition;
      // A skipped capture is visual failure, never a reason to replay the commit.
      void transition.ready.catch(() => {});
      const release = () => {
        if (activeTransition === transition && id === request) {
          clearNames(); activeTransition = null;
        }
      };
      void transition.finished.then(release, release);
      await transition.updateCallbackDone;
    }
  } catch (error) {
    if (id !== request || error.name === 'AbortError') return;
    restoring = false;
    clearNames();
    showNotice('This study could not be opened.', () => navigate(url, mode, restoredState));
  }
}

function restorePosition(url, mode, state) {
  const previous = state?.paperRoom;
  const anchor = url.hash && document.getElementById(decodeURIComponent(url.hash.slice(1)));
  const target = (mode === 'pop' && previous?.focus ? document.querySelector(`[data-focus-key="${CSS.escape(previous.focus)}"]`) : anchor) || main.querySelector('h1');
  target?.focus({ preventScroll: true });
  if (mode === 'pop' && previous?.scroll) window.scrollTo({ left: previous.scroll[0], top: previous.scroll[1], behavior: 'instant' });
  else if (anchor) anchor.scrollIntoView({ behavior: 'instant', block: 'start' });
  else window.scrollTo({ top: 0, left: 0, behavior: 'instant' });
}

function createScene() {
  const root = main.querySelector('[data-assembly]');
  if (!root) return null;
  const toggle = main.querySelector('[data-cover-toggle]');
  const replay = main.querySelector('[data-replay]');
  const pause = main.querySelector('[data-pause]');
  const status = main.querySelector('[data-motion-status]');
  const listeners = new AbortController();
  let tracks = [], generation = 0, paused = false, disposed = false;
  const owners = ['travel', 'tilt', 'hinge', 'sheet', 'note'].map(name => root.querySelector(`[data-motion="${name}"]`));
  // One clock; each row owns a different node. The hinge's origin is its seam.
  const schedule = [{ at: 0, duration: 850 }, { at: 55, duration: 1120 }, { at: 180, duration: 1130 }, { at: 510, duration: 790 }, { at: 870, duration: 390 }];
  const read = node => ({ transform: getComputedStyle(node).transform, opacity: getComputedStyle(node).opacity });
  const stop = () => { generation++; tracks.forEach(a => a.cancel()); tracks = []; paused = false; };
  const updateControls = running => {
    const open = root.dataset.open === 'true';
    toggle.textContent = open ? 'Close cover' : 'Open cover';
    toggle.setAttribute('aria-pressed', String(open));
    pause.disabled = !running;
    pause.textContent = 'Pause';
    status.textContent = reduce.matches ? `${open ? 'Open' : 'Closed'} — static view for reduced motion.` : running ? (open ? 'Turning the cover. Revealing the inside.' : 'Returning to the front face.') : open ? 'Open — the inside face is in view.' : 'Closed — the front face is in view.';
  };
  function play(open, restart = false) {
    if (disposed) return;
    let from = owners.map(read);
    stop();
    if (restart) { root.dataset.open = 'false'; from = owners.map(read); }
    root.dataset.open = String(open);
    if (reduce.matches || typeof owners[0].animate !== 'function' || document.timeline.currentTime === null) { updateControls(false); return; }
    const to = owners.map(read), own = generation;
    const clock = document.timeline.currentTime;
    tracks = owners.map((node, i) => {
      const row = schedule[i];
      const frames = i === 4 ? [from[i], to[i]] : [{ transform: from[i].transform }, { transform: to[i].transform }];
      const animation = node.animate(frames, { delay: open ? row.at : row.at * .25, duration: open ? row.duration : row.duration * .7, easing: 'cubic-bezier(.22,.68,.2,1)', fill: 'both' });
      animation.startTime = clock;
      void animation.finished.catch(error => { if (error.name !== 'AbortError') console.error(error); });
      return animation;
    });
    updateControls(true);
    Promise.all(tracks.map(a => a.finished)).then(() => {
      if (disposed || generation !== own) return;
      stop(); updateControls(false); // Responsive base styles already own the settled pose.
    }, error => { if (error.name !== 'AbortError') console.error(error); });
  }
  toggle.addEventListener('click', () => play(root.dataset.open !== 'true'), { signal: listeners.signal });
  replay.addEventListener('click', () => play(true, true), { signal: listeners.signal });
  pause.addEventListener('click', () => {
    if (!tracks.length) return;
    paused = !paused;
    tracks.forEach(a => paused ? a.pause() : a.play());
    pause.textContent = paused ? 'Resume' : 'Pause';
    status.textContent = paused ? 'Paused — continue when you are ready.' : 'Turning the cover. Revealing the inside.';
  }, { signal: listeners.signal });
  updateControls(false);
  return { settle() { stop(); updateControls(false); }, dispose() { disposed = true; stop(); listeners.abort(); } };
}

document.addEventListener('click', event => {
  const link = event.target.closest('a[href]');
  if (!link || event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || link.hasAttribute('download') || (link.target && link.target !== '_self')) return;
  const url = new URL(link.href);
  if (url.origin !== location.origin || !isStudyURL(url)) return;
  if (url.pathname === location.pathname && url.search === location.search) {
    request++; pending?.abort(); activeTransition?.skipTransition(); activeTransition = null; clearNames(); notice.hidden = true;
    if (url.hash) return; // Ordinary anchors keep the browser's native history behavior.
    event.preventDefault(); main.querySelector('h1')?.focus({ preventScroll: true }); window.scrollTo({ top: 0, behavior: 'instant' });
    return;
  }
  event.preventDefault(); void navigate(url);
});
addEventListener('popstate', event => { void navigate(new URL(location.href), 'pop', event.state); });
const scheduleSave = () => { cancelAnimationFrame(saveFrame); saveFrame = requestAnimationFrame(saveEntry); };
addEventListener('scroll', scheduleSave, { passive: true });
document.addEventListener('focusin', scheduleSave);
reduce.addEventListener('change', event => { if (event.matches) { activeTransition?.skipTransition(); scene?.settle(); } });
addEventListener('pagehide', () => { saveEntry(); pending?.abort(); activeTransition?.skipTransition(); scene?.dispose(); scene = null; cancelAnimationFrame(saveFrame); });
addEventListener('pageshow', event => { if (event.persisted && !scene) scene = createScene(); });
void navigate(new URL(location.href), 'initial');
