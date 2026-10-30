// Installation eligibility is a binary floor, not a qualification claim or an OS-version ceiling.
export function supportsMacOSVersion(version) {
  if (typeof version !== 'string') return false;
  const parts = /^(\d+)\.(\d+)(?:\.(\d+))?$/.exec(version);
  return parts !== null && parts.slice(1).every(part => part === undefined || Number.isSafeInteger(Number(part))) && Number(parts[1]) >= 26;
}
