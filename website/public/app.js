const button = document.querySelector('#copy');
const prompt = document.querySelector('#install-prompt');
const status = document.querySelector('#copy-status');
button.hidden = false;
button.addEventListener('click', async () => {
  try {
    await navigator.clipboard.writeText(prompt.value);
    status.textContent = 'Copied. Paste it into Codex or Claude to start setup.';
  } catch {
    prompt.focus();
    prompt.select();
    status.textContent = 'Copy unavailable. The prompt is selected; copy it with your keyboard or touch menu.';
  }
});

// Progressive enhancement: without JavaScript all three explanations stay readable.
const controls = document.querySelector('.workflow-controls');
const stages = [...document.querySelectorAll('[data-stage]')];
const panels = [...document.querySelectorAll('[data-panel]')];
function selectStage(stage) {
  for (const control of stages) control.setAttribute('aria-pressed', String(control.dataset.stage === stage));
  for (const panel of panels) panel.hidden = panel.dataset.panel !== stage;
}
if (controls) {
  controls.hidden = false;
  selectStage('scope');
  for (const control of stages) control.addEventListener('click', () => selectStage(control.dataset.stage));
}
