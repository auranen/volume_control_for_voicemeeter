<p align="center">
    <img width="150" 
        src="media/icon.svg" 
        alt="logo">
</p>

<h1 align="center">volume_control_for_voicemeeter</h1>

A simple and tiny tray application that syncs the Windows volume of the `VoiceMeeter Input` device to the volume of the first virtual input strip (which corresponds to that `VoiceMeeter Input` device).

This allows the built-in Windows volume slider 🎚️, any mixer apps' sliders 🎚️ as well as gestures and keyboard volume keys ⌨️ to control the volume of Voicemeeter.

Written in Rust  🦀

### Features

* Instead of constantly polling the volume slider for changes, this app uses the built-in Windows' `IAudioEndpointVolumeCallback` interface, thanks to which, the program is **completely** idle and not using **any** CPU resources when the volume isn't being changed.
* Tiny footprint (**.77mb** exe, **0%** CPU, **2.4mb** RAM).
* The application properly unregisters with Voicemeeter's API when exiting, which prevents leaking resources and causing visual weirdness in the Voicemeeter's GUI.
* Tray icon visually consistant with that of Voicemeeter.

## Download

Simply download the **exe** from the latest of [Releases](https://github.com/not-holar/volume_control_for_voicemeeter/releases)

## Installation

This application is portable and lives entirely within it's .exe

To make the application start with Windows:

1. Open **File Explorer**
2. Paste **`shell:startup`** into the Address bar and press **Enter**
3. Put **volume_control_for_voicemeeter.exe** (or a shortcut to it) in the folder that opened
4. Launch the **exe** if it wasn't already running

## Contributing

Please report any bugs you encounter as well as suggest improvements by using [Github issues](https://github.com/not-holar/volume_control_for_voicemeeter/issues).

Feel free to file Pull requests with your contributions

## Troubleshooting

To see the log of what is going on, exit the application and relaunch it inside of a terminal
