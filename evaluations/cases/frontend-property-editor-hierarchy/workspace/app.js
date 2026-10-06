const names = { en: 'English', fr: 'French' };
const locale = document.querySelector('#locale');
const saveStatus = document.querySelector('#save-status');
const fields = [document.querySelector('#property-name'), document.querySelector('#description')];
const saved = { en: { name: 'Harbor House', description: 'A quiet place near the harbor.' }, fr: { name: 'Maison du Port', description: 'Un lieu paisible près du port.' } };
function showLocale() {
  const content = saved[locale.value];
  fields[0].value = content.name;
  fields[1].value = content.description;
  document.querySelector('#locale-context').textContent = `Editing ${names[locale.value]} content`;
  saveStatus.textContent = 'All changes saved';
}
locale.addEventListener('change', showLocale);
fields.forEach(field => field.addEventListener('input', () => { saveStatus.textContent = 'Unsaved changes'; }));
document.querySelector('#save').addEventListener('click', () => {
  saved[locale.value] = { name: fields[0].value, description: fields[1].value };
  saveStatus.textContent = 'All changes saved';
});
document.querySelector('#theme-toggle').addEventListener('click', () => {
  document.documentElement.dataset.theme = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
});
