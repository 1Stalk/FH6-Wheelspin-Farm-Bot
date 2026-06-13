<script lang="ts">
  /**
   * / — Main Route.
   * Connects to WebSocket and Tauri API in real-time.
   */
  import "../app.css";
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
  import type { LogEntry, BotConfig } from "$lib/mock";
  import tauriConfig from "../../src-tauri/tauri.conf.json";

  import BotHeader from "$lib/design/BotHeader.svelte";
  import StageMap from "$lib/design/StageMap.svelte";
  import ControlSidebar from "$lib/design/ControlSidebar.svelte";
  import LogConsole from "$lib/design/LogConsole.svelte";
  import SettingsModal from "$lib/design/SettingsModal.svelte";
  import WelcomeModal from "$lib/design/WelcomeModal.svelte";
  import ChooseCarModal from "$lib/design/ChooseCarModal.svelte";
import StartNoticeModal from "$lib/design/StartNoticeModal.svelte";

  // --- WebSocket state & Tauri connection ------------------------------------
  const MAX_LOGS = 500;
  let connected: boolean = false;
  let unlistenFunctions: (() => void)[] = [];
  let showWelcome: boolean = false;
  let showStartNotice: boolean = false;

  // --- Bot state -------------------------------------------------------------
  let botState: string = "idle";
  let subState: string | null = null;
  let cycle: number = 0;
  let sessionSp: number = 0;

  // Credits and wheelspins remain as stubs/hardcode per user request
  let sessionCr: number = 0;
  let sessionWspins: number = 0;

  let config: BotConfig = {
    stages_enabled: { stage1: true, stage2: true, stage3: true, stage4: true },
    loop_count: 0,
    credits_budget: 0,
    stage1_duration: 6,
    stage2_iterations: 2,
    stage3_iterations: 2,
    stage4_iterations: 2,
    smart_settings: true,
    stage1_car: "toyota_tacoma_fe",
    stage2_car: "subaru_impreza_22b",
  };
  let logs: LogEntry[] = [];

  // --- UI & Sandbox controls ------------------------------------------------
  let isSandbox: boolean = false;
  let showSettings: boolean = false;
  let showChooseCar: "stage1" | "stage2" | null = null;
  let selectedCarId: string = "toyota_tacoma_fe";
  let selectedCarIdS2: string = "subaru_impreza_22b";

  $: controlsDisabled = botState !== "idle";

  $: stagesEnabled = config.stages_enabled;
  $: loopCount = config.loop_count;
  $: creditsBudget = config.credits_budget;
  $: stage1Duration = config.stage1_duration;
  $: stage2Iterations = config.stage2_iterations;
  $: stage3Iterations = config.stage3_iterations;
  $: stage4Iterations = config.stage4_iterations;

  // --- Tauri Listeners ------------------------------------------------------
  async function setupTauriListeners() {
    connected = true;

    try {
      const statusUnlisten = await listen("status", (event) => {
        handleMessage(event.payload);
      });
      unlistenFunctions.push(statusUnlisten);

      const logUnlisten = await listen("log", (event) => {
        handleMessage(event.payload);
      });
      unlistenFunctions.push(logUnlisten);

      const errorUnlisten = await listen("error", (event) => {
        handleMessage(event.payload);
      });
      unlistenFunctions.push(errorUnlisten);

      const configUnlisten = await listen("config_saved", (event) => {
        handleMessage(event.payload);
      });
      unlistenFunctions.push(configUnlisten);

      // Get initial bot status
      const initialStatus = await invoke<any>("get_bot_status");
      handleMessage(initialStatus);
    } catch (err) {
      console.error("Failed to setup Tauri event listeners:", err);
    }
  }

  function handleMessage(msg: any) {
    if (msg.type === "status") {
      botState = msg.state;
      subState = msg.sub_state ?? null;
      cycle = msg.cycle ?? 0;
      sessionSp = msg.session_sp ?? 0;
      sessionCr = msg.session_cr ?? 0;
      sessionWspins = msg.session_wspins ?? 0;
      if (msg.config) {
        config = msg.config;
        if (config.stage1_car) {
          selectedCarId = config.stage1_car;
        }
        if (config.stage2_car) {
          selectedCarIdS2 = config.stage2_car;
        }
      }
    } else if (msg.type === "log") {
      addLog(msg.level, msg.message, msg.ts);
    } else if (msg.type === "error") {
      addLog("error", `[${msg.stage}] ${msg.message}`);
    } else if (msg.type === "config_saved") {
      if (msg.config) {
        config = msg.config;
        if (config.stage1_car) {
          selectedCarId = config.stage1_car;
        }
        if (config.stage2_car) {
          selectedCarIdS2 = config.stage2_car;
        }
      }
      addLog("info", "Config saved.");
    }
  }

  function addLog(level: string, message: string, ts: string | null = null) {
    const now = ts ?? new Date().toTimeString().slice(0, 8);
    logs = [...logs.slice(-MAX_LOGS + 1), { level, message, ts: now }];
  }

  // --- Handlers ──────────────────────────────────────────────────────────────
  function onCmd(e: CustomEvent<{ cmd: string }>) {
    const { cmd } = e.detail;
    if (cmd === "start") {
      const hasStartedBefore = localStorage.getItem("fh6_start_clicked_before") === "true";
      if (!hasStartedBefore) {
        showStartNotice = true;
      } else {
        invoke("start_bot");
      }
    } else if (cmd === "stop") {
      invoke("stop_bot");
    } else if (cmd === "pause") {
      invoke("pause_bot");
    } else if (cmd === "resume") {
      invoke("resume_bot");
    }
  }

  function onToggle(e: CustomEvent<{ key: "stage1" | "stage2" | "stage3" | "stage4" }>) {
    const { key } = e.detail;
    config.stages_enabled[key] = !config.stages_enabled[key];
    config.stages_enabled = { ...config.stages_enabled };
    
    // Resolve any conflicts that may occur due to stage toggling
    config = resolveConfigConflicts(config, false, false, false, false);
    
    invoke("update_config", { newConfig: config });
  }

  // --- Conflict resolution helper for Smart Settings -------------------------
  function resolveConfigConflicts(
    cfg: BotConfig,
    s1Changed = false,
    s3Changed = false,
    s2Changed = false,
    s4Changed = false
  ): BotConfig {
    if (!cfg.smart_settings) return cfg;

    const newCfg = { ...cfg };
    newCfg.stages_enabled = { ...cfg.stages_enabled };

    let s1ChangedLocal = s1Changed;
    let s3ChangedLocal = s3Changed;
    let s2ChangedLocal = s2Changed;
    let s4ChangedLocal = s4Changed;

    // S2 estimated SP limit (99 SP check)
    if (newCfg.stages_enabled.stage2) {
      const maxS2 = 99;
      if (newCfg.stage2_iterations > maxS2) {
        newCfg.stage2_iterations = maxS2;
        s2ChangedLocal = true;
      }
    }

    // 00. S1 + S2 + S3 + S4 Quad-link resolution
    if (newCfg.stages_enabled.stage1 && newCfg.stages_enabled.stage2 && newCfg.stages_enabled.stage3 && newCfg.stages_enabled.stage4) {
      const maxS2 = 99;
      const maxS4 = 33;
      const maxS3 = 33;

      if (s1ChangedLocal) {
        const laps = Math.floor(newCfg.stage1_duration / 6);
        const maxS3FromS1 = Math.floor((laps * 160000) / 86000);
        let stage4 = Math.min(maxS4, maxS3FromS1);
        if (stage4 < 1) {
          stage4 = 1;
          newCfg.stage1_duration = Math.max(newCfg.stage1_duration, 6);
        }
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage2_iterations = stage4 * 3;
        s4ChangedLocal = true;
        s3ChangedLocal = true;
        s2ChangedLocal = true;
      } else if (s2ChangedLocal) {
        if (newCfg.stage2_iterations > maxS2) {
          newCfg.stage2_iterations = maxS2;
        }
        let stage4 = Math.floor(newCfg.stage2_iterations / 3);
        if (stage4 < 1) {
          stage4 = 1;
          newCfg.stage2_iterations = 3;
        }
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage1_duration = Math.ceil((stage4 * 86000) / 160000) * 6;
        s4ChangedLocal = true;
        s3ChangedLocal = true;
        s1ChangedLocal = true;
      } else if (s3ChangedLocal) {
        if (newCfg.stage3_iterations > maxS3) {
          newCfg.stage3_iterations = maxS3;
        }
        const stage4 = Math.max(1, newCfg.stage3_iterations);
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage2_iterations = stage4 * 3;
        newCfg.stage1_duration = Math.ceil((stage4 * 86000) / 160000) * 6;
        s4ChangedLocal = true;
        s2ChangedLocal = true;
        s1ChangedLocal = true;
      } else if (s4ChangedLocal) {
        if (newCfg.stage4_iterations > maxS4) {
          newCfg.stage4_iterations = maxS4;
        }
        const stage4 = Math.max(1, newCfg.stage4_iterations);
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage2_iterations = stage4 * 3;
        newCfg.stage1_duration = Math.ceil((stage4 * 86000) / 160000) * 6;
        s3ChangedLocal = true;
        s2ChangedLocal = true;
        s1ChangedLocal = true;
      } else {
        const stage4 = Math.min(maxS4, Math.max(1, newCfg.stage4_iterations));
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage2_iterations = Math.max(newCfg.stage2_iterations, stage4 * 3);
        const requiredS1 = Math.ceil((stage4 * 86000) / 160000) * 6;
        newCfg.stage1_duration = Math.max(newCfg.stage1_duration, requiredS1);
      }
    } else if (newCfg.stages_enabled.stage2 && newCfg.stages_enabled.stage3 && newCfg.stages_enabled.stage4) {
      const maxS2 = 99;
      const maxS4 = 33;
      const maxS3 = 33;

      if (s2ChangedLocal) {
        if (newCfg.stage2_iterations > maxS2) {
          newCfg.stage2_iterations = maxS2;
        }
        let stage4 = Math.floor(newCfg.stage2_iterations / 3);
        if (stage4 < 1) {
          stage4 = 1;
          newCfg.stage2_iterations = 3;
        }
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        s4ChangedLocal = true;
        s3ChangedLocal = true;
      } else if (s3ChangedLocal) {
        if (newCfg.stage3_iterations > maxS3) {
          newCfg.stage3_iterations = maxS3;
        }
        const stage4 = Math.max(1, newCfg.stage3_iterations);
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage2_iterations = stage4 * 3;
        s4ChangedLocal = true;
        s2ChangedLocal = true;
      } else if (s4ChangedLocal) {
        if (newCfg.stage4_iterations > maxS4) {
          newCfg.stage4_iterations = maxS4;
        }
        const stage4 = Math.max(1, newCfg.stage4_iterations);
        newCfg.stage4_iterations = stage4;
        newCfg.stage3_iterations = stage4;
        newCfg.stage2_iterations = stage4 * 3;
        s3ChangedLocal = true;
        s2ChangedLocal = true;
      } else {
        if (newCfg.stage4_iterations > maxS4) {
          newCfg.stage4_iterations = maxS4;
        }
        newCfg.stage3_iterations = newCfg.stage4_iterations;
        newCfg.stage2_iterations = Math.max(newCfg.stage2_iterations, newCfg.stage4_iterations * 3);
      }
    } else {
      // 1. S2 + S4 Conflict resolution (SP check with 99 SP limit for S2 / 33 limit for S4)
      if (newCfg.stages_enabled.stage2 && newCfg.stages_enabled.stage4) {
        const maxS2 = 99;
        const maxS4 = 33;

        if (s2ChangedLocal) {
          // User changed S2, adjust S4 down if it conflicts
          if (newCfg.stage2_iterations < newCfg.stage4_iterations * 3) {
            const maxAllowedS4 = Math.floor(newCfg.stage2_iterations / 3);
            newCfg.stage4_iterations = Math.max(1, maxAllowedS4);
            s4ChangedLocal = true;
          }
        } else if (s4ChangedLocal) {
          // User changed S4, cap it to maxS4 and adjust S2 up
          if (newCfg.stage4_iterations > maxS4) {
            newCfg.stage4_iterations = maxS4;
          }
          newCfg.stage2_iterations = newCfg.stage4_iterations * 3;
        } else {
          // General resolution / toggle: cap S4 and S2 to bounds, and ensure S2 >= S4 * 3
          if (newCfg.stage4_iterations > maxS4) {
            newCfg.stage4_iterations = maxS4;
          }
          newCfg.stage2_iterations = Math.min(newCfg.stage2_iterations, maxS2);
          newCfg.stage2_iterations = Math.max(newCfg.stage2_iterations, newCfg.stage4_iterations * 3);
        }
      }

      // 2. S3 + S4 Conflict resolution (S3 >= S4 check)
      if (newCfg.stages_enabled.stage3 && newCfg.stages_enabled.stage4) {
        if (newCfg.stage3_iterations < newCfg.stage4_iterations) {
          if (s3ChangedLocal) {
            // User decreased S3, so S4 must decrease to match S3
            newCfg.stage4_iterations = newCfg.stage3_iterations;
            s4ChangedLocal = true;
          } else {
            // User increased S4 (or other), so S3 must increase to match S4
            newCfg.stage3_iterations = newCfg.stage4_iterations;
            s3ChangedLocal = true;
          }
        }
      }
    }

    // 3. S1 + S3 Conflict resolution (Credits check)
    if (newCfg.stages_enabled.stage1 && newCfg.stages_enabled.stage3) {
      const laps = Math.floor(newCfg.stage1_duration / 6);
      const credits = laps * 160000;
      const spent = newCfg.stage3_iterations * 86000;
      if (credits < spent) {
        if (s1ChangedLocal) {
          // User changed S1 (decreased it), adjust S3 down
          const maxS3 = Math.floor((laps * 160000) / 86000);
          newCfg.stage3_iterations = Math.max(1, maxS3);
          
          // Ensure S3 >= S4 holds if S4 is enabled
          if (newCfg.stages_enabled.stage4 && newCfg.stage3_iterations < newCfg.stage4_iterations) {
            newCfg.stage3_iterations = newCfg.stage4_iterations;
            // Since S3 must be at least S4, we must adjust S1 back up!
            const requiredLaps = Math.ceil((newCfg.stage3_iterations * 86000) / 160000);
            newCfg.stage1_duration = requiredLaps * 6;
          }
        } else {
          // User changed S3 (increased it) or toggle occurred: adjust S1 up
          const requiredLaps = Math.ceil((newCfg.stage3_iterations * 86000) / 160000);
          newCfg.stage1_duration = requiredLaps * 6;
        }
      }
    }

    return newCfg;
  }

  // --- Config update handler --------------------------------------------------
  function onConfig(e: CustomEvent<{
    stagesEnabled: BotConfig['stages_enabled'];
    loopCount: number;
    creditsBudget: number;
    stage1Duration: number;
    stage2Iterations: number;
    stage3Iterations: number;
    stage4Iterations: number;
  }>) {
    const {
      stagesEnabled: newStagesEnabled,
      loopCount: newLoopCount,
      creditsBudget: newCreditsBudget,
      stage1Duration: newStage1Duration,
      stage2Iterations: newStage2Iterations,
      stage3Iterations: newStage3Iterations,
      stage4Iterations: newStage4Iterations,
    } = e.detail;

    // Detect which field changed
    const s1Changed = newStage1Duration !== config.stage1_duration;
    const s3Changed = newStage3Iterations !== config.stage3_iterations;
    const s2Changed = newStage2Iterations !== config.stage2_iterations;
    const s4Changed = newStage4Iterations !== config.stage4_iterations;

    let newConfig: BotConfig = {
      ...config,
      stages_enabled: newStagesEnabled,
      loop_count: newLoopCount,
      credits_budget: newCreditsBudget,
      stage1_duration: newStage1Duration,
      stage2_iterations: newStage2Iterations,
      stage3_iterations: newStage3Iterations,
      stage4_iterations: newStage4Iterations,
    };

    const resolvedConfig = resolveConfigConflicts(newConfig, s1Changed, s3Changed, s2Changed, s4Changed);

    if (JSON.stringify(resolvedConfig) !== JSON.stringify(newConfig)) {
      // Step 1: Temporarily accept user change so Svelte registers the state change
      config = newConfig;
      // Step 2: Revert to resolved config on the next tick so Svelte updates the DOM
      setTimeout(() => {
        config = resolvedConfig;
        invoke("update_config", { newConfig: config });
      }, 0);
    } else {
      config = newConfig;
      invoke("update_config", { newConfig: config });
    }
  }

  function onRunNav(e: CustomEvent<{ target: string }>) {
    invoke("run_nav_test", { target: e.detail.target });
  }

  function onTestGamepad() {
    invoke("run_gamepad_test");
  }

  function onTestCV() {
    invoke("run_cv_diagnostics");
  }

  // ── Dev controls (sandbox only — state switcher) ─────────────
  const STATES = [
    "idle",
    "running",
    "stage_1",
    "stage_2",
    "stage_3",
    "stage_4",
    "paused",
    "error",
  ];
  let stateIdx = STATES.indexOf(botState);

  function cycleState() {
    stateIdx = (stateIdx + 1) % STATES.length;
    botState = STATES[stateIdx];
    addLog("debug", `[DEV] State changed to: ${botState}`);
  }

  function toggleConnection() {
    connected = !connected;
    addLog(
      connected ? "info" : "warn",
      connected
        ? "Connected to bot server"
        : "Disconnected from bot server. Reconnecting...",
    );
  }

  let collapsed: boolean = false;
  let unlistenPromise: Promise<any> | null = null;

  async function handleToggleCollapse(e: CustomEvent<{ collapsed: boolean }>) {
    const isCollapsed = e.detail.collapsed;
    try {
      if ((window as any).__TAURI_INTERNALS__) {
        const appWindow = getCurrentWindow();
        const minHeight = tauriConfig?.app?.windows?.[0]?.minHeight ?? 630;
        const height = isCollapsed ? minHeight : 800;
        await appWindow.setSize(new LogicalSize(600, height));
      }
    } catch (err) {
      console.warn("Could not resize window:", err);
    }
  }

  function onSelectCar(e: CustomEvent<{ carId: string }>) {
    const carId = e.detail.carId;
    if (showChooseCar === "stage1") {
      selectedCarId = carId;
      localStorage.setItem("stage1_selected_car", selectedCarId);
      
      config.stage1_car = selectedCarId;
      config = { ...config };
      invoke("update_config", { newConfig: config });

      const carDisplay = selectedCarId.replace(/_/g, " ").toUpperCase();
      addLog("info", `Selected car for Stage 1: ${carDisplay}`);
    } else if (showChooseCar === "stage2") {
      selectedCarIdS2 = carId;
      localStorage.setItem("stage2_selected_car", selectedCarIdS2);
      
      config.stage2_car = selectedCarIdS2;
      config = { ...config };
      invoke("update_config", { newConfig: config });

      const carDisplay = selectedCarIdS2.replace(/_/g, " ").toUpperCase();
      addLog("info", `Selected car for Stage 2: ${carDisplay}`);
    }
  }

  // --- Lifecycle -------------------------------------------------------------
  onMount(async () => {
    isSandbox = window.location.pathname.includes("/design");
    setupTauriListeners();

    const storedCar = localStorage.getItem("stage1_selected_car");
    selectedCarId = (storedCar === "toyota_tacoma_fe" || storedCar === "nissan_scargo_fe") ? storedCar : "toyota_tacoma_fe";

    const storedCarS2 = localStorage.getItem("stage2_selected_car");
    selectedCarIdS2 = (storedCarS2 === "subaru_impreza_22b") ? storedCarS2 : "subaru_impreza_22b";

    if (!isSandbox) {
      const firstLaunchDone = localStorage.getItem("fh6_first_launch_done") === "true";
      if (!firstLaunchDone) {
        showWelcome = true;
      } else {
        try {
          const hasVigem = await invoke<boolean>("check_vigem_status");
          if (!hasVigem) {
            showWelcome = true;
          }
        } catch (err) {
          console.error("Failed to check ViGEmBus status on mount:", err);
        }
      }
    }

    try {
      if ((window as any).__TAURI_INTERNALS__) {
        const appWindow = getCurrentWindow();
        unlistenPromise = appWindow.onResized(async (event) => {
          try {
            const factor = await appWindow.scaleFactor();
            const logicalSize = event.payload.toLogical(factor);
            const minHeight = tauriConfig?.app?.windows?.[0]?.minHeight ?? 630;
            if (logicalSize.height <= minHeight + 5) {
              collapsed = true;
            } else {
              collapsed = false;
            }
          } catch (err) {
            console.warn("Error in window resize handler:", err);
          }
        });
      }
    } catch (err) {
      console.warn("Could not setup window resize listener:", err);
    }
  });

  onDestroy(async () => {
    for (const unlisten of unlistenFunctions) {
      unlisten();
    }
    if (unlistenPromise) {
      const unlisten = await unlistenPromise;
      unlisten();
    }
  });
  function onSaveAdvancedSettings(e: CustomEvent<{ smartSettings: boolean }>) {
    config.smart_settings = e.detail.smartSettings;
    
    // Automatically resolve conflicts if smart settings were enabled
    config = resolveConfigConflicts(config, false, false, false, false);
    
    invoke("update_config", { newConfig: config });
  }
</script>

<div class="design-shell">
  <!-- Dev toolbar (design only, will not appear in production UI) -->
  {#if isSandbox}
    <div class="dev-toolbar">
      <span class="dev-badge">⚙ DESIGN SANDBOX</span>
      <span class="dev-hint">Buttons below change mock state (overrides live WS)</span>
      <button class="dev-btn" on:click={cycleState}>
        State: <strong>{botState}</strong> → next
      </button>
      <button class="dev-btn" on:click={toggleConnection}>
        WS: <strong>{connected ? "online" : "offline"}</strong>
      </button>
      <button
        class="dev-btn"
        on:click={() => {
          cycle++;
          sessionSp += 180;
          sessionCr += 160000;
          sessionWspins += 6;
          addLog("info", `Cycle ${cycle} complete. SP +180, CR +160K, Wheelspins +6`);
        }}
      >
        +Cycle
      </button>
    </div>
  {/if}

  <!-- ── Actual UI layout ─────────────────────────────────────────────────── -->
  <div class="app-shell">
    <!-- Header -->
    <BotHeader
      state={botState}
      {subState}
      {cycle}
      {sessionSp}
      {sessionCr}
      {sessionWspins}
      {connected}
      on:settings={() => (showSettings = true)}
    />

    <!-- Stage Map (center block) -->
    <div class="map-area">
      <StageMap
        {stagesEnabled}
        {botState}
        {subState}
        disabled={controlsDisabled}
        on:toggle={onToggle}
        on:choose_car={(e) => (showChooseCar = e.detail.stage)}
      />
    </div>

    <!-- Bottom row: Sidebar + Log -->
    <div class="bottom-row">
      <ControlSidebar
        state={botState}
        {stagesEnabled}
        {loopCount}
        {creditsBudget}
        {stage1Duration}
        {stage2Iterations}
        {stage3Iterations}
        {stage4Iterations}
        on:cmd={onCmd}
        on:config={onConfig}
      />

      <LogConsole bind:logs bind:collapsed on:toggleCollapse={handleToggleCollapse} />
    </div>

    <!-- Footer -->
    <div class="app-footer">
      <span class="footer-left">FH6 WHEELSPIN FARM BOT</span>
      <span class="footer-right mono">v1.0.1 · FH6WheelspinFarmBot</span>
    </div>
  </div>

  {#if showSettings}
    <SettingsModal
      {connected}
      smartSettings={config.smart_settings}
      on:close={() => (showSettings = false)}
      on:save={onSaveAdvancedSettings}
      on:run_nav={onRunNav}
      on:test_gamepad={onTestGamepad}
      on:test_cv={onTestCV}
      on:open_setup={() => {
        showSettings = false;
        showWelcome = true;
      }}
    />
  {/if}

  {#if showWelcome}
    <WelcomeModal
      on:close={() => {
        showWelcome = false;
        localStorage.setItem("fh6_first_launch_done", "true");
      }}
    />
  {/if}

  {#if showStartNotice}
    <StartNoticeModal
      on:close={() => {
        showStartNotice = false;
        localStorage.setItem("fh6_start_clicked_before", "true");
        invoke("start_bot");
      }}
    />
  {/if}

  {#if showChooseCar}
    <ChooseCarModal
      stage={showChooseCar}
      selectedCarId={showChooseCar === "stage1" ? selectedCarId : selectedCarIdS2}
      on:close={() => (showChooseCar = null)}
      on:select={onSelectCar}
    />
  {/if}
</div>

<style>
  /* ── Dev toolbar ──────────────────────────────────── */
  .design-shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .dev-toolbar {
    display: flex;
    align-items: center;
    gap: var(--gap-sm);
    padding: 3px var(--gap-lg);
    background: #f0f0f0;
    border-bottom: 1.5px solid rgba(0, 0, 0, 0.12);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .dev-badge {
    font-size: 8px;
    font-weight: 900;
    letter-spacing: 0.18em;
    color: #1a2000;
    background: var(--fh6-lime);
    border: 1px solid var(--fh6-lime-dim);
    padding: 2px 8px;
    clip-path: polygon(5px 0%, 100% 0%, calc(100% - 5px) 100%, 0% 100%);
    flex-shrink: 0;
  }

  .dev-hint {
    font-size: 9px;
    color: var(--text-muted);
    flex: 1;
    font-family: var(--font-mono);
  }

  .dev-btn {
    font-size: 9px;
    font-weight: 700;
    padding: 2px 10px;
    background: white;
    border: 1px solid var(--border-mid);
    color: var(--text-secondary);
    clip-path: polygon(5px 0%, 100% 0%, calc(100% - 5px) 100%, 0% 100%);
    cursor: pointer;
    transition:
      background 0.15s,
      color 0.15s,
      border-color 0.15s;
    flex-shrink: 0;
    letter-spacing: 0.06em;
  }
  .dev-btn:hover {
    background: var(--fh6-lime);
    color: #1a2000;
    border-color: var(--fh6-lime-dim);
  }
  .dev-btn strong {
    color: var(--fh6-lime-dim);
  }

  /* ── App shell layout ─────────────────────────────────────── */
  .app-shell {
    flex: 1;
    display: grid;
    grid-template-rows: 64px auto 1fr 22px;
    grid-template-columns: 1fr;
    height: 100%;
    overflow: hidden;
    background: var(--bg-root);
    position: relative;
  }

  /* FH6 diagonal background pattern (light) */
  .app-shell::before {
    content: "";
    position: absolute;
    inset: 0;
    background-image: repeating-linear-gradient(
        -8deg,
        rgba(0, 0, 0, 0.028) 0px,
        rgba(0, 0, 0, 0.028) 1px,
        transparent 1px,
        transparent 56px
      ),
      repeating-linear-gradient(
        -8deg,
        rgba(0, 0, 0, 0.014) 0px,
        rgba(0, 0, 0, 0.014) 1px,
        transparent 1px,
        transparent 28px
      );
    pointer-events: none;
    z-index: 0;
  }

  /* Map area */
  .map-area {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    border-bottom: 1.5px solid var(--border-mid);
    overflow: hidden;
    background: var(--bg-root);
  }

  /* Bottom row */
  .bottom-row {
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-rows: auto 1fr;
    overflow: hidden;
    height: 100%;
    /* Console header is always visible (~32px header) */
    min-height: 80px;
  }

  /* Footer */
  .app-footer {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--gap-lg);
    height: 22px;
    background: #1d2a2c;
    border-top: none;
    flex-shrink: 0;
    overflow: hidden;
  }

  /* Lime accent left */
  .app-footer::before {
    content: "";
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--fh6-lime);
  }

  .footer-left {
    font-size: 8px;
    font-weight: 900;
    letter-spacing: 0.22em;
    color: var(--fh6-lime);
    text-transform: uppercase;
    padding-left: 8px;
  }

  .footer-right {
    font-size: 9px;
    color: rgba(255, 255, 255, 0.5);
    letter-spacing: 0.06em;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
