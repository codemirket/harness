/* Native browser study: finite motion, DOM semantics, a bounded Canvas field.
 * The drawing is decorative. Text and the SVG poster work without this script. */
(() => {
  'use strict';
  const media = matchMedia('(prefers-reduced-motion: reduce)');
  let reduced = media.matches;
  const field = document.querySelector('.field');
  const canvas = field.querySelector('canvas');
  const context = canvas.getContext('2d');
  const card = document.querySelector('.session-card');
  const choices = [...document.querySelectorAll('[data-mode]')];
  const effects = new Set();
  const controller = new AbortController();
  let width = 0, height = 0, scrollProgress = 0, raf = 0;
  let visible = true, active = true, mode = 'focus', burstStart = 0;
  let resizeObserver, intersectionObserver;
  const modes = {
    focus: { number:'MODE / 01', time:'50 MIN + 10 MIN REST', title:'One thing.<br>All the way in.', description:'A clear target and a generous stretch of quiet. Close the extra tabs. Follow one thread until it becomes something.', link:'Build your focus ritual' },
    explore: { number:'MODE / 02', time:'25 MIN + 5 MIN REST', title:'Follow a spark.<br>See what connects.', description:'A loose question and permission to wander. Collect fragments, sketch possibilities, and leave room for the unexpected connection.', link:'Build your exploration ritual' }
  };
  function animate(element, frames, options) {
    if (reduced || !element.animate) return;
    const effect = element.animate(frames, options);
    effects.add(effect);
    effect.finished.catch(() => {}).finally(() => effects.delete(effect));
    return effect;
  }
  function stopEffects() { effects.forEach(effect => effect.cancel()); effects.clear(); }
  function measure() {
    const bounds = field.getBoundingClientRect();
    width = bounds.width; height = bounds.height;
    const ratio = Math.min(devicePixelRatio || 1, 2);
    canvas.width = Math.round(width * ratio); canvas.height = Math.round(height * ratio);
    context?.setTransform(ratio, 0, 0, ratio, 0, 0);
    progress();
  }
  function progress() {
    const rect = field.getBoundingClientRect();
    scrollProgress = Math.max(0, Math.min(1, (innerHeight - rect.top) / (innerHeight + rect.height)));
    requestDraw();
  }
  function draw(now) {
    raf = 0;
    if (!active || !context || width === 0 || height === 0) return;
    // A 1.4 second activation beat settles; idle/hidden/offscreen/reduced do not loop.
    const elapsed = Math.max(0, now - burstStart);
    const playing = burstStart > 0 && elapsed < 1400 && !reduced && visible && !document.hidden;
    const impulse = playing ? Math.sin(Math.PI * elapsed / 1400) * .28 : 0;
    const phase = reduced ? .48 : scrollProgress;
    context.clearRect(0, 0, width, height);
    const gradient = context.createLinearGradient(0, 0, width, height);
    gradient.addColorStop(0, '#ccff5e'); gradient.addColorStop(1, '#bdabff');
    context.strokeStyle = gradient; context.lineWidth = .9;
    const rows = width < 600 ? 24 : 36;
    const points = Math.min(160, Math.ceil(width / 7));
    // Keep a quiet caption band at every scroll phase and narrow size.
    const drawingHeight = Math.max(1, height - 68);
    const spacing = Math.min(4, drawingHeight * .42 / (rows - 1));
    const amplitude = Math.min(55 + phase * 28, drawingHeight * .26);
    for (let line = 0; line < rows; line++) {
      context.beginPath();
      for (let point = 0; point <= points; point++) {
        const x = point / points * width;
        const u = x / width;
        const envelope = Math.sin(u * Math.PI);
        const wave = Math.sin(u * Math.PI * 2 + line * .08 + phase * 2.4 + impulse);
        const y = drawingHeight * .5 + 8 + (line - (rows - 1) / 2) * spacing + wave * amplitude * envelope;
        if (point === 0) context.moveTo(x, y); else context.lineTo(x, y);
      }
      context.globalAlpha = .35 + (line / rows) * .55;
      context.stroke();
    }
    context.globalAlpha = 1;
    field.dataset.canvas = 'ready';
    if (playing) raf = requestAnimationFrame(draw);
  }
  function requestDraw() {
    if (active && context && !raf && !document.hidden && visible) raf = requestAnimationFrame(draw);
  }
  function select(next) {
    if (!modes[next] || mode === next) return;
    stopEffects();
    mode = next;
    const data = modes[mode];
    choices.forEach(button => button.setAttribute('aria-pressed', String(button.dataset.mode === mode)));
    card.dataset.mode = mode;
    document.querySelector('#mode-number').textContent = data.number;
    document.querySelector('#mode-time').textContent = data.time;
    document.querySelector('#mode-title').innerHTML = data.title; // Closed, authored strings only.
    document.querySelector('#mode-description').textContent = data.description;
    card.querySelector('.card-link').firstChild.textContent = `${data.link} `;
    burstStart = performance.now(); requestDraw();
  }
  function preferenceChanged(event) {
    reduced = event.matches;
    stopEffects();
    if (reduced) burstStart = 0;
    if (raf) cancelAnimationFrame(raf);
    raf = 0; requestDraw();
  }
  function start() {
    active = true;
    reduced = media.matches;
    document.querySelector('.choices').hidden = false;
    measure();
    resizeObserver = new ResizeObserver(measure); resizeObserver.observe(field);
    intersectionObserver = new IntersectionObserver(entries => {
      visible = entries[0].isIntersecting;
      if (!visible) { cancelAnimationFrame(raf); raf = 0; burstStart = 0; }
      else requestDraw();
    });
    intersectionObserver.observe(field);
  }
  function suspend() {
    active = false; cancelAnimationFrame(raf); raf = 0; burstStart = 0;
    stopEffects(); resizeObserver?.disconnect(); intersectionObserver?.disconnect();
  }
  choices.forEach(button => button.addEventListener('click', () => select(button.dataset.mode), {signal:controller.signal}));
  addEventListener('scroll', progress, {passive:true, signal:controller.signal});
  addEventListener('resize', progress, {passive:true, signal:controller.signal});
  media.addEventListener('change', preferenceChanged, {signal:controller.signal});
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) { cancelAnimationFrame(raf); raf = 0; burstStart = 0; stopEffects(); }
    else requestDraw();
  }, {signal:controller.signal});
  addEventListener('pagehide', event => { suspend(); if (!event.persisted) controller.abort(); });
  addEventListener('pageshow', event => { if (event.persisted) start(); });
  start();
  if (!reduced) {
    const schedule = [
      [document.querySelector('.hero .eyebrow'), 0, 380, 7],
      [document.querySelector('h1'), 80, 620, 19],
      [document.querySelector('.hero-bottom'), 240, 450, 10]
    ];
    schedule.forEach(([element, delay, duration, distance]) => animate(element,
      [{opacity:0, transform:`translateY(${distance}px)`}, {opacity:1, transform:'translateY(0)'}],
      {delay, duration, easing:'cubic-bezier(.2,.7,.2,1)', fill:'backwards'}));
    burstStart = performance.now(); requestDraw();
  }
})();
