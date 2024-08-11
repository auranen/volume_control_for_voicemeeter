#![windows_subsystem = "windows"]

mod voicemeeter;
mod windows_eco_mode;
mod windows_volume;

use std::sync::Arc;
use anyhow::{Context};
use lerp::Lerp;
use smol::future::FutureExt;
use tokio::sync::Notify;
use windows::Win32::Foundation::BOOL;

fn print_error(err: impl std::fmt::Display) {
    println!("{err}");
}

fn handle_the_error(err: impl std::fmt::Display) {
    print_error(err);

    // println!("\nPress ENTER to continue...");
    // std::io::stdin().lines().next();
}

fn main() {
    println!("Started");

    smol::block_on(listen()).unwrap_or_else(handle_the_error);

    println!("Exiting safely");
}

async fn listen() -> anyhow::Result<()> {
    // Initialize Win32's COM libray. Things break without this step.
    windows_volume::initialize_com().context("COM initialization failed")?;

    println!("COM initialized");

    windows_eco_mode::set_eco_mode_for_current_process()
        .unwrap_or_else(|err| println!("Failed to set Process mode to Eco: {}", err));

    let observer = windows_volume::VolumeObserver::from_device_name("voicemeeter input")?;
    let mut windows_volume_stream = observer.subscribe();

    let link = voicemeeter::Link::new().context("Failed to register with Voicemeeter")?;

    let exit_signal = Arc::new(Notify::new());

    {
        let exit_signal = exit_signal.to_owned();

        std::thread::Builder::new()
            .name("Tray icon event loop".into())
            .spawn(move || {
                (|| -> anyhow::Result<()> {
                    let tray_menu = tray_icon::menu::Menu::new();
                    tray_menu
                        .append(&tray_icon::menu::MenuItem::new("Exit", true, None))
                        .context("Couldn't add item to the tray menu")?;

                    let _tray_icon = tray_icon::TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu))
                        .with_tooltip(format!(
                            "{}  v{}",
                            env!("CARGO_PKG_NAME"),
                            env!("CARGO_PKG_VERSION")
                        ))
                        .with_icon(
                            tray_icon::Icon::from_resource(1, None)
                                .context("Failed to make icon")?,
                        )
                        .build()
                        .unwrap();

                    // let tray_channel = tray_icon::TrayIconEvent::receiver();
                    let menu_channel = tray_icon::menu::MenuEvent::receiver();

                    use winit::event_loop::EventLoopBuilder;
                    use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
                    use winit::platform::windows::EventLoopBuilderExtWindows;

                    EventLoopBuilder::new()
                        .with_any_thread(true)
                        .build()?
                        .run_on_demand(|_, elwt| {
                            // if let Ok(event) = tray_channel.try_recv() {
                            //     println!("{event:?}");
                            // }
                            if menu_channel.try_recv().is_ok() {
                                elwt.exit();
                            }
                        })?;

                    Ok(())
                })()
                .unwrap_or_else(print_error);

                exit_signal.notify_one();
            })?;
    }

    (async {
        tokio_graceful::default_signal().await;
        anyhow::Ok(())
    })
    .or(async {
        exit_signal.notified().await;
        anyhow::Ok(())
    })
    .or(async {
        let mut previous_vm_edition = ::voicemeeter::types::VoicemeeterApplication::None;
        let mut vm_gain_parameter = None;
        let mut vm_mute_parameter = None;

        println!("Listening for volume changes");

        loop {
            windows_volume_stream
                .changed()
                .await
                .context("windows_volume_stream error 🤨")?;

            // linear position of the volume slider from 0.0 to 1.0
            let (Some(volume_slider_position), Some(mute)) = ({ *windows_volume_stream.borrow() }) else {
                continue;
            };

            let mute_bool = mute != BOOL(0);

            println!("Changed to {volume_slider_position}, mute {mute_bool}");

            link.wait_for_connection().await;

            {
                let mut remote = link.inner.remote.lock().await;

                let vm_edition = {
                    remote.update_program()?;
                    remote.program
                };

                if vm_edition != previous_vm_edition {
                    previous_vm_edition = vm_edition;
                    match voicemeeter::Link::virtual_inputs(&remote).next() {
                        Some(strip) => {
                            vm_gain_parameter = Some(link.gain_parameter_of(&strip));
                            vm_mute_parameter = Some(link.mute_parameter_of(&strip));
                        }
                        _ => {
                            print_error("There should absolutely be at least one \
                            Virtual Input in any Voicemeeter edition \
                            but it's not there 🤷.")
                        }
                    }
                }
            };
            let vm_gain_parameter = vm_gain_parameter.as_ref().unwrap();
            let vm_mute_parameter = vm_mute_parameter.as_ref().unwrap();

            let gain = (-60.0).lerp(0.0, volume_slider_position);

            vm_mute_parameter
                .set(mute.0 as f32)
                .await
                .context("Couldn't set mute value")
                .unwrap_or_else(print_error);
            vm_gain_parameter
                .set(gain)
                .await
                .context("Couldn't set slider value")
                .unwrap_or_else(print_error)
        }
    })
    .await
    .unwrap_or_else(print_error);

    Ok(())
}
