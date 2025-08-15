export interface ChartInfo {
  name: string;
  level: string;
  charter: string;
  composer: string;
  illustrator: string;

  tip: string | null;

  aspectRatio: number;
  backgroundDim: number;
  HoldPartialCover: boolean;
}

export type TaskStatus =
  | {
      type: 'pending';
    }
  | {
      type: 'loading';
    }
  | {
      type: 'mixing';
    }
  | {
      type: 'rendering';
      progress: number;
      fps: number;
      estimate: number;
    }
  | {
      type: 'done';
      duration: number;
      output: string;
    }
  | {
      type: 'canceled';
    }
  | {
      type: 'failed';
      error: string;
    };

export interface Task {
  id: number;
  name: string;
  output: string;
  path: string;
  cover: string;
  status: TaskStatus;
}

export interface RenderConfig {
  resolution: number[];
  ffmpegPreset: string;
  endingLength: number;
  disableLoading: boolean;
  chartDebug: boolean;
  flidX: boolean;
  showProgressText: boolean;
  showTimeText: boolean;
  chartRatio: number;
  bufferSize: number;
  combo: string;
  watermark: string;
  background: boolean;
  fps: number;
  hardwareAccel: boolean;
  hevc: boolean;
  bitrateControl: string;
  bitrate: string;
  targetAudio: number;

  aggressive: boolean;
  challengeColor: string;
  challengeRank: number;
  disableEffect: boolean;
  doubleHint: boolean;
  fxaa: boolean;
  noteScale: number;
  particle: boolean;
  playerAvatar: string | null;
  playerName: string;
  playerRks: number;
  sampleCount: number;
  resPackPath: string | null;
  speed: number;
  volumeMusic: number;
  volumeSfx: number;
  handSplit: boolean;
  noteSpeedFactor: number;

  uiScore: boolean;
  uiName: boolean;
  uiLine: boolean;
  uiLevel: boolean;
  uiCombo: boolean;
  uiPb: boolean;
  uiPause: boolean;

  ffmpegThread: boolean;
}

export interface RPEChart {
  name: string;
  id: string;
  path: string;
  illustration: string;
  charter: string;
}

export interface BatchChart {
  path: string;
  name: string;
  level: string;
  charter: string;
  preset: string;
  status: 'pending' | 'rendering' | 'done' | 'failed';
  error?: string;
}

export interface FileDropEvent {
  paths: string[];
  position: { x: number; y: number };
}