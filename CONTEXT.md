# why2025-badge-rust

why2025-badge-rust is the Rust support stack for building software for the WHY2025 badge. It exists to provide the bindings, build integration, Emulation, and helper crates that make badge software practical to write and test.

## Language

**WHY2025 badge**:
The physical device that runs BadgeVMS and Apps.
_Avoid_: badge hardware, badge device

**Compute Unit**:
The removable badge module built around the ESP32-P4-32MB and intended to be the main execution unit for badge software.
_Avoid_: CPU module, main board

**Carrier Board**:
The badge board that hosts the Compute Unit's supporting hardware and the ESP32-C6 connectivity chip for Wi-Fi and Bluetooth.
_Avoid_: main board, radio board

**Wi-Fi**:
The badge's wireless-networking capability. In this repo BadgeVMS exposes a Wi-Fi ABI and **System Settings** configures Wi-Fi connections; the hardware docs place this capability on the **Carrier Board**'s ESP32-C6.
_Avoid_: WiFi, network stack

**Bluetooth**:
The badge's short-range wireless connectivity capability. The hardware docs place it on the **Carrier Board**'s ESP32-C6 alongside **Wi-Fi**.
_Avoid_: Wi-Fi, radio stack

**Micro SD Card**:
The removable storage medium used by the badge's storage hardware. In the M2 storage schematic it appears as a `Micro_SD_Card_Det2` connector, and in BadgeVMS the corresponding storage device is exposed as `SD0` when present.
_Avoid_: SD0, storage alias

**NOR Flash**:
The non-volatile flash-storage chip on the badge's storage hardware. In the M2 storage schematic it appears as the `XT25F128FWOIGT-W` part with description `NOR Flash`, and in BadgeVMS the corresponding storage device is exposed as `FLASH0`.
_Avoid_: FLASH0, firmware binary, generic flash

**Display**:
The physical 4-inch square screen on the **WHY2025 badge**. In the current BadgeVMS and emulation code in this repo, fullscreen presentation targets 720x720 pixels.
_Avoid_: window, framebuffer, screen surface

**Frontpanel**:
The front-facing badge hardware layer described in the hardware-design repo. The dedicated Frontpanel docs in that repo present it as a customizable part with a KiCad template and optional side-B population including four WS2812B LEDs and three pogo pins.
_Avoid_: frontplate, faceplate

**Case**:
The badge enclosure design tracked in the hardware repo. Current docs describe a minimal case that mostly covers the batteries for grip while leaving much of the badge exposed, plus a fuller back-cover variant.
_Avoid_: carrier board, frontpanel, shell

**Spacer**:
The badge spacer part used in production and in printable or laser-cut upgrades. Current docs describe it as the part that sets keyboard protrusion and display clearance, including both full-spacer and two-part variants.
_Avoid_: case, shim

**Keyboard**:
The physical badge keyboard exposed by BadgeVMS as `KEYBOARD0`. In the current firmware test layout it has six rows, including a top row of `ESC`, square, triangle, cross, circle, cloud, diamond, and backspace, then number, QWERTY, ASDF, ZXCV, and bottom rows with `Fn`, arrows, and modifiers.
_Avoid_: keypad, button matrix

**Sensor**:
A physical measurement component on the **WHY2025 badge**. The hardware docs and current firmware in this repo expose two named sensors: **BMI270** for motion and orientation, and **BME690** for air-quality and environmental readings.
_Avoid_: device, peripheral

**BMI270**:
The badge's motion-and-orientation **Sensor**. Hardware docs describe it as the accelerometer-and-gyroscope chip, and the current firmware registers it as `ORIENTATION0`.
_Avoid_: generic IMU, gas sensor

**BME690**:
The badge's environmental **Sensor**. Hardware docs describe it as the air-quality, temperature, humidity, and pressure chip, and the current firmware registers it as `GAS0`.
_Avoid_: generic gas sensor, orientation sensor

**BadgeVMS**:
The firmware platform on the WHY2025 badge that runs Apps.
_Avoid_: runtime

**Compositor**:
The BadgeVMS system component that manages windows, framebuffers, event polling, screen presentation, and some foreground-app scheduling behavior.
_Avoid_: window manager, graphics backend

**Startup Configuration**:
The BadgeVMS boot-time configuration that tells init which programs to launch and supervise, including policies such as `run_once`, `restart_on_failure`, `start_delay`, and `start_every`. In the current firmware it is loaded from `init.toml`.
_Avoid_: startup app, boot script

**Emulation**:
Running an App against a host-side implementation of BadgeVMS instead of on the WHY2025 badge.
_Avoid_: host emulation, emulator

**Host build**:
An App build compiled for the host machine and intended to run using Emulation.
_Avoid_: emulator build, Linux build, my machine build

**BadgeVMS target**:
The Rust target for building Apps to run on BadgeVMS.
_Avoid_: badge target

**App Linking**:
The App build policy for producing a BadgeVMS-loadable shared object. In this repo the primary workflow uses the built-in `riscv32imafc-unknown-badgevms` target, which owns `main`, PIC, shared-object linking, and export pruning. The old `why2025-badge-build` `build.rs` flow remains only as legacy `riscv32imafc-unknown-none-elf` compatibility.
_Avoid_: linker args, retain.txt, build.rs snippet

**App-facing facade**:
The Rust crate layer an App depends on to target both BadgeVMS and Emulation without pulling the lower-level badge bindings into the App's build setup. In this repo that role is currently provided by `why2025-badge-app-no-std`, mainly as no*std runtime and entry glue.
\_Avoid*: runtime, sys crate, app template

**Provided runtime**:
The default runtime bundle supplied by the current **App-facing facade** for `riscv32` Apps. In this repo it enables the **Provided allocator** and **Provided panic handler** unless an App disables the default feature set.
_Avoid_: BadgeVMS, system runtime, allocator

**Provided allocator**:
The default global allocator supplied by the current **App-facing facade** on `riscv32`. In this repo it is the Talc-based allocator behind the `provided-allocator` feature.
_Avoid_: heap, system allocator, malloc

**Provided panic handler**:
The default panic handler supplied by the current **App-facing facade** on `riscv32`. In this repo it prints panic information through BadgeVMS `printf` and then halts.
_Avoid_: unwind runtime, crash reporter, exception handler

**Embedded graphics driver**:
The Rust graphics-driver layer that adapts the badge **Display** to the `embedded-graphics` ecosystem. In this repo that role is currently provided by the `why2025-badge-embedded-graphics` crate.
_Avoid_: compositor, window, framebuffer

**Firmware ABI**:
The exported firmware-level symbol surface that BadgeVMS makes available to Apps. In this repo it is the ABI surface that the workspace binds, links against, and tracks for host-side Emulation support.
_Avoid_: ABI manifest, app ABI, App manifest

**Raw bindings**:
The autogenerated Rust FFI layer for functions exported by the WHY2025 badge. In this repo it is canonically provided by `why2025-badge-sys-bindings`, generated from firmware headers, and kept close to the badge ABI rather than wrapped as a safe or idiomatic Rust API. The sibling `why2025-badge-sys` crate re-exports that surface and adds wrapper-only behavior such as Emulation and badge-link support.
_Avoid_: sys crate, safe wrapper, runtime

**Emulated layer**:
The host-side export layer inside this repo that implements or shims firmware symbols for Emulation. In this repo it is the layer whose symbol support is counted by **Firmware Symbol Coverage**.
_Avoid_: Emulation, host libc, safe wrapper

**Badge-link metadata**:
The Cargo build-script metadata that carries badge-linking inputs such as the entry symbol and retain-symbols file to the final App build step. In this repo `why2025-badge-sys` can still emit it behind `badge-app-link`, and `why2025-badge-build` can still consume it for the legacy `riscv32imafc-unknown-none-elf` path, but the repo's primary BadgeVMS workflow no longer depends on this metadata because the built-in target owns final linking.
_Avoid_: linker args, retain.txt, final binary

**Firmware Symbol Coverage**:
The repo's accounting of how much of the exported firmware ABI is supported in Linux host-side **Emulation**. In this repo `why2025-badge-sys` documents it by mirroring the vendored `symbols.yml` export list and marking each symbol as supported, partial, or unsupported in the emulated layer.
_Avoid_: ABI manifest, unit-test coverage, feature matrix

**App entry macro**:
The macro an App uses to expose its Rust entry body as the correct `main` symbol for both BadgeVMS and host builds. In this repo the example Apps use `why2025_badge_app_no_std::app_main!`.
_Avoid_: main function, build script, linker flag

**App entry function**:
The Rust function that contains an App's top-level behavior and is passed to the **App entry macro**. In the example Apps in this repo it is typically a `fn run() -> i32` that returns the App's exit status.
_Avoid_: main, callback, task function

**BadgeVMS path**:
A VMS-style path used by BadgeVMS, written as `DEVICE:[DIR.SUBDIR]FILE`.
Common path devices verified in this repo: `FLASH0` for internal flash storage, `SD0` for SD card storage when present, `STORAGE` as a logical storage alias over `SD0` then `FLASH0` or just `FLASH0`, `APPS` as the logical root for installed App files under `...[BADGEVMS.APPS]`, and `APP` as the current App-relative path base used by the App APIs.
_Avoid_: Unix path, host path

**App**:
Software that runs on the badge.
_Avoid_: badge app, application, BadgeVMS application, badge binary

**Installed App**:
An App instance present on BadgeVMS and represented by an App manifest.
_Avoid_: installed application, local app, dummy app

**Preinstalled App**:
An Installed App staged into the badge image at build time.
_Avoid_: bundled app, built-in app, default app

**Core App**:
An App that the firmware build treats as part of the badge's system functionality.
_Avoid_: system app, built-in app

**Regular App**:
An App that the firmware build treats as ordinary app content rather than badge system functionality. A **Regular App** may still be preinstalled.
_Avoid_: content app, bundled content

**OTA updater**:
The **Preinstalled App** that checks for and installs **App updates** and **Firmware updates**.
_Avoid_: update app, system updates, `why2025_ota`

**Launcher**:
The **Preinstalled App** that lists installed Apps and launches the selected App.
_Avoid_: BadgeVMS Launcher, application launcher, `badgevms_launcher`

**System Settings**:
The **Preinstalled App** that exposes badge settings, currently Wi-Fi settings and an About screen.
_Avoid_: settings, settings app, `badgevms_settings`

**Window**:
A Compositor-managed UI surface that an App creates to present content, receive events, and control properties such as title, position, size, and flags.
_Avoid_: surface, screen, framebuffer

**Fullscreen Window**:
A **Window** configured to occupy the full screen and participate in the Compositor's fullscreen scheduling rules.
_Avoid_: fullscreen app, fullscreen mode

**Floating Window**:
A **Window** created in floating mode rather than fullscreen mode, with normal window positioning instead of full-screen occupancy.
_Avoid_: undecorated window, normal window

**Always-on-top Window**:
A **Window** configured to stay above other non-fullscreen windows in the Compositor's stacking order.
_Avoid_: foreground window, fullscreen window

**Low-priority Window**:
A **Fullscreen Window** configured not to receive the Compositor's foreground priority boost even when it is frontmost.
_Avoid_: background app, low-priority app

**Framebuffer**:
The backing pixel buffer attached to a **Window** that an **App** draws into before presenting through the **Compositor**.
_Avoid_: screen buffer, canvas, surface

**Event**:
A BadgeVMS notification polled from a **Window**'s event queue.
Event handling is window-scoped: receiving an **Event** does not by itself destroy a **Window** or terminate a **Process**, **Task**, or **Thread**; app code decides what to do in response.
The public event types in this repo are **No Event**, **Quit Event**, **Key Down Event**, **Key Up Event**, and **Window Resize Event**. **Keyboard Event** is the umbrella term for the key-down and key-up forms.
In the traced current firmware and host emulation in this repo, polling may return **No Event** when no queued notification is available and the built-in producer emits keyboard events; quit and resize are defined in the API but were not observed emitted by that producer.
_Avoid_: SDL event, callback

**No Event**:
The sentinel **Event** value returned when a **Window** has no queued notification available.
It is not queued input or window activity, and it does not imply any change to a **Window**, **Process**, **Task**, or **Thread**.
_Avoid_: null event, idle event

**Quit Event**:
A BadgeVMS **Event** that asks app code to stop running its window loop and exit cleanly.
It is a notification on a **Window**'s event queue, not a direct command that by itself destroys the **Window** or terminates the owning **Process**, **Task**, or **Thread**.
In the traced current firmware in this repo, apps handle this type, but the built-in BadgeVMS event producer was not observed emitting it.
_Avoid_: close window, kill task, kill process

**Keyboard Event**:
A BadgeVMS **Event** carrying key input details such as scancode, key code, modifiers, resolved text, press state, and repeat state.
The current API uses **Key Down Event** and **Key Up Event** variants for key press and release; these are input notifications, not lifecycle operations on a **Window**, **Process**, **Task**, or **Thread**.
_Avoid_: key event, SDL keyboard event

**Key Down Event**:
A **Keyboard Event** reporting that a key is pressed for a **Window**.
It is an input notification, not a command that changes the lifecycle of a **Window**, **Process**, **Task**, or **Thread**.
_Avoid_: key press command, button down action

**Key Up Event**:
A **Keyboard Event** reporting that a key is released for a **Window**.
It is an input notification, not a command that changes the lifecycle of a **Window**, **Process**, **Task**, or **Thread**.
_Avoid_: key release command, button up action

**Window Resize Event**:
A BadgeVMS **Event** reporting that a **Window**'s size changed.
It is a window notification, not a command that by itself resizes, destroys, or terminates a **Window**, **Process**, **Task**, or **Thread**.
In the traced current firmware and host emulation in this repo, this public event type was not observed emitted by the built-in BadgeVMS event producer.
_Avoid_: resize command, resize operation

**Scancode**:
The physical-key identifier carried by a **Keyboard Event**, used to distinguish which badge key was pressed or released independent of the resolved text.
_Avoid_: key code, character, button

**Key Code**:
The layout-resolved virtual-key identifier carried by a **Keyboard Event**. In this repo the named key-code constants use the `BADGEVMS_KEY_*` namespace.
_Avoid_: scancode, character, button

**Key Modifier**:
The modifier-key state carried by a **Keyboard Event**.
Modifiers defined in this repo:

- Left Shift (`BADGEVMS_KMOD_LSHIFT`)
- Right Shift (`BADGEVMS_KMOD_RSHIFT`)
- Level 5 (`BADGEVMS_KMOD_LEVEL5`)
- Left Ctrl (`BADGEVMS_KMOD_LCTRL`)
- Right Ctrl (`BADGEVMS_KMOD_RCTRL`)
- Left Alt (`BADGEVMS_KMOD_LALT`)
- Right Alt (`BADGEVMS_KMOD_RALT`)
- Left GUI (`BADGEVMS_KMOD_LGUI`)
- Right GUI (`BADGEVMS_KMOD_RGUI`)
- Num Lock (`BADGEVMS_KMOD_NUM`)
- Caps Lock (`BADGEVMS_KMOD_CAPS`)
- Mode / AltGr (`BADGEVMS_KMOD_MODE`)
- Scroll Lock (`BADGEVMS_KMOD_SCROLL`)
  Grouped aliases also exist for Ctrl, Shift, Alt, and GUI.
  _Avoid_: modifier mask, SDL mod

**Foreground App**:
An App whose **Fullscreen Window** is currently the frontmost eligible fullscreen window for foreground scheduling. The compositor may temporarily boost its **Task** priority while it remains in that state.
_Avoid_: active app, focused app, fullscreen app

**App ID**:
The stable identifier for an App across BadgeVMS and BadgeHub.
_Avoid_: unique identifier, slug, name

**App manifest**:
The JSON record BadgeVMS uses to identify, describe, and launch an **Installed App**. The same schema is usually authored as `manifest.json` for bundled firmware Apps and is then stored by BadgeVMS as a separate `<App ID>.json` record under `APPS` next to the App's installed files. It contains the **App ID**, display metadata, launch metadata such as `binary_path`, optional `interpreter`, an optional **Metadata file** reference through `metadata_file`, and `source`. It does not contain the App binary, a full file inventory, BadgeHub revision history, or the firmware ABI. In the traced launch path in this repo, `binary_path` is the field that actually drives launch; `interpreter` is stored in the schema but is not consumed by `application_launch`, and the **Metadata file** is separate from the manifest itself.
_Avoid_: metadata file, ABI manifest

**App files**:
The files of an Installed App that live under its `APPS:[<App ID>]` directory.
_Avoid_: app payload, app directory, the app

**App executable**:
The executable file of an Installed App.
_Avoid_: app binary, main binary

**Firmware binary**:
The binary file used to install a firmware update.
_Avoid_: `badgevms.bin`, firmware image, firmware file

**App update**:
A newer **Revision** of a **BadgeHub Project** that BadgeVMS can install onto an existing **Installed App**.
_Avoid_: app upgrade, downloaded app, OTA item

**Firmware update**:
A newer **Revision** of BadgeVMS that BadgeVMS can install onto the **WHY2025 badge**.
_Avoid_: system update, OTA item, firmware upgrade

**Version file**:
The file that records the version for a Revision or firmware update.
_Avoid_: `version.txt`, version artifact

**Metadata file**:
An optional App-owned file referenced by an **App manifest** through the `metadata_file` field. It lives inside the App's installed files rather than in the separate `<App ID>.json` App manifest record. BadgeVMS stores and path-validates the reference, and OTA currently points it at `metadata.json`, but the traced launch path does not use it to identify or launch the App.
_Avoid_: App manifest, `manifest.json`

**ABI manifest**:
The BadgeVMS exported-symbol manifest, vendored in this repo as `firmware/badgevms/symbols.yml`. It describes the firmware ABI surface that Apps can import and that this workspace binds, links against, and tracks for Emulation coverage. It is platform-level, not per-App, and it does not describe App installation state or launch metadata.
_Avoid_: App manifest, app metadata

**Interpreter**:
The program BadgeVMS uses to run an App instead of launching its binary directly.

**Interpreted App**:
An App that BadgeVMS runs through an Interpreter instead of launching directly.

**BadgeHub-sourced App**:
An **Installed App** whose App manifest records BadgeHub as its source.
_Avoid_: BadgeHub app, downloaded app, default app

**BadgeHub**:
The place to share Apps.
_Avoid_: Badgehub, badgehub

**BadgeHub Project**:
A BadgeHub record that publishes an App.
_Avoid_: app listing, upload

**Default BadgeHub Project**:
A BadgeHub Project that BadgeHub marks as Default and that the OTA updater treats as part of its seed list for missing BadgeHub-sourced Apps.
_Avoid_: default app, default project

**Revision**:
A versioned release of a BadgeHub Project.
_Avoid_: build, upload

**Task**:
A BadgeVMS execution unit managed by the tasking subsystem. BadgeVMS uses this as the umbrella term for the running units counted by `get_num_tasks()` and affected by task-priority controls. In the current firmware that public task count excludes the internal kernel tasks.
_Avoid_: FreeRTOS task, kernel task

**Process**:
A **Task** created from an executable path, including App launch.
_Avoid_: OS process

**Thread**:
A **Task** created from a function pointer and user data rather than from an executable path. In the current firmware it shares the creator's runtime thread state instead of getting its own.
_Avoid_: OS thread

## Relationships

- The **WHY2025 badge** has one **Compute Unit**
- The **WHY2025 badge** has one **Carrier Board**
- The **WHY2025 badge** may use **Wi-Fi**
- The **WHY2025 badge** may use **Bluetooth**
- The **WHY2025 badge** has one **NOR Flash**
- The **WHY2025 badge** may have one **Micro SD Card**
- The **WHY2025 badge** has one **Display**
- The **WHY2025 badge** has one **Frontpanel**
- The **WHY2025 badge** may have one **Case**
- The **WHY2025 badge** has one **Spacer**
- The **WHY2025 badge** has one **Keyboard**
- The **WHY2025 badge** has one or more **Sensors**
- The **WHY2025 badge** runs **BadgeVMS**
- The **Compute Unit** runs **BadgeVMS**
- The **Carrier Board** provides **Wi-Fi**
- The **Carrier Board** provides **Bluetooth**
- BadgeVMS may expose **NOR Flash** as `FLASH0`
- BadgeVMS may expose a **Micro SD Card** as `SD0`
- The `STORAGE:` logical name may resolve to a **Micro SD Card** before **NOR Flash**
- A **Case** covers the back of the **WHY2025 badge**
- A **Spacer** provides keyboard protrusion and display clearance for the **WHY2025 badge**
- The **Compositor** presents **Windows** on the **Display**
- The **Keyboard** produces **Keyboard Events**
- A **BMI270** is a **Sensor**
- A **BME690** is a **Sensor**
- **BadgeVMS** has one **Compositor**
- **BadgeVMS** has one **Startup Configuration**
- **Emulation** runs **Apps** against a host-side implementation of **BadgeVMS**
- A **Host build** runs using **Emulation**
- A **BadgeVMS target** builds **Apps** to run on **BadgeVMS**
- **App Linking** applies to **Apps** built for the **BadgeVMS target**
- An **App** may depend on an **App-facing facade**
- An **App-facing facade** supports **Apps** across **BadgeVMS** and **Emulation**
- An **App-facing facade** may provide a **Provided runtime**
- A **Provided runtime** may include a **Provided allocator**
- A **Provided runtime** may include a **Provided panic handler**
- An **App** may use an **Embedded graphics driver**
- An **Embedded graphics driver** targets the **Display**
- The **Firmware ABI** is described by the **ABI manifest**
- **Raw bindings** expose badge-exported symbols to **Apps**
- **Raw bindings** expose the **Firmware ABI** to **Apps**
- The **Emulated layer** supports **Emulation**
- **Firmware Symbol Coverage** tracks support for the **Firmware ABI** in the **Emulated layer**
- An **App-facing facade** may re-export **Raw bindings**
- **Badge-link metadata** supports **App Linking**
- **Firmware Symbol Coverage** tracks **Emulation** support for the **ABI manifest**
- An **App** may use an **App entry macro**
- An **App entry macro** wraps one **App entry function**
- An **App entry function** belongs to one **App**
- **BadgeVMS** runs **Apps**
- **BadgeVMS** uses **BadgeVMS paths**
- An **App** has one **App ID**
- An **Installed App** is an **App**
- An **Installed App** has one **App ID**
- An **Installed App** has one **App manifest**
- A **Preinstalled App** is an **Installed App**
- A **Core App** is an **App**
- A **Regular App** is an **App**
- The **OTA updater** is a **Core App**
- The **Launcher** is a **Core App**
- **System Settings** is a **Core App**
- The **OTA updater** is a **Preinstalled App**
- The **Launcher** is a **Preinstalled App**
- **System Settings** is a **Preinstalled App**
- An **App** may create **Windows**
- A **Fullscreen Window** is a **Window**
- A **Floating Window** is a **Window**
- An **Always-on-top Window** is a **Window**
- A **Low-priority Window** is a **Fullscreen Window**
- A **Window** may have one **Framebuffer**
- A **Window** may produce **Events**
- A **No Event** indicates that no queued **Event** was available from a **Window**
- A **Quit Event** is an **Event**
- A **Keyboard Event** is an **Event**
- A **Key Down Event** is a **Keyboard Event**
- A **Key Up Event** is a **Keyboard Event**
- A **Window Resize Event** is an **Event**
- A **Keyboard Event** has one **Scancode**
- A **Keyboard Event** has one **Key Code**
- A **Keyboard Event** may have one or more **Key Modifiers**
- An **App** may draw into a **Framebuffer**
- An **App** may poll **Events** from a **Window**
- An **App** may handle **Keyboard Events**
- A **Foreground App** is an **App**
- An **Installed App** has **App files**
- An **Installed App** has one **App executable**
- An **App manifest** records an **Installed App**'s identity, descriptive metadata, and launch metadata
- **App files** belong to one **Installed App**
- An **App executable** is one of an **Installed App**'s **App files**
- An **App manifest** may point to one **App executable**
- The **Launcher** lists **Installed Apps**
- The **Launcher** launches **Apps**
- **System Settings** configures badge **Wi-Fi** settings
- **System Settings** exposes badge settings information
- The **Compositor** manages **Windows**
- A **Foreground App**'s **Task** may receive foreground priority from the compositor
- The **Compositor** manages windows and screen presentation for **Apps**
- The **Compositor** may boost a **Foreground App**'s **Task** priority
- An **App update** targets one **Installed App**
- An **App update** installs one **Revision** of one **BadgeHub Project**
- A **Firmware update** installs one **Revision** of **BadgeVMS** onto the **WHY2025 badge**
- The **OTA updater** checks for **App updates** and **Firmware updates**
- The **OTA updater** installs **App updates** and **Firmware updates**
- A **Revision** may include one **Firmware binary**
- A **Revision** may include one **Version file**
- An **App manifest** may point to one **Metadata file**
- A **Metadata file** belongs to one **App**
- The **ABI manifest** describes the **BadgeVMS** symbols available to **Apps**
- An **Interpreted App** has an **Interpreter**
- A **BadgeHub-sourced App** is an **Installed App**
- A **BadgeHub-sourced App** has one **BadgeHub Project**
- A **BadgeHub Project** publishes an **App**
- A **Default BadgeHub Project** is a **BadgeHub Project**
- A **BadgeHub Project** is published under the same **App ID** as the **App** it distributes
- A **BadgeHub Project** has one or more **Revisions**
- **BadgeHub** shares **Apps**
- BadgeVMS runs **Tasks**
- A **Process** is a **Task**
- A **Thread** is a **Task**
- The **Startup Configuration** starts and supervises **Processes**
- BadgeVMS launches an **App** as a **Process**
- A **Task** may create child **Tasks**
- A **Task** may have one parent **Task**

## Example dialogue

> **Dev:** "Should this new tool be documented as a badge binary or a BadgeVMS application?"
> **Domain expert:** "Call it an **App**. **BadgeHub** is where people share **Apps**."

## Flagged ambiguities

- "badge app", "application", "BadgeVMS application", and "badge binary" were all used to mean **App** — resolved: **App** is the canonical term.
- "native app" is too ambiguous because host binaries also run natively on the host — resolved: use **BadgeVMS-native App** only as a contrast term when discussing **Interpreted Apps**.
- "BadgeVMS" was used to mean only the runtime layer — resolved: in this repo it names the firmware platform on the **WHY2025 badge**.
- "Compositor" and generic windowing/graphics language can blur the system component with the APIs it exposes — resolved: use **Compositor** for the BadgeVMS system component that manages windows, framebuffers, events, and presentation.
- "startup app" is close to the firmware's internal `startup_app_t`, but the surfaced repo term is **Startup Configuration** for the `init.toml`-driven boot supervision model — resolved: use **Startup Configuration** for the real repo concept and "startup app" only when discussing the implementation struct.
- "host emulation", "emulator", and similar phrases were used for the Linux development mode — resolved: **Emulation** is the canonical term, defined as running an **App** against a host-side implementation of **BadgeVMS**.
- "compile for Linux", "compile for my machine", and "compile for the emulator" were used for the host-side build target — resolved: **Host build** is the canonical term for an **App** build intended to run using **Emulation**.
- "badge target" and "BadgeVMS target" were both used for the on-device Rust target — resolved: **BadgeVMS target** is the canonical term.
- "app linking", "badge app linking", and raw linker-arg talk were used for the same build policy — resolved: use **App Linking** for the shared-object entry/export policy, and name specific linker flags only when the implementation detail matters.
- "why2025-badge-app-no-std", "facade crate", and generic runtime language can blur the App dependency layer with BadgeVMS itself or with the lower-level bindings — resolved: use **App-facing facade** for the App-facing dependency role and reserve the crate name for code or config references.
- "provided runtime", "default runtime", and the allocator or panic-handler features can blur the default bundle with its parts — resolved: use **Provided runtime** for the bundle, **Provided allocator** for the allocator piece, and **Provided panic handler** for the panic-handling piece.
- "embedded graphics driver", "display driver", and compositor language can blur a Rust graphics crate with BadgeVMS windowing — resolved: use **Embedded graphics driver** for the Rust `embedded-graphics` integration layer, and keep **Compositor**, **Window**, and **Display** for the platform and hardware concepts.
- "firmware ABI", exported symbols, and the **ABI manifest** can blur the runtime surface with the file that records it — resolved: use **Firmware ABI** for the exported symbol surface itself, and **ABI manifest** for the vendored manifest that describes that surface.
- "bindings", "raw bindings", and helper crates can blur the low-level FFI surface with the higher-level App dependency layer — resolved: use **Raw bindings** for the autogenerated firmware-facing layer, and **App-facing facade** for the App-oriented dependency layer.
- "Emulation" and the "emulated layer" can blur the developer-facing execution mode with the implementation layer that makes it work — resolved: use **Emulation** for running Apps against a host-side BadgeVMS implementation, and **Emulated layer** for the host-side exported symbol layer inside this repo.
- "badge-link metadata", build-script metadata, and final linker args can blur the propagated inputs with **App Linking** itself — resolved: use **Badge-link metadata** for the entry and retain inputs passed through Cargo metadata, and **App Linking** for the overall shared-object build policy.
- "firmware symbol coverage", **ABI manifest**, and generic test coverage can blur the status report with the symbol list itself — resolved: use **Firmware Symbol Coverage** for emulation-support accounting over firmware exports, and **ABI manifest** for the exported-symbol list.
- "app_main!", `main`, and an App's Rust `run` function can blur the wrapper macro with the author-defined body — resolved: use **App entry macro** for the wrapper and **App entry function** for the function it wraps.
- BadgeVMS path devices were previously implicit — resolved: use **BadgeVMS path** for the path model and refer to concrete devices such as `FLASH0`, `SD0`, `STORAGE`, `APPS`, and `APP` when the distinction matters.
- "BadgeHub" was spelled as "Badgehub" and "badgehub" in parts of the repo — resolved: **BadgeHub** is the canonical form.
- "App" and "BadgeHub Project" were close enough to blur together — resolved: an **App** runs on the badge, while a **BadgeHub Project** publishes it through one or more **Revisions**.
- "App" and "Installed App" can blur software with badge-local installation state — resolved: use **App** for the software concept and **Installed App** for the badge-local instance represented by an **App manifest**.
- "Preinstalled App", **Core App**, and **Regular App** capture different concerns — resolved: use **Preinstalled App** for image-staged installation status, **Core App** for the system-functionality category, and **Regular App** for bundled app content.
- "compute unit", "carrier board", and the badge as a whole can blur the removable ESP32-P4 module with the rest of the hardware — resolved: use **Compute Unit** for the ESP32-P4 module, **Carrier Board** for the supporting board with ESP32-C6 connectivity, and **WHY2025 badge** for the assembled device.
- "WiFi" and "Wi-Fi" are both used around the repo — resolved: use **Wi-Fi** for the radio capability, and keep the unhyphenated spelling only when quoting source text or identifiers.
- "storage", `STORAGE:`, `FLASH0`, and `SD0` can blur BadgeVMS path devices with the underlying hardware media — resolved: use **NOR Flash** and **Micro SD Card** for the hardware, and use `FLASH0`, `SD0`, and `STORAGE:` only for the BadgeVMS devices and logical alias.
- "flash" can blur the storage chip, the firmware image, and the act of flashing — resolved: use **NOR Flash** for the storage hardware, **Firmware binary** for the update artifact, and verbs like "flash" only for the installation action.
- "display", "screen", "panel", "window", and "framebuffer" can blur the physical panel with the software surfaces and buffers — resolved: use **Display** for the physical 4-inch square panel, **Window** for the Compositor-managed UI surface, and **Framebuffer** for the backing pixel buffer attached to a **Window**.
- "frontplate", "front plate", and "frontpanel" are all used in the hardware docs for the same front-facing hardware layer — resolved: use **Frontpanel** for the component and treat the other spellings as source wording only.
- "case", "back cover", and other enclosure language can blur the optional enclosure with the core badge boards — resolved: use **Case** for the enclosure design and keep it separate from **Carrier Board**, **Frontpanel**, and **Spacer**.
- "spacer" and case-like enclosure language can blur a clearance-setting assembly part with the actual enclosure — resolved: use **Spacer** for the part that sets keyboard protrusion and display clearance, and **Case** for the enclosure.
- "keyboard", key-matrix language, and key-event language can blur the physical hardware with software input notifications — resolved: use **Keyboard** for the physical input hardware, and **Keyboard Event**, **Scancode**, **Key Code**, and **Key Modifier** for the software-side input concepts.
- "sensor", "orientation sensor", and "gas sensor" can blur the generic hardware class with the concrete badge parts — resolved: use **Sensor** for the generic hardware class, **BMI270** for motion and orientation, and **BME690** for air-quality, temperature, humidity, and pressure readings.
- "fullscreen window" and "floating window" can blur window mode with app-level state — resolved: use **Fullscreen Window** and **Floating Window** for window-level modes, and use **Foreground App** for compositor scheduling state at the app level.
- "always-on-top window", "foreground window", and fullscreen state can blur stacking order with fullscreen scheduling — resolved: use **Always-on-top Window** for a non-fullscreen window kept above other non-fullscreen windows, **Fullscreen Window** for full-screen occupancy, and **Foreground App** for the app-level scheduling state.
- "low-priority window", "foreground app", and generic background language can blur fullscreen window policy with app-level priority — resolved: use **Low-priority Window** for the fullscreen window mode that opts out of the compositor's foreground priority boost, and **Foreground App** for the app that actually receives that boost.
- "no event", empty polling, and idle state can blur a sentinel return value with an actual queued notification — resolved: use **No Event** only for the sentinel meaning that no queued **Event** was available.
- "quit event", window close, task deletion, and process exit can blur a window-queue notification with actual lifecycle control — resolved: use **Quit Event** for the notification, and say window destruction, process exit, task deletion, or thread termination only when that direct action really happens.
- "event", keyboard input, and SDL event language can blur BadgeVMS notifications with the host or SDL bridges — resolved: use **Event** for the BadgeVMS notification polled from a **Window**, and qualify SDL-specific concepts explicitly when that distinction matters.
- "event", "keyboard event", "key down event", and "key up event" can blur the generic notification with its keyboard-specific forms — resolved: use **Event** for the generic notification, **Keyboard Event** for the umbrella keyboard form, and **Key Down Event** or **Key Up Event** when the press or release distinction matters.
- "window resize event" and resizing a window can blur the notification with the resize operation itself — resolved: use **Window Resize Event** for the queue notification and say window resize only when you mean the size change operation or resulting state.
- "scancode" and "key code" can blur physical key identity with layout-resolved key meaning — resolved: use **Scancode** for the physical-key identifier carried by a **Keyboard Event**, and **Key Code** for the layout-resolved virtual-key identifier carried by that same event.
- "modifier", "mod", and SDL modifier language can blur BadgeVMS key-state flags with host bridge terminology — resolved: use **Key Modifier** for the modifier state carried by a **Keyboard Event**, and name the specific modifier when the distinction matters.
- "foreground app", "foreground window", and generic fullscreen state can blur compositor scheduling with window state — resolved: use **Foreground App** for the App currently owning the frontmost eligible **Fullscreen Window**, use **Fullscreen Window** for the window-level state, and say "foreground window" only when that specific phrasing is needed.
- "process", "thread", and "task" were close enough to blur the public BadgeVMS runtime model with its lower-level substrate — resolved: use **Task** for the generic BadgeVMS execution unit, **Process** for a **Task** created from an executable path, **Thread** for a **Task** created from a function pointer, and "FreeRTOS task" or "kernel task" only when you mean the lower-level implementation.
- "default app" can blur a **Preinstalled App** with an App seeded from a **Default BadgeHub Project** — resolved: use **Preinstalled App** for an App staged into the badge image and **Default BadgeHub Project** for the BadgeHub category with OTA seeding semantics.
- "OTA updater", `why2025_ota`, "WHY 2025 OTA updater", and "System Updates" were used at different levels for the same system App — resolved: use **OTA updater** for the App as a system component, `why2025_ota` only when you mean its **App ID**, the manifest display name only when quoting that installed name, and "System Updates" only for the UI label.
- "Launcher", `badgevms_launcher`, "BadgeVMS Launcher", and "Application Launcher" were used at different levels for the same system App — resolved: use **Launcher** for the App as a system component, `badgevms_launcher` only when you mean its **App ID**, and the longer names only when quoting the manifest or UI label.
- "System Settings", "System settings", "settings app", and `badgevms_settings` were used at different levels for the same system App — resolved: use **System Settings** for the App as a system component, `badgevms_settings` only when you mean its **App ID**, and the other forms only when quoting manifest or UI text.
- Firmware APIs call the **App ID** a `unique_identifier`, while BadgeHub JSON uses `slug` — resolved: **App ID** is the canonical docs term, with `unique_identifier` and `slug` treated as source-specific names for the same concept.
- `manifest.json`, the installed `<App ID>.json`, the optional `metadata.json`, the **App executable**, and the other files under `APPS:[<App ID>]` are easy to blur together — resolved: use **App manifest** for the BadgeVMS app record/schema, **Metadata file** for the separate App-owned file it can reference, **App executable** for the launched file, **App files** for the installed file tree as a whole, and `manifest.json` only when you mean the source-tree file.
- "update" can blur App and firmware flows that the updater handles separately — resolved: use **App update** for installing a newer **Revision** onto an existing **Installed App**, and **Firmware update** for installing a newer BadgeVMS **Revision** onto the **WHY2025 badge**.
- `badgevms.bin` is a concrete filename, not the canonical term — resolved: use **Firmware binary** for the firmware update artifact and `badgevms.bin` only when you mean that specific file.
- `version.txt` is a concrete filename, not the canonical term — resolved: use **Version file** for the version-carrying artifact and `version.txt` only when you mean that specific file.
- **App manifest** and **ABI manifest** are distinct — resolved: the **App manifest** is per-**Installed App** installation and launch metadata, while the **ABI manifest** is the BadgeVMS exported-symbol list in `symbols.yml`.
- "BadgeHub app", "downloaded app", and "default app" can blur App provenance with BadgeHub category or install state — resolved: use **BadgeHub-sourced App** when you mean an installed **App** whose **App manifest** records BadgeHub as its source.

Use **Interpreter** and **Interpreted App** only when the launch-mode distinction matters. Otherwise, prefer **App**. When you need the opposite of **Interpreted App**, say **BadgeVMS-native App**.
