<script lang="ts">
  import { afterUpdate, createEventDispatcher } from 'svelte';
  import type { LogEntry } from '$lib/mock';
  
  export let logs: LogEntry[] = [];
  export let collapsed: boolean = false;

  const dispatch = createEventDispatcher<{
    toggleCollapse: { collapsed: boolean };
  }>();

  let container: HTMLDivElement | undefined;
  let autoScroll: boolean = true;
  let copied: boolean = false;

  afterUpdate(() => {
    if (autoScroll && container) container.scrollTop = container.scrollHeight;
  });

  function onScroll() {
    if (!container) return;
    const atBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 20;
    autoScroll = atBottom;
  }

  interface LevelStyle {
    tag: string;
    color: string;
    bg: string;
  }

  const LEVEL_META: Record<string, LevelStyle> = {
    info:  { tag: 'INFO', color: '#a8d400',          bg: 'rgba(168,212,0,0.07)'   },
    warn:  { tag: 'WARN', color: '#2cbca4',           bg: 'rgba(44,188,164,0.08)'  },
    error: { tag: 'ERR',  color: '#fe0288',           bg: 'rgba(254,2,136,0.07)'   },
    debug: { tag: 'DBG',  color: 'rgba(255,255,255,0.28)', bg: 'transparent'       },
  };

  $: levelOf = (l: string) => LEVEL_META[l] ?? LEVEL_META.info;

  function toggleCollapse() {
    collapsed = !collapsed;
    dispatch('toggleCollapse', { collapsed });
  }
  function clearLogs() { logs = []; }

  async function copyLogs() {
    if (!logs.length) return;
    const text = logs.map(e => `[${e.ts}] ${e.level.toUpperCase().padEnd(5)} ${e.message}`).join('\n');
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => copied = false, 1800);
    } catch {
      // fallback: select all text in body
    }
  }
</script>

<div class="log-panel">
  <!-- Header -->
  <div class="log-header">
    <div class="log-header-left">
      <span class="terminal-prompt">&gt;_</span>
      <span class="log-title">CONSOLE OUTPUT</span>
      <span class="log-count">{logs.length} lines</span>
    </div>
    <div class="header-actions">
      <button class="action-btn collapse-btn" on:click={toggleCollapse}>
        <svg width="9" height="9" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="display:inline-block;vertical-align:middle;margin-right:3px">
          {#if collapsed}
            <polyline points="2,4 6,8 10,4"/>
          {:else}
            <polyline points="2,8 6,4 10,8"/>
          {/if}
        </svg>{collapsed ? 'EXPAND' : 'COLLAPSE'}
      </button>
      <button class="action-btn copy-btn" on:click={copyLogs} disabled={logs.length === 0}>
        {#if copied}
          ✓ COPIED
        {:else}
          <svg width="9" height="9" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5" style="display:inline-block;vertical-align:middle;margin-right:3px"><rect x="4" y="1" width="7" height="8" rx="1"/><rect x="1" y="3" width="7" height="8" rx="1" fill="currentColor" stroke="none" opacity="0.25"/><rect x="1" y="3" width="7" height="8" rx="1"/></svg>COPY
        {/if}
      </button>
      <button class="action-btn clear-btn" on:click={clearLogs} disabled={logs.length === 0}>
        ✕ CLEAR
      </button>
    </div>
  </div>

  <!-- Body -->
  {#if !collapsed}
    <div class="log-body" bind:this={container} on:scroll={onScroll}>
      {#if logs.length === 0}
        <div class="empty-state">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
            <rect x="3" y="4" width="18" height="16" rx="1"/>
            <path d="M7 8h2M11 8h6M7 12h1M10 12h4M7 16h8"/>
          </svg>
          <span>Awaiting bot output...</span>
        </div>
      {:else}
        {#each logs as entry, i (i)}
          {@const meta = levelOf(entry.level)}
          <div class="log-line" style="--lc: {meta.color}; --lb: {meta.bg}">
            <span class="ts">{entry.ts}</span>
            <span class="tag" style="color: {meta.color}">{meta.tag}</span>
            <span class="msg">{entry.message}</span>
          </div>
        {/each}
      {/if}
    </div>

    {#if !autoScroll}
      <button class="scroll-btn" on:click={() => { autoScroll = true; if(container) container.scrollTop = container.scrollHeight; }}>
        ↓ SCROLL TO BOTTOM
      </button>
    {/if}
  {/if}
</div>

<style>
  /* ── Panel ─────────────────────────────────────────────── */
  .log-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: linear-gradient(160deg, #256370 0%, #297276 100%);
    position: relative;
    border-top: 1px solid rgba(255,255,255,0.06);
  }

  /* ── Header ─────────────────────────────────────────────── */
  .log-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 14px;
    border-bottom: 1px solid rgba(255,255,255,0.10);
    flex-shrink: 0;
    background: #1e4f59;
    position: relative;
  }

  /* Lime left bar */
  .log-header::before {
    content: '';
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 3px;
    background: var(--fh6-lime);
  }

  .log-header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
  }

  .terminal-prompt {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    color: var(--fh6-lime-dim);
    flex-shrink: 0;
  }

  .log-title {
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 0.20em;
    color: rgba(255,255,255,0.50);
    text-transform: uppercase;
  }

  .log-count {
    font-family: var(--font-mono);
    font-size: 9px;
    color: rgba(255,255,255,0.25);
    padding: 1px 6px;
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.10);
    border-radius: 2px;
  }

  /* ── Header action buttons ───────────────────────────────── */
  .header-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .action-btn {
    font-family: var(--font-mono);
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.12em;
    padding: 3px 10px;
    background: transparent;
    border: 1px solid rgba(255,255,255,0.14);
    color: rgba(255,255,255,0.40);
    clip-path: var(--clip-sm);
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s, background 0.15s;
    white-space: nowrap;
  }

  .action-btn:disabled {
    opacity: 0.30;
    cursor: not-allowed;
  }

  .copy-btn:not(:disabled):hover {
    border-color: var(--fh6-teal);
    color: var(--fh6-teal);
    background: rgba(44,188,164,0.10);
  }

  .collapse-btn:hover {
    border-color: var(--fh6-lime-dim);
    color: var(--fh6-lime);
    background: rgba(168,212,0,0.08);
  }

  .clear-btn:not(:disabled):hover {
    border-color: var(--fh6-pink);
    color: var(--fh6-pink);
    background: rgba(254,2,136,0.08);
  }

  /* ── Body ──────────────────────────────────────────────── */
  .log-body {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 4px 0;
    background: transparent;
    scrollbar-width: thin;
    scrollbar-color: rgba(255,255,255,0.15) transparent;
  }

  .log-body::-webkit-scrollbar { width: 4px; }
  .log-body::-webkit-scrollbar-track { background: transparent; }
  .log-body::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); border-radius: 2px; }

  /* ── Empty state ─────────────────────────────────────────── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 10px;
    color: rgba(255,255,255,0.18);
    font-size: 10px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    font-family: var(--font-mono);
  }

  /* ── Log line ────────────────────────────────────────────── */
  .log-line {
    display: flex;
    align-items: baseline;
    gap: 10px;
    padding: 2px 14px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    line-height: 1.7;
    border-left: 2px solid transparent;
    transition: background 0.06s;
    user-select: text;
  }

  .log-line:hover {
    background: var(--lb);
    border-left-color: var(--lc);
  }

  .ts {
    color: rgba(255,255,255,0.40);
    flex-shrink: 0;
    font-size: 9px;
    width: 56px;
  }

  .tag {
    font-size: 9px;
    font-weight: 700;
    letter-spacing: 0.14em;
    flex-shrink: 0;
    width: 30px;
  }

  .msg {
    color: rgba(255,255,255,0.88);
    word-break: break-word;
    flex: 1;
  }

  /* ── Scroll button ───────────────────────────────────────── */
  .scroll-btn {
    flex-shrink: 0;
    width: 100%;
    padding: 5px;
    font-size: 9px;
    font-weight: 800;
    letter-spacing: 0.14em;
    background: rgba(44,188,164,0.06);
    border: none;
    border-top: 1px solid rgba(44,188,164,0.30);
    color: var(--fh6-teal);
    cursor: pointer;
    transition: background 0.15s;
    font-family: var(--font-mono);
  }

  .scroll-btn:hover { background: rgba(44,188,164,0.12); }
</style>
