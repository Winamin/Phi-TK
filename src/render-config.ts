import type { RenderConfig } from './model';

export function parseBitrateBps(raw: string): number | null {
  const match = raw.trim().match(/^(\d+(?:\.\d+)?)\s*(k(?:bps)?|m(?:bps)?|bps)?$/i);
  if (!match) return null;
  const unit = (match[2] || '').toLowerCase();
  const multiplier = unit.startsWith('m') ? 1e6 : unit.startsWith('k') ? 1e3 : 1;
  const value = Number(match[1]) * multiplier;
  return Number.isFinite(value) && value >= 200000 && value <= 2147483647 ? Math.round(value) : null;
}

export function normalizeRenderConfig(config: RenderConfig): RenderConfig {
  const bitrate = config.bitrateControl?.toUpperCase() === 'CRF' ? '8M' : config.bitrate?.trim() || '8M';
  return {
    ...config,
    videoCodec: ['hevc', 'h265'].includes(config.videoCodec?.toLowerCase()) ? 'hevc' : 'h264',
    bitrateControl: 'CBR',
    bitrate,
    targetAudio: 48000,
  };
}
