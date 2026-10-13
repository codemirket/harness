/* Original educational specimen: editable SVG + finite, instance-owned poses.
 * Status changes synchronously. No animation callback changes application state. */
const poses = {
  ready: { body: 'translateY(0px) rotate(0deg)', face: 'translate(0px, 0px)', label: 'Ready — waiting for the next task.' },
  working: { body: 'translateY(-5px) rotate(-5deg)', face: 'translate(3px, -2px)', label: 'Working — attending to the current task.' },
  rest: { body: 'translateY(2px) rotate(3deg)', face: 'translate(0px, 3px)', label: 'Rest — no task is running.' }
};

class CharacterStudy extends HTMLElement {
  connectedCallback() {
    this.abort?.abort();
    this.abort = new AbortController();
    const options = { signal: this.abort.signal };
    this.motion = new Map();
    this.parts = [this.querySelector('[data-rig]'), this.querySelector('[data-face]')];
    this.preference = matchMedia('(prefers-reduced-motion: reduce)');
    this.visible = false; // Observer establishes visibility; no mount animation.
    this.preference.addEventListener('change', () => this.settle(), options);
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) this.settle();
    }, options);
    this.observer = new IntersectionObserver(([entry]) => {
      this.visible = entry.isIntersecting;
      if (!this.visible) this.settle();
    });
    this.observer.observe(this.querySelector('.character'));
    for (const button of this.querySelectorAll('button[data-pose]')) {
      button.disabled = false;
      button.addEventListener('click', () => this.select(button.dataset.pose), options);
    }
    this.select(this.dataset.pose || 'ready', false);
  }

  select(name, animate = true) {
    const pose = poses[name];
    if (!pose || !this.isConnected) return;
    this.dataset.pose = name;
    this.querySelector('[role=status]').textContent = pose.label;
    this.querySelectorAll('button[data-pose]').forEach(button => {
      button.setAttribute('aria-pressed', String(button.dataset.pose === name));
    });
    const canMove = animate && !this.preference.matches && !document.hidden && this.visible;
    [pose.body, pose.face].forEach((target, index) => {
      const part = this.parts[index];
      // Sample before cancellation: rapid reversal starts at the displayed pose.
      const from = getComputedStyle(part).transform;
      this.motion.get(part)?.cancel();
      this.motion.delete(part);
      part.style.transform = target; // Persistent endpoint, independent of playback.
      if (!canMove || typeof part.animate !== 'function') return;
      const animation = part.animate([{ transform: from }, { transform: target }], {
        duration: 240, easing: 'cubic-bezier(.2,.7,.3,1)'
      });
      this.motion.set(part, animation);
      animation.finished.then(() => {
        // An obsolete completion cannot cancel or replace a newer transition.
        if (this.motion.get(part) === animation) {
          this.motion.delete(part);
          animation.cancel();
        }
      }).catch(() => {}); // Cancellation is the expected interruption path.
    });
  }

  settle() {
    // Inline styles already contain the latest semantic state's readable pose.
    for (const animation of this.motion.values()) animation.cancel();
    this.motion.clear();
  }

  disconnectedCallback() {
    this.settle();
    this.abort.abort();
    this.observer.disconnect();
  }
}

customElements.define('character-study', CharacterStudy);
