<script lang="ts">
  export let state: string = 'idle';
  export let subState: string | null = null;

  interface StateStyle {
    label: string;
    color: string;
    bg: string;
    border: string;
  }

  const STATE_CONFIG: Record<string, StateStyle> = {
    idle:        { label: 'IDLE',     color: '#e0e0e0',  bg: 'linear-gradient(135deg, rgba(20,28,30,0.85) 0%, rgba(60,72,76,0.70) 100%)', border: 'rgba(255,255,255,0.18)' },
    running:     { label: 'RUNNING',  color: '#1a2000',  bg: 'linear-gradient(135deg, #4a5c00 0%, #8cb800 50%, #b8e000 100%)',            border: '#7aa000' },
    stage_1:     { label: 'STAGE 1',  color: '#1a2000',  bg: 'linear-gradient(135deg, #3e4e00 0%, #7aaa00 50%, #a8d000 100%)',            border: '#6a9400' },
    stage_2:     { label: 'STAGE 2',  color: '#ffffff',  bg: 'linear-gradient(135deg, #0d4039 0%, #1d8070 50%, #2cbca4 100%)',            border: '#1d9b87' },
    stage_3:     { label: 'STAGE 3',  color: '#ffffff',  bg: 'linear-gradient(135deg, #5a0035 0%, #b80064 50%, #fe0288 100%)',            border: '#c8006a' },
    stage_4:     { label: 'STAGE 4',  color: '#1a2000',  bg: 'linear-gradient(135deg, #4a5c00 0%, #8cb800 50%, #b8e000 100%)',            border: '#7aa000' },
    paused:      { label: 'PAUSED',   color: '#ffffff',  bg: 'linear-gradient(135deg, #0d4039 0%, #1d8070 50%, #2cbca4 100%)',            border: '#1d9b87' },
    stopping:    { label: 'STOPPING', color: '#ffffff',  bg: 'linear-gradient(135deg, #5c2000 0%, #bb4800 50%, #fe6600 100%)',            border: '#cc5200' },
    error:       { label: 'ERROR',    color: '#ffffff',  bg: 'linear-gradient(135deg, #5a0035 0%, #b80064 50%, #fe0288 100%)',            border: '#c8006a' },
    test_inputs: { label: 'TEST',     color: '#ffffff',  bg: 'linear-gradient(135deg, #0d4039 0%, #1d8070 50%, #2cbca4 100%)',            border: '#1d9b87' },
  };

  $: cfg = STATE_CONFIG[state] ?? { label: state.toUpperCase(), color: '#4a4a4a', bg: 'rgba(0,0,0,0.08)', border: 'rgba(0,0,0,0.2)' };
  $: pulse = state === 'running' || state.startsWith('stage_');
</script>

<div
  class="pill"
  class:pulse
  style="background: {cfg.bg}; border-color: {cfg.border}; color: {cfg.color};"
>
  <span class="dot" class:pulse style="background: {cfg.color}; opacity: 0.7;"></span>
  <div class="pill-text">
    <span class="label">{cfg.label}</span>
    {#if subState}
      <span class="sub" style="color: {cfg.color}; opacity: 0.65;">{subState}</span>
    {/if}
  </div>
</div>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px 4px 8px;
    border: 1.5px solid;
    clip-path: polygon(8px 0%, 100% 0%, calc(100% - 8px) 100%, 0% 100%);
    transition: box-shadow 0.3s, transform 0.2s;
    min-width: 0;
  }

  .pill.pulse {
    box-shadow: 0 2px 16px rgba(0,0,0,0.12);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
    transition: opacity 0.3s;
  }

  .dot.pulse {
    animation: dotpulse 1.4s ease-in-out infinite;
  }

  @keyframes dotpulse {
    0%, 100% { opacity: 0.7; }
    50%       { opacity: 0.2; }
  }

  .pill-text {
    display: flex;
    flex-direction: column;
    gap: 0;
    min-width: 0;
  }

  .label {
    font-family: var(--font-display);
    font-size: 11px;
    font-weight: 900;
    letter-spacing: 0.10em;
    text-transform: uppercase;
    line-height: 1;
    color: rgba(255, 255, 255, 0.92);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sub {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 400;
    letter-spacing: 0.02em;
    line-height: 1;
    margin-top: 2px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 80px;
  }
</style>
