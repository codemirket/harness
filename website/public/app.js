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
