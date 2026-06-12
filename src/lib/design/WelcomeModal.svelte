<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  const dispatch = createEventDispatcher<{
    close: null;
  }>();

  let isChecking = true;
  let driverInstalled = false;
  let installStatus:
    | "idle"
    | "downloading"
    | "installing"
    | "success"
    | "error" = "idle";
  let errorMessage = "";
  let unlistenStatus: (() => void) | null = null;

  let step2Completed = false;
  let step3Completed = false;
  let step4Completed = false;

  let hoveredImage: string | null = null;
  let hoveredAlt = "";

  function handleImageHover(src: string, alt: string) {
    hoveredImage = src;
    hoveredAlt = alt;
  }

  function handleImageLeave() {
    hoveredImage = null;
  }

  function toggleStep2() {
    step2Completed = !step2Completed;
    localStorage.setItem("fh6_setup_step2_completed", String(step2Completed));
  }

  function toggleStep3() {
    step3Completed = !step3Completed;
    localStorage.setItem("fh6_setup_step3_completed", String(step3Completed));
  }

  function toggleStep4() {
    step4Completed = !step4Completed;
    localStorage.setItem("fh6_setup_step4_completed", String(step4Completed));
  }

  async function checkStatus() {
    isChecking = true;
    try {
      driverInstalled = await invoke<boolean>("check_vigem_status");
      if (driverInstalled) {
        installStatus = "success";
      } else if (installStatus === "success") {
        installStatus = "idle";
      }
    } catch (err) {
      console.error("Failed to check ViGEmBus status:", err);
      driverInstalled = false;
    } finally {
      isChecking = false;
    }
  }

  async function handleInstall() {
    if (installStatus === "downloading" || installStatus === "installing")
      return;
    installStatus = "downloading";
    errorMessage = "";

    try {
      await invoke("start_vigem_install");
    } catch (err: any) {
      installStatus = "error";
      errorMessage =
        err?.toString() || "Failed to start driver installation thread.";
    }
  }

  function handleClose() {
    dispatch("close");
  }

  onMount(async () => {
    await checkStatus();

    // Load persisted step checkmarks
    step2Completed =
      localStorage.getItem("fh6_setup_step2_completed") === "true";
    step3Completed =
      localStorage.getItem("fh6_setup_step3_completed") === "true";
    step4Completed =
      localStorage.getItem("fh6_setup_step4_completed") === "true";

    try {
      unlistenStatus = await listen<{ status: string; message: string }>(
        "vigem-install-status",
        (event) => {
          const payload = event.payload;
          installStatus = payload.status as any;
          if (payload.status === "success") {
            driverInstalled = true;
          } else if (payload.status === "error") {
            errorMessage =
              payload.message || "An error occurred during installation.";
          }
        },
      );
    } catch (err) {
      console.error("Failed to setup install status listener:", err);
    }
  });

  onDestroy(() => {
    if (unlistenStatus) {
      unlistenStatus();
    }
  });
</script>

<div
  role="button"
  aria-label="Welcome Modal"
  tabindex="-1"
  class="backdrop"
  transition:fade={{ duration: 180 }}
>
  <div class="modal" transition:scale={{ start: 0.96, duration: 180 }}>
    <!-- Header -->
    <div class="modal-header">
      <span class="modal-title">BOT INITIAL SETUP</span>
    </div>

    <!-- Body -->
    <div class="modal-body">
      <div class="welcome-intro">
        <p>
          To simulate gamepad controls in <strong>Forza Horizon 6</strong>, bot
          requires a virtual controller emulation driver named
          <strong>ViGEmBus</strong>.
        </p>
      </div>

      <!-- Connection Status indicator -->
      <div class="status-section">
        <span class="status-label">DRIVERS STATUS</span>
        <div class="status-row">
          <div class="driver-info">
            <span class="driver-name">ViGEmBus Virtual Driver</span>
            <span class="driver-desc"
              >Allows bot to emit virtual controller button presses.</span
            >
          </div>

          <div
            class="conn"
            class:online={driverInstalled}
            class:checking={isChecking}
          >
            <span class="conn-dot"></span>
            <div class="conn-text">
              <span class="conn-status">
                {#if isChecking}
                  CHECKING
                {:else if driverInstalled}
                  CONNECTED
                {:else}
                  DISCONNECTED
                {/if}
              </span>
              <span class="conn-label">VIGEMBUS DRIVER</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Installation Wizard cards -->
      <div class="setup-steps">
        <!-- Step 1 -->
        <div class="step-card" class:active={!driverInstalled}>
          <div class="step-header">
            <span class="step-num">STEP 1</span>
            <span class="step-title">INSTALL VIGEMBUS DRIVER</span>
          </div>

          {#if driverInstalled}
            <div class="step-success">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
              >
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
              <span>Driver is installed and ready to use!</span>
            </div>
          {:else}
            <p class="step-desc">
              Click the button below to download the latest ViGEmBus installer.
              An administrator prompt (UAC) will ask for permissions to register
              the system device.
            </p>

            <div class="action-row">
              <button
                class="install-btn"
                disabled={installStatus === "downloading" ||
                  installStatus === "installing"}
                on:click={handleInstall}
              >
                {#if installStatus === "downloading"}
                  <span class="spinner"></span> DOWNLOADING INSTALLER...
                {:else if installStatus === "installing"}
                  <span class="spinner"></span> RUNNING INSTALLATION...
                {:else}
                  🎮 INSTALL VIGEMBUS DRIVER
                {/if}
              </button>

              <button
                class="check-btn"
                on:click={checkStatus}
                disabled={isChecking}
              >
                {#if isChecking}Checking...{:else}🔄 Check again{/if}
              </button>
            </div>

            {#if installStatus === "error" && errorMessage}
              <div class="error-box">
                <span class="error-title">INSTALLATION FAILED</span>
                <span class="error-msg">{errorMessage}</span>
              </div>
            {/if}
          {/if}
        </div>

        <!-- Step 2 -->
        <div
          class="step-card"
          class:active={driverInstalled && !step2Completed}
          class:disabled={!driverInstalled}
        >
          <div class="step-header">
            <span class="step-num">STEP 2</span>
            <span class="step-title">ACQUIRE & TUNE DRIVING CAR</span>
            <label class="step-checkbox-wrapper">
              <input
                type="checkbox"
                checked={step2Completed}
                on:change={toggleStep2}
                disabled={!driverInstalled}
              />
              <span class="custom-checkbox"></span>
            </label>
          </div>

          <p class="step-desc">
            Obtain the <strong class="highlight-car"
              >1998 Subaru Impreza 22B-STi</strong
            >
            Apply Tune via Tuning Share Code:
            <span class="share-code-narrow">871 988 972</span>
            <br /><strong class="text-warn">IMPORTANT:</strong> Make sure to max
            out/unlock the <strong>Car Mastery</strong> perks for this vehicle to ensure optimal
            Skill Points farming speed.
          </p>

          <div class="step-images-grid">
            <div class="image-container">
              <img
                src="/assets/Subaru_22B_Tuned.png"
                alt="Subaru 22B Tuned"
                class="step-image"
                on:mouseenter={() =>
                  handleImageHover(
                    "/assets/Subaru_22B_Tuned.png",
                    "Subaru 22B Tuned",
                  )}
                on:mouseleave={handleImageLeave}
              />
              <div class="image-label">Tuning Guide</div>
            </div>
            <div class="image-container">
              <img
                src="/assets/Subaru_22B_Mastery.png"
                alt="Subaru 22B Car Mastery"
                class="step-image"
                on:mouseenter={() =>
                  handleImageHover(
                    "/assets/Subaru_22B_Mastery.png",
                    "Subaru 22B Car Mastery",
                  )}
                on:mouseleave={handleImageLeave}
              />
              <div class="image-label">Car Mastery Perks</div>
            </div>
          </div>
        </div>

        <!-- Step 3 -->
        <div
          class="step-card"
          class:active={driverInstalled && step2Completed && !step3Completed}
          class:disabled={!driverInstalled}
        >
          <div class="step-header">
            <span class="step-num">STEP 3</span>
            <span class="step-title">ACQUIRE & TUNE FARMING CAR</span>
            <label class="step-checkbox-wrapper">
              <input
                type="checkbox"
                checked={step3Completed}
                on:change={toggleStep3}
                disabled={!driverInstalled}
              />
              <span class="custom-checkbox"></span>
            </label>
          </div>

          <p class="step-desc">
            Obtain <strong>either</strong> of the following farm vehicles, apply its tuning,
            and <strong class="text-warn">max out all Car Mastery perks</strong>
            to ensure optimal Credits farming speed.
          </p>

          <div class="farm-options">
            <div class="farm-option-card">
              <div class="farm-option-header">
                <span class="option-badge option-a">Tacoma FE</span>
                <span class="car-name"
                  >2019 Toyota Tacoma TRD Pro <span class="fe-tag">FE</span
                  ></span
                >
              </div>
              <p class="step-desc opt-desc">
                Tuning Share Code: <span class="share-code-narrow"
                  >155 494 373</span
                >
              </p>
              <div class="step-images-grid">
                <div class="image-container">
                  <img
                    src="/assets/Toyota_Tacoma_Tuned.png"
                    alt="Toyota Tacoma Tuned"
                    class="step-image"
                    on:mouseenter={() =>
                      handleImageHover(
                        "/assets/Toyota_Tacoma_Tuned.png",
                        "Toyota Tacoma Tuned",
                      )}
                    on:mouseleave={handleImageLeave}
                  />
                  <div class="image-label">Tuning Guide</div>
                </div>
                <div class="image-container">
                  <img
                    src="/assets/Toyota_Tacoma_Mastery.png"
                    alt="Toyota Tacoma Car Mastery"
                    class="step-image"
                    on:mouseenter={() =>
                      handleImageHover(
                        "/assets/Toyota_Tacoma_Mastery.png",
                        "Toyota Tacoma Car Mastery",
                      )}
                    on:mouseleave={handleImageLeave}
                  />
                  <div class="image-label">Car Mastery Perks</div>
                </div>
              </div>
            </div>

            <div class="or-separator">
              <span>— OR —</span>
            </div>

            <div class="farm-option-card">
              <div class="farm-option-header">
                <span class="option-badge option-b">S-Cargo FE</span>
                <span class="car-name"
                  >1989 Nissan S-Cargo <span class="fe-tag">FE</span></span
                >
              </div>
              <p class="step-desc opt-desc">
                Tuning Share Code: <span class="share-code-narrow"
                  >101 759 352</span
                >
              </p>
              <div class="step-images-grid">
                <div class="image-container">
                  <img
                    src="/assets/Nissan_S-Cargo_Tuned.png"
                    alt="Nissan S-Cargo Tuned"
                    class="step-image"
                    on:mouseenter={() =>
                      handleImageHover(
                        "/assets/Nissan_S-Cargo_Tuned.png",
                        "Nissan S-Cargo Tuned",
                      )}
                    on:mouseleave={handleImageLeave}
                  />
                  <div class="image-label">Tuning Guide</div>
                </div>
                <div class="image-container">
                  <img
                    src="/assets/Nissan_S-Cargo_Mastery.png"
                    alt="Nissan S-Cargo Car Mastery"
                    class="step-image"
                    on:mouseenter={() =>
                      handleImageHover(
                        "/assets/Nissan_S-Cargo_Mastery.png",
                        "Nissan S-Cargo Car Mastery",
                      )}
                    on:mouseleave={handleImageLeave}
                  />
                  <div class="image-label">Car Mastery Perks</div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 4 -->
        <div
          class="step-card"
          class:active={driverInstalled &&
            step2Completed &&
            step3Completed &&
            !step4Completed}
          class:disabled={!driverInstalled}
        >
          <div class="step-header">
            <span class="step-num">STEP 4</span>
            <span class="step-title">PLAY EVENTLAB CUSTOM TRACK</span>
            <label class="step-checkbox-wrapper">
              <input
                type="checkbox"
                checked={step4Completed}
                on:change={toggleStep4}
                disabled={!driverInstalled}
              />
              <span class="custom-checkbox"></span>
            </label>
          </div>

          <p class="step-desc">
            In pause menu, go to <strong>Creative Hub</strong> -> <strong>EventLab</strong> -> <strong>Play
            Event</strong> -> <strong>Search</strong>. Enter Share Code <span
              class="share-code-narrow">100 405 213</span
            > and play this event once in <strong>Solo</strong> mode. This ensures that the track
            appears at the top of your <strong>My History</strong> list in EventLab so the bot
            can find it.
          </p>

          <div class="step-images-grid single">
            <div class="image-container">
              <img
                src="/assets/MyHistoryEventLab.png"
                alt="My History EventLab"
                class="step-image"
                on:mouseenter={() =>
                  handleImageHover(
                    "/assets/MyHistoryEventLab.png",
                    "My History EventLab",
                  )}
                on:mouseleave={handleImageLeave}
              />
              <div class="image-label">EventLab History Screen</div>
            </div>
          </div>
        </div>

        <!-- Step 5 -->
        <div
          class="step-card"
          class:disabled={!driverInstalled}
          class:active={driverInstalled &&
            step2Completed &&
            step3Completed &&
            step4Completed}
        >
          <div class="step-header">
            <span class="step-num">STEP 5</span>
            <span class="step-title">READY TO START</span>
          </div>
          <p class="step-desc text-muted">
            Once all configurations are complete, click "Finish Setup" below to
            launch the dashboard and begin running the wheelspin farm.
          </p>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="modal-footer">
      <button
        class="action-btn save"
        disabled={!driverInstalled || isChecking}
        on:click={handleClose}
      >
        Finish Setup
      </button>
    </div>
  </div>

  {#if hoveredImage}
    <div class="zoom-overlay" transition:fade={{ duration: 150 }}>
      <div
        class="zoom-preview-container"
        transition:scale={{ start: 0.95, duration: 150 }}
      >
        <img src={hoveredImage} alt={hoveredAlt} class="zoom-preview-image" />
        <div class="zoom-preview-label">{hoveredAlt}</div>
      </div>
    </div>
  {/if}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(18, 25, 27, 0.65);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    outline: none;
    cursor: default;
  }

  .modal {
    background: #1b2628;
    width: 580px;
    max-width: 92vw;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    border: 4px solid #000000;
    box-shadow:
      0 0 0 3px var(--fh6-lime),
      0 20px 48px rgba(0, 0, 0, 0.6);
  }

  /* ── Header ─────────────────────────────────────────── */
  .modal-header {
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 2px solid var(--fh6-lime);
    background: rgba(0, 0, 0, 0.25);
    padding: 0 20px;
    flex-shrink: 0;
  }

  .modal-title {
    font-size: 16px;
    font-weight: 900;
    letter-spacing: 0.18em;
    color: #ffffff;
    text-transform: uppercase;
  }

  /* ── Body ───────────────────────────────────────────── */
  .modal-body {
    padding: 24px;
    background: radial-gradient(
      circle at top left,
      rgba(202, 255, 2, 0.03) 0%,
      transparent 70%
    );
    max-height: 80vh;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--gap-lg);
  }

  /* Custom styling for the scrollbar inside modal body to match theme */
  .modal-body::-webkit-scrollbar {
    width: 6px;
  }
  .modal-body::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.1);
  }
  .modal-body::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
  .modal-body::-webkit-scrollbar-thumb:hover {
    background: var(--fh6-lime);
  }

  .welcome-intro p {
    font-size: 15px;
    color: rgba(255, 255, 255, 0.7);
    line-height: 1.5;
  }

  .welcome-intro strong {
    color: var(--fh6-lime);
  }

  /* ── Status Section ─────────────────────────────────── */
  .status-section {
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid rgba(255, 255, 255, 0.06);
    padding: 16px;
    border-radius: 8px;
  }

  .status-label {
    font-size: 13px;
    font-weight: 800;
    letter-spacing: 0.14em;
    color: var(--fh6-lime);
    text-transform: uppercase;
    display: block;
    margin-bottom: 10px;
  }

  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
  }

  .driver-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .driver-name {
    font-size: 14px;
    font-weight: 800;
    color: #ffffff;
    text-transform: uppercase;
  }

  .driver-desc {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.4);
  }

  .conn {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    flex-shrink: 0;
  }

  .conn-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #ea4335;
    display: block;
  }

  .conn.checking .conn-dot {
    background: #fbbc05;
    animation: pulse 1s infinite alternate;
  }

  .conn.online .conn-dot {
    background: var(--fh6-lime);
    box-shadow: 0 0 8px var(--fh6-lime-glow);
  }

  .conn-text {
    display: flex;
    flex-direction: column;
  }

  .conn-status {
    font-size: 13px;
    font-weight: 800;
    color: #ea4335;
    line-height: 1;
  }

  .conn.checking .conn-status {
    color: #fbbc05;
  }

  .conn.online .conn-status {
    color: var(--fh6-lime);
  }

  .conn-label {
    font-size: 11px;
    font-weight: 700;
    color: rgba(255, 255, 255, 0.25);
    letter-spacing: 0.1em;
    text-transform: uppercase;
    margin-top: 2px;
  }

  /* ── Setup Steps ────────────────────────────────────── */
  .setup-steps {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .step-card {
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    padding: 16px;
    transition: all 0.2s ease;
  }

  .step-card.active {
    border-color: rgba(202, 255, 2, 0.25);
    background: rgba(202, 255, 2, 0.02);
  }

  .step-card.disabled {
    opacity: 0.4;
  }

  .step-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
  }

  .step-num {
    font-size: 12px;
    font-weight: 800;
    color: #12191b;
    background: var(--fh6-lime);
    padding: 2px 6px;
    border-radius: 3px;
    letter-spacing: 0.05em;
  }

  .step-card.disabled .step-num {
    background: rgba(255, 255, 255, 0.2);
    color: rgba(255, 255, 255, 0.5);
  }

  .step-title {
    font-size: 14px;
    font-weight: 900;
    color: #ffffff;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .step-desc {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.65);
    line-height: 1.45;
  }

  .opt-desc {
    margin-top: 4px;
  }

  .step-success {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fh6-lime);
    font-size: 14px;
    font-weight: 800;
    margin-top: 8px;
  }

  .step-success svg {
    width: 16px;
    height: 16px;
  }

  .action-row {
    display: flex;
    gap: 10px;
    margin-top: 14px;
  }

  .install-btn {
    flex: 1;
    background: var(--fh6-lime);
    border: 1.5px solid var(--fh6-lime-dim);
    color: var(--fh6-lime-solid);
    font-size: 13px;
    font-weight: 800;
    padding: 8px 16px;
    clip-path: var(--clip-btn);
  }

  .install-btn:hover:not(:disabled) {
    background: var(--fh6-lime-dim);
    box-shadow: 0 0 12px var(--fh6-lime-glow);
  }

  .check-btn {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: #ffffff;
    font-size: 13px;
    padding: 8px 14px;
  }

  .check-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }

  .spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 2px solid rgba(0, 0, 0, 0.2);
    border-top-color: var(--fh6-lime-solid);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-right: 6px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* ── Custom Checkbox styling ────────────────────────── */
  .step-checkbox-wrapper {
    display: inline-flex;
    align-items: center;
    cursor: pointer;
    margin-left: auto;
    position: relative;
    user-select: none;
  }
  .step-checkbox-wrapper input {
    position: absolute;
    opacity: 0;
    cursor: pointer;
    height: 0;
    width: 0;
  }
  .custom-checkbox {
    height: 18px;
    width: 18px;
    background-color: rgba(0, 0, 0, 0.4);
    border: 1.5px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    position: relative;
    transition: all 0.2s ease;
  }
  .step-checkbox-wrapper:hover input:not(:disabled) ~ .custom-checkbox {
    border-color: var(--fh6-lime);
    box-shadow: 0 0 6px var(--fh6-lime-glow);
  }
  .step-checkbox-wrapper input:checked ~ .custom-checkbox {
    background-color: var(--fh6-lime);
    border-color: var(--fh6-lime-dim);
    box-shadow: 0 0 10px var(--fh6-lime-glow);
  }
  .custom-checkbox:after {
    content: "";
    position: absolute;
    display: none;
  }
  .step-checkbox-wrapper input:checked ~ .custom-checkbox:after {
    display: block;
  }
  .step-checkbox-wrapper .custom-checkbox:after {
    left: 5px;
    top: 1px;
    width: 4px;
    height: 9px;
    border: solid var(--fh6-lime-solid);
    border-width: 0 2px 2px 0;
    transform: rotate(45deg);
  }
  .step-checkbox-wrapper input:disabled ~ .custom-checkbox {
    opacity: 0.3;
    cursor: not-allowed;
    border-color: rgba(255, 255, 255, 0.1);
  }

  /* ── Center narrow share code layout ────────────────── */
  .share-code-narrow {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 700;
    color: var(--fh6-lime);
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(202, 255, 2, 0.2);
    padding: 2px 8px;
    border-radius: 4px;
    display: inline-block;
    letter-spacing: 0.05em;
    margin: 4px 0;
  }

  /* ── Farm Options layout ────────────────────────────── */
  .farm-options {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 12px;
  }
  .farm-option-card {
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 6px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .farm-option-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .option-badge {
    font-size: 11px;
    font-weight: 800;
    padding: 1px 6px;
    border-radius: 3px;
    color: #ffffff;
    text-transform: uppercase;
  }
  .option-badge.option-a {
    background: var(--fh6-teal);
  }
  .option-badge.option-b {
    background: var(--fh6-pink);
  }
  .car-name {
    font-size: 13px;
    font-weight: 700;
    color: #ffffff;
  }
  .fe-tag {
    font-size: 11px;
    font-weight: 800;
    background: linear-gradient(90deg, #ff007f, #7f00ff);
    color: #ffffff;
    padding: 1px 4px;
    border-radius: 2px;
    margin-left: 4px;
    letter-spacing: 0.05em;
  }
  .or-separator {
    text-align: center;
    font-size: 12px;
    font-weight: 800;
    color: #ffffff;
    letter-spacing: 0.1em;
    margin: 2px 0;
  }
  .highlight-car {
    color: var(--fh6-lime);
  }

  /* ── Image grids with hover zoom ───────────────────── */
  .step-images-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-top: 12px;
  }
  .step-images-grid.single {
    grid-template-columns: 1fr;
    max-width: 244px;
    margin: 12px auto 0 auto;
  }
  .image-container {
    position: relative;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.25);
  }
  .step-image {
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
    display: block;
    border-radius: 5px;
    border: 1px solid transparent;
    transition:
      transform 0.2s cubic-bezier(0.25, 0.8, 0.25, 1),
      border-color 0.2s ease,
      box-shadow 0.2s ease;
    position: relative;
    z-index: 1;
    cursor: zoom-in;
  }
  .step-image:hover {
    transform: scale(1.03);
    border-color: var(--fh6-lime);
    box-shadow: 0 0 10px var(--fh6-lime-glow);
    z-index: 2;
  }
  .image-label {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(18, 25, 27, 0.85);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.85);
    font-size: 12px;
    font-weight: 800;
    text-transform: uppercase;
    text-align: center;
    padding: 4px 0;
    pointer-events: none;
    letter-spacing: 0.05em;
    border-bottom-left-radius: 5px;
    border-bottom-right-radius: 5px;
  }

  /* ── Zoom Overlay with Uncropped Aspect Ratio ────────── */
  .zoom-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    z-index: 10005; /* higher than modal (9999) */
  }
  .zoom-preview-container {
    position: relative;
    background: #12191b;
    border: 3px solid var(--fh6-lime);
    border-radius: 12px;
    box-shadow:
      0 20px 50px rgba(0, 0, 0, 0.85),
      0 0 30px var(--fh6-lime-glow);
    padding: 6px;
    padding-bottom: 40px; /* Space at the bottom for the label */
    max-width: 85vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
  }
  .zoom-preview-image {
    max-width: 100%;
    max-height: calc(80vh - 65px);
    object-fit: contain; /* Displays the image in its original uncropped format */
    border-radius: 6px;
    display: block;
  }
  .zoom-preview-label {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(18, 25, 27, 0.9);
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
    border-top: 1.5px solid rgba(255, 255, 255, 0.08);
    color: var(--fh6-lime);
    font-size: 13px;
    font-weight: 800;
    text-transform: uppercase;
    text-align: center;
    padding: 8px 0;
    pointer-events: none;
    letter-spacing: 0.08em;
    border-bottom-left-radius: 9px;
    border-bottom-right-radius: 9px;
  }

  /* ── Error Box ──────────────────────────────────────── */
  .error-box {
    margin-top: 12px;
    background: rgba(254, 2, 136, 0.06);
    border: 1.5px solid rgba(254, 2, 136, 0.3);
    border-radius: 6px;
    padding: 10px 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .error-title {
    font-size: 11px;
    font-weight: 900;
    color: var(--fh6-pink);
    letter-spacing: 0.08em;
  }

  .error-msg {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.85);
    font-family: var(--font-mono);
  }

  /* ── Footer ─────────────────────────────────────────── */
  .modal-footer {
    display: flex;
    justify-content: flex-end;
    padding: 14px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.25);
  }

  .action-btn {
    font-size: 13px;
    font-weight: 900;
    letter-spacing: 0.08em;
    border-radius: 4px;
    text-transform: uppercase;
    transition: all 0.2s ease;
    padding: 8px 18px;
    cursor: pointer;
  }

  .action-btn.save {
    background: var(--fh6-lime);
    border: 1.5px solid var(--fh6-lime-dim);
    color: var(--fh6-lime-solid);
  }

  .action-btn.save:hover:not(:disabled) {
    background: var(--fh6-lime-dim);
    box-shadow: 0 0 12px var(--fh6-lime-glow);
  }

  @keyframes pulse {
    from {
      opacity: 0.5;
    }
    to {
      opacity: 1;
    }
  }
</style>
