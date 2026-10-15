(() => {
  const root = document.querySelector('[data-diagram]');
  if (!root) return;
  const drawings = [...root.querySelectorAll('svg')].map(svg => ({
    svg,
    paths: [...svg.querySelectorAll('[data-route]')],
    packets: [...svg.querySelectorAll('[data-packet]')],
    rings: [...svg.querySelectorAll('[data-ring]')],
  }));
  const replay = root.querySelector('[data-replay]');
  const preference = matchMedia('(prefers-reduced-motion: reduce)');
  const awkward = new URLSearchParams(location.search).get('variant') === 'awkward';
  drawings.forEach(drawing => { drawing.lengths = drawing.paths.map(path => path.getTotalLength()); });
  let frame = 0;
  let start = 0;
  let current = 2400;
  const clamp = value => Math.max(0, Math.min(1, value));
  const ease = value => 1 - Math.pow(1 - clamp(value), 3);
  function paint(milliseconds) {
    const t = Math.max(0, Math.min(2400, milliseconds));
    current = t;
    const first = ease((t - 250) / 780);
    const second = awkward ? first : ease((t - 1230) / 780);
    const values = [first, second];
    const active = awkward ? 0 : t < 1030 ? 0 : t < 2010 ? 1 : 2;
    drawings.forEach(drawing => {
      drawing.paths.forEach((path, index) => {
        path.style.strokeDasharray = `${drawing.lengths[index]}`;
        path.style.strokeDashoffset = `${drawing.lengths[index] * (1 - values[index])}`;
        path.style.opacity = values[index] > 0 && values[index] < 1 ? '1' : '0';
        const point = path.getPointAtLength(drawing.lengths[index] * values[index]);
        drawing.packets[index].setAttribute('cx', point.x.toFixed(2));
        drawing.packets[index].setAttribute('cy', point.y.toFixed(2));
        drawing.packets[index].style.opacity = values[index] > 0 && values[index] < 1 ? '1' : '0';
      });
      drawing.rings.forEach((ring, index) => {
        const highlighted = active === index && (t < 2250 || awkward);
        ring.setAttribute('stroke-width', highlighted ? '6' : '2');
        ring.style.stroke = highlighted ? '#1687ac' : '';
      });
    });
  }
  function stop() {
    if (frame) cancelAnimationFrame(frame);
    frame = 0;
  }
  function settle() { stop(); paint(2400); }
  function tick(now) {
    paint(now - start);
    if (current < 2400 && !preference.matches && !document.hidden) frame = requestAnimationFrame(tick);
    else frame = 0;
  }
  function play() {
    stop();
    if (preference.matches || document.hidden) { settle(); return; }
    paint(0);
    start = performance.now();
    frame = requestAnimationFrame(tick);
  }
  replay.disabled = preference.matches;
  replay.addEventListener('click', play);
  preference.addEventListener('change', () => {
    replay.disabled = preference.matches;
    if (preference.matches) settle();
  });
  document.addEventListener('visibilitychange', () => { if (document.hidden) settle(); });
  window.diagramStudy = {play, settle, seek: milliseconds => { stop(); paint(milliseconds); }, state: () => ({time:current, running:frame !== 0, reduced:preference.matches})};
  if (preference.matches) settle(); else play();
})();
