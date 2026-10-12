/* Finite presentation effects; semantic state is always committed synchronously.
   Three owners: entrance shells, the sun's inner motion group, and the study orb. */
(() => {
  const root = document.body;
  const reduce = matchMedia('(prefers-reduced-motion: reduce)');
  const picker = document.querySelector('.phase-picker');
  const buttons = [...picker.querySelectorAll('button')];
  const sun = document.querySelector('[data-solar-motion]');
  const observation = document.querySelector('.observation');
  const title = document.querySelector('#observation-title');
  const description = document.querySelector('#observation-copy');
  const study = document.querySelector('.study-window');
  const orb = document.querySelector('.study-orb');
  const notes = [...document.querySelectorAll('.note')];
  const phases = {
    dawn: { title: 'The first light.', copy: 'Low, warm light. Long shadows. A quiet invitation to look again.', x: -35, y: 32 },
    noon: { title: 'The clear hour.', copy: 'A high sun sharpens the edges. Look for small shadows and bright, open color.', x: 27, y: -63 },
    dusk: { title: 'The last warmth.', copy: 'The angle softens. Color deepens. Notice what stays luminous as the day fades.', x: 66, y: 60 },
  };
  let phase = 'dawn';
  let active = false;
  let frame = 0;
  let studyVisible = false;
  let entranceObserver;
  let visibilityObserver;
  let resizeObserver;
  let inputController;
  const tracks = new Set();
  const propertyTracks = new Map();
  const revealed = new Set();

  function forget(animation) {
    tracks.delete(animation);
    for (const [node, current] of propertyTracks) {
      if (current === animation) propertyTracks.delete(node);
    }
  }

  function animate(node, frames, timing, propertyOwner = false) {
    if (reduce.matches || document.hidden || !active || !node.animate) return;
    const animation = node.animate(frames, { fill: 'none', ...timing });
    tracks.add(animation);
    if (propertyOwner) propertyTracks.set(node, animation);
    animation.finished.then(() => forget(animation), error => {
      forget(animation);
      if (error.name !== 'AbortError') console.error('SOLSTICE animation failed:', error);
    });
    return animation;
  }

  function cancelEffects() {
    for (const animation of tracks) animation.cancel();
    tracks.clear();
    propertyTracks.clear();
    if (frame) cancelAnimationFrame(frame);
    frame = 0;
  }

  function setPhase(next) {
    if (next === phase || !phases[next]) return;
    // Read the current presentation before cancellation: quick reversals retarget
    // from that position, never queue an obsolete state or completion callback.
    const from = getComputedStyle(sun).transform;
    const oldTextOpacity = getComputedStyle(observation).opacity;
    propertyTracks.get(sun)?.cancel();
    propertyTracks.get(observation)?.cancel();
    phase = next;
    const state = phases[phase];
    root.dataset.phase = phase;
    title.textContent = state.title;
    description.textContent = state.copy;
    buttons.forEach(button => button.setAttribute('aria-pressed', String(button.dataset.phase === phase)));
    const destination = `translate(${state.x}px, ${state.y}px)`;
    sun.style.transform = destination;
    animate(sun, [{ transform: from }, { transform: destination }], {
      duration: 820, easing: 'cubic-bezier(.22,.7,.22,1)',
    }, true);
    animate(observation, [{ opacity: Math.max(.65, Number(oldTextOpacity)) }, { opacity: 1 }], {
      duration: 300, easing: 'ease-out',
    }, true);
  }

  function updateScroll() {
    frame = 0;
    if (!active || document.hidden || !studyVisible || reduce.matches) return;
    // One geometry read per requested frame; natural scroll maps reversibly to
    // this illustration's passage through the viewport. No time-based catch-up.
    const box = study.getBoundingClientRect();
    const progress = Math.max(0, Math.min(1, (innerHeight - box.top) / (innerHeight + box.height)));
    const horizontal = (progress - .5) * box.width * .28;
    const vertical = (progress - .5) * box.height * .42;
    orb.style.transform = `translate(${horizontal.toFixed(2)}px, ${vertical.toFixed(2)}px)`;
  }

  function requestScroll() {
    if (!frame && active && studyVisible && !document.hidden && !reduce.matches) {
      frame = requestAnimationFrame(updateScroll);
    }
  }

  function onPreference() {
    cancelEffects();
    orb.style.removeProperty('transform');
    if (!reduce.matches) requestScroll();
  }

  function onVisibility() {
    if (document.hidden) {
      root.dataset.suspended = '';
      cancelEffects();
    } else {
      delete root.dataset.suspended;
      requestScroll();
    }
  }

  function entrance() {
    const origin = document.timeline.currentTime;
    const schedule = [
      [document.querySelector('.hero .eyebrow'), 0, 520, 8],
      [document.querySelector('.hero h1'), 90, 760, 22],
      [document.querySelector('.hero-description'), 220, 630, 12],
      [document.querySelector('.hero .text-link'), 300, 560, 8],
      [document.querySelector('.instrument'), 170, 900, 18],
      [document.querySelector('.hero-foot'), 480, 560, 8],
    ];
    for (const [node, at, duration, distance] of schedule) {
      const effect = animate(node, [
        { opacity: 0, transform: `translateY(${distance}px)` },
        { opacity: 1, transform: 'translateY(0)' },
      ], { duration, delay: at, easing: 'cubic-bezier(.2,.7,.2,1)', fill: 'backwards' });
      if (effect && origin !== null) effect.startTime = origin;
    }
  }

  function mount(playEntrance) {
    if (active) return;
    active = true;
    inputController = new AbortController();
    const { signal } = inputController;
    picker.hidden = false;
    buttons.forEach(button => button.addEventListener('click', () => setPhase(button.dataset.phase), { signal }));
    reduce.addEventListener('change', onPreference, { signal });
    document.addEventListener('visibilitychange', onVisibility, { signal });
    addEventListener('scroll', requestScroll, { passive: true, signal });
    addEventListener('resize', requestScroll, { signal });

    if ('IntersectionObserver' in window) {
      // Triggered notes reveal once; the study below uses progress, not replay.
      entranceObserver = new IntersectionObserver(entries => {
        for (const entry of entries) {
          if (!entry.isIntersecting || revealed.has(entry.target)) continue;
          revealed.add(entry.target);
          entranceObserver.unobserve(entry.target);
          animate(entry.target, [
            { opacity: .35, transform: 'translateY(16px)' },
            { opacity: 1, transform: 'translateY(0)' },
          ], { duration: 650, easing: 'cubic-bezier(.2,.7,.2,1)' });
        }
      }, { threshold: .22 });
      notes.forEach(note => { if (!revealed.has(note)) entranceObserver.observe(note); });
      visibilityObserver = new IntersectionObserver(entries => {
        for (const entry of entries) {
          if (entry.target === study) {
            studyVisible = entry.isIntersecting;
            if (studyVisible) requestScroll();
            else if (frame) { cancelAnimationFrame(frame); frame = 0; }
          } else if (!entry.isIntersecting) {
            // Finite effects that leave view settle; returning does not replay.
            for (const animation of [...tracks]) {
              if (entry.target.contains(animation.effect?.target)) animation.cancel();
            }
          }
        }
      });
      visibilityObserver.observe(study);
      visibilityObserver.observe(document.querySelector('.hero'));
      notes.forEach(note => visibilityObserver.observe(note));
    }
    if ('ResizeObserver' in window) {
      resizeObserver = new ResizeObserver(requestScroll);
      resizeObserver.observe(study);
    }
    onVisibility();
    if (playEntrance) entrance();
  }

  function dispose() {
    active = false;
    cancelEffects();
    inputController?.abort();
    entranceObserver?.disconnect();
    visibilityObserver?.disconnect();
    resizeObserver?.disconnect();
    root.dataset.suspended = '';
    orb.style.removeProperty('transform');
  }

  // pagehide also covers a bfcache suspension; pageshow restores listeners but
  // keeps the chosen phase and does not replay an already-seen introduction.
  addEventListener('pagehide', dispose);
  addEventListener('pageshow', () => mount(false));
  mount(true);
})();
