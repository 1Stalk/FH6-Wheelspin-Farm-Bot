# Forza Horizon 6 Wheelspin Farm Bot

![Main Window](assets/screenshots/main_window.png)

A fully automated, 100% AFK bot for farming Super Wheelspins, Credits, and Skill Points in Forza Horizon 6.

## The Auto-Farming Cycle

The bot runs on a continuous, fully automated 4-stage loop that handles the entire farming process AFK, compiling the best legal farming methods discovered by the community to date:

```
┌────────────────────────────────────┐      ┌────────────────────────────────────┐
│ Stage 1: Colossus Autopilot        │      │ Stage 2: Eventlab farm map         │
│ Farm credits.                      │─────>│ Farm SP.                           │
└────────────────────────────────────┘      └────────────────────────────────────┘
                    ▲                                       │
                    │                                       │
                    │                                       ▼
┌────────────────────────────────────┐      ┌────────────────────────────────────┐
│ Stage 4: Unlock Super Wheelspins   │      │ Stage 3: Buy cars in journal       │
│ Spend SP on Subaru's 22B + Cleanup │<─────│ Spend credits. Bulk buy Subaru 22B │
└────────────────────────────────────┘      └────────────────────────────────────┘
```

## Prerequisites

To run the bot, your system needs the following components:

* **WebView2 Runtime** (used for rendering the user interface)
* **ViGEmBus Driver** (required for virtual controller emulation)

> [!NOTE]
> If either of these dependencies is missing, the bot will automatically detect it and prompt you to install them upon startup.

## Getting Started

1. Set your Forza Horizon 6 to **Windowed** or **Borderless** mode (2560x1440 or 1920x1080 is recommended).
2. **Lock your game framerate to 60 FPS** and ensure a stable 60 FPS. If FPS drops to 30-40 or below, the bot's keystrokes and inputs may not register properly.
3. In the game, enter the **Open World**, open the **Pause Menu**, and make sure you are on the **first tab**.
4. Launch the bot executable.
5. Set up Stage 1 & 2 cars following the **Initial Setup** guide in the bot's UI on launch.
6. Click **Start** and let the bot do the work!

## Ban Risk & Safety Measures (Am I going to get banned?)

Using any automation tool or bot is technically against the game's Terms of Service and carries an inherent risk of suspension or ban. There is **no such thing as a 100% safe bot**. Use this tool at your own risk.

> [!TIP]
> **Safety Recommendation**: To minimize detection risk and keep your account safe, it is highly recommended not to run the bot for more than 12 consecutive hours at a time.

However, this bot was designed from the ground up with **advanced anti-detection measures** to mimic human players as closely as possible:

* **Hardware Emulation (ViGEmBus)**: The bot does not use suspicious simulated keyboard or mouse events. Instead, it emulates a virtual Xbox 360 controller at the driver level via ViGEmBus—the same highly-trusted system driver utilized by popular software like **DS4Windows**, **Parsec**, **ReWASD**, and **Moonlight/Sunshine** to handle virtual gamepad inputs. This makes the bot appear to the operating system and game as a physical hardware controller.
* **Lognormal Human Timing**: All button hold durations, navigation intervals, stage-switching pauses, and "thinking delays" are randomly sampled using lognormal distributions modeled on real human keystroke dynamics.
* **Smooth Kinematics (Ruckig OTG)**: Analog stick and trigger movements do not snap instantly. The bot uses the **Ruckig** online trajectory generation library to calculate smooth velocity, acceleration, and jerk curves.
* **OpenSimplex Tremor & Noise**: Real human hands have micro-tremors. The bot constantly injects organic micro-vibrations into held sticks and triggers using OpenSimplex noise.
* **Analog Stick Overshooting**: To mimic human error and correction, the bot has a random chance (35%) of overshooting its joystick target slightly before correcting, simulating Fitts's Law target acquisition.