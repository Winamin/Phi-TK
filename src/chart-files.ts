import { invoke } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';
import { open } from '@tauri-apps/plugin-dialog';
import type { ChartInfo } from './model';

export async function pickChartFiles(directory = false, multiple = false): Promise<string[]> {
  if (platform() === 'android') return invoke('pick_chart_files', { directory, multiple });
  const result = await open({ directory, multiple, filters: directory ? undefined : [{ name: 'Chart', extensions: ['zip', 'pez'] }] });
  return result ? (Array.isArray(result) ? result : [result]) : [];
}

export function importChart(path: string): Promise<{ path: string; info: ChartInfo }> {
  return invoke('import_chart', { path });
}
