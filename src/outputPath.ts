const KEY = 'outputPath';

export function getOutputPath(): string {
  return (localStorage.getItem(KEY) || '').trim();
}

export function getOutputPathArg(): string | null {
  return getOutputPath() || null;
}

export function setOutputPath(path: string): void {
  const value = path.trim();
  if (value) localStorage.setItem(KEY, value);
  else localStorage.removeItem(KEY);
}

export function clearOutputPath(): void {localStorage.removeItem(KEY);}
