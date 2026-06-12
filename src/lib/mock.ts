/**
 * mock.ts — mock data for design sandbox (/design).
 * WebSocket is not needed — everything is static here.
 */

export interface LogEntry {
  level: string;
  message: string;
  ts: string;
}

export interface BotConfig {
  stages_enabled: {
    stage1: boolean;
    stage2: boolean;
    stage3: boolean;
    stage4: boolean;
  };
  loop_count: number;
  credits_budget: number;
  stage1_duration: number;
  stage2_iterations: number;
  stage3_iterations: number;
  stage4_iterations: number;
  smart_settings: boolean;
  stage1_car: string;
  stage2_car: string;
}

export interface StageMeta {
  key: 'stage1' | 'stage2' | 'stage3' | 'stage4';
  id: number;
  label: string;
  sublabel: string;
  description: string;
  color: string;
  icon: string;
}

export interface MockState {
  botState: string;
  subState: string;
  cycle: number;
  session_sp: number;
  session_cr: number;
  session_wspins: number;
  connected: boolean;
  config: BotConfig;
  logs: LogEntry[];
}

export const mockState: MockState = {
  botState: 'running',  // 'idle' | 'running' | 'paused' | 'error'
  subState: 'Driving to finish...',
  cycle: 3,
  session_sp: 420,
  session_cr: 480000,
  session_wspins: 7,
  connected: true,
  config: {
    stages_enabled: { stage1: true, stage2: true, stage3: false, stage4: true },
    loop_count: 0,
    credits_budget: 50000,
    stage1_duration: 6,
    stage2_iterations: 2,
    stage3_iterations: 2,
    stage4_iterations: 2,
    smart_settings: true,
    stage1_car: 'toyota_tacoma_fe',
    stage2_car: 'subaru_impreza_22b',
  },
  logs: [
    { level: 'info', message: 'Connected to bot server', ts: '10:42:01' },
    { level: 'info', message: 'Stage 1 started — Colossus Race', ts: '10:42:03' },
    { level: 'info', message: 'Macro: Pressing RT (peak=1.0, hold=27.0s)', ts: '10:42:05' },
    { level: 'info', message: 'Macro: Waiting 1.054s', ts: '10:42:33' },
    { level: 'warn', message: 'Stage 2: FPS drop detected, retrying...', ts: '10:42:45' },
    { level: 'info', message: 'Stage 2 complete. SP +180', ts: '10:43:01' },
    { level: 'info', message: 'Stage 3: DISABLED, skipping', ts: '10:43:02' },
    { level: 'info', message: 'Stage 4 started — Wheelspin', ts: '10:43:03' },
    { level: 'info', message: 'Cycle 3 complete. Session SP: 420', ts: '10:44:10' },
    { level: 'error', message: 'Template not found: autoshow_main.png', ts: '10:44:12' },
    { level: 'info', message: 'Retrying via neutral_position reset...', ts: '10:44:13' },
    { level: 'info', message: 'Interface reset complete. Resuming.', ts: '10:44:18' },
  ],
};

export const STAGE_META: StageMeta[] = [
  {
    key: 'stage1',
    id: 1,
    label: 'COLOSSUS RACE',
    sublabel: 'Earns Credits',
    description: 'Drives the Horizon Colossus circuit autonomously to farm credits.',
    color: '#00FF66',
    icon: '🏎',
  },
  {
    key: 'stage2',
    id: 2,
    label: 'FARM SKILL POINTS',
    sublabel: 'Earns SP',
    description: 'Performs skill chains to accumulate Skill Points for Wheelspin.',
    color: '#00CFFF',
    icon: '⚡',
  },
  {
    key: 'stage4',
    id: 4,
    label: 'GET SUPER WHEELSPINS',
    sublabel: 'Spends SP + Garage Cleanup',
    description: 'Redeems accumulated Skill Points in Wheelspin and clears garage.',
    color: '#CCFF00',
    icon: '🎰',
  },
  {
    key: 'stage3',
    id: 3,
    label: 'BUY CARS IN JOURNAL',
    sublabel: 'Spends Credits',
    description: 'Navigates Autoshow and purchases cheapest available cars.',
    color: '#FF007F',
    icon: '🛒',
  },
];
