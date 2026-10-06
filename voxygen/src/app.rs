//! Shared Voxygen bootstrap for desktop builds and the Android NativeActivity.

use clap::Parser;
use i18n::{self, LocalizationHandle};
use crate::{
    GlobalState,
    audio::AudioFrontend,
    cli,
    error::Error,
    panic_handler,
    profile::Profile,
    render::RenderError,
    run,
    scene::terrain::SpriteRenderContext,
    settings::{AudioOutput, Settings, get_fps},
    window::Window,
};
#[cfg(feature = "singleplayer")]
use crate::singleplayer::SingleplayerState;

use chrono::Utc;
use common::{clock::Clock, consts::MIN_RECOMMENDED_TOKIO_THREADS};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::runtime::Builder;
#[cfg(target_os = "android")]
use std::{
    ffi::CString,
    io::{self, Read},
    path::{Component, Path},
};
use tracing::{info, warn};
#[cfg(feature = "egui-ui")]
use crate::ui::egui::EguiState;
use wgpu::{Backends, Instance};

pub fn run(
    #[cfg(target_os = "android")] android_app: &winit::platform::android::activity::AndroidApp,
) {
    #[cfg(target_os = "android")]
    initialize_android_storage(android_app);
    #[cfg(target_os = "android")]
    crate::android_log::stage("android storage ready");

    // Process CLI arguments. On Android the process argv comes from the system
    // and carries no user arguments; parse a fixed argv so unexpected entries
    // can never abort startup with a clap usage error.
    #[cfg(target_os = "android")]
    let args = cli::Args::try_parse_from(["voxygen"]).expect("empty argv must parse");
    #[cfg(not(target_os = "android"))]
    let args = cli::Args::parse();

    if let Some(command) = args.command {
        match command {
            cli::Commands::ListWgpuBackends => {
                #[cfg(target_os = "windows")]
                let backends = &["opengl", "dx12", "vulkan"];
                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                let backends = &["opengl", "vulkan"];
                #[cfg(target_os = "macos")]
                let backends = &["metal"];

                for backend in backends {
                    println!("{backend}");
                }
                return;
            },
            cli::Commands::ListWgpuDevices => {
                let adapters = Instance::new(&wgpu::InstanceDescriptor::from_env_or_default())
                    .enumerate_adapters(Backends::default());
                for adapter in adapters {
                    println!("{}", adapter.get_info().name);
                }
                return;
            },
        }
    }

    #[cfg(target_os = "android")]
    crate::android_log::stage("cli args parsed");

    #[cfg(feature = "tracy")]
    common_base::tracy_client::Client::start();

    let userdata_dir = common_base::userdata_dir();

    // Determine where Voxygen's logs should go
    // Choose a path to store the logs by the following order:
    //  - The VOXYGEN_LOGS environment variable
    //  - The <userdata>/voxygen/logs
    let logs_dir = std::env::var_os("VOXYGEN_LOGS")
        .map(PathBuf::from)
        .unwrap_or_else(|| userdata_dir.join("voxygen").join("logs"));

    // Init logging and hold the guards.
    let now = Utc::now();
    let log_filename = format!("{}_voxygen.log", now.format("%Y-%m-%d"));
    let _guards = common_frontend::init_stdout(Some((&logs_dir, &log_filename)));

    // Re-run userdata selection so any warnings will be logged
    common_base::userdata_dir();

    info!("Using userdata dir at: {}", userdata_dir.display());

    // Determine Voxygen's config directory either by env var or placed in veloren's
    // userdata folder
    let config_dir = std::env::var_os("VOXYGEN_CONFIG")
        .map(PathBuf::from)
        .and_then(|path| {
            if path.exists() {
                Some(path)
            } else {
                warn!(?path, "VOXYGEN_CONFIG points to invalid path.");
                None
            }
        })
        .unwrap_or_else(|| userdata_dir.join("voxygen"));
    info!("Using config dir at: {}", config_dir.display());

    // Load the settings
    let mut settings = Settings::load(&config_dir);
    settings.display_warnings();

    panic_handler::set_panic_hook(log_filename, logs_dir);

    // Setup tokio runtime
    // TODO: evaluate std::thread::available_concurrency as a num_cpus replacement
    let cores = num_cpus::get();
    let tokio_runtime = Arc::new(
        Builder::new_multi_thread()
            .enable_all()
            .worker_threads((cores / 4).max(MIN_RECOMMENDED_TOKIO_THREADS))
            .thread_name_fn(|| {
                static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                format!("tokio-voxygen-{}", id)
            })
            .build()
            .unwrap(),
    );

    // Initialise watcher for animation hot-reloading
    #[cfg(feature = "hot-anim")]
    {
        anim::init();
    }

    // Initialise watcher for egui hot-reloading
    #[cfg(feature = "hot-egui")]
    {
        voxygen_egui::init();
    }

    // Setup audio
    let mut audio = match settings.audio.output {
        AudioOutput::Off => AudioFrontend::no_audio(),
        AudioOutput::Automatic => AudioFrontend::new(
            settings.audio.num_sfx_channels,
            settings.audio.num_ui_channels,
            settings.audio.subtitles,
            // settings.audio.combat_music_enabled,
            false, // We're disabling combat music for now
            settings.audio.buffer_size.samples,
            settings.audio.sample_rate,
        ),
    };

    audio.set_master_volume(settings.audio.master_volume.get_checked());
    audio.set_music_volume(settings.audio.music_volume.get_checked());
    audio.set_sfx_volume(settings.audio.sfx_volume.get_checked());
    audio.set_instrument_volume(settings.audio.instrument_volume.get_checked());
    audio.set_ambience_volume(settings.audio.ambience_volume.get_checked());
    audio.set_music_spacing(settings.audio.music_spacing);

    // Load the profile.
    let profile = Profile::load(&config_dir);

    let mut i18n =
        LocalizationHandle::load(&settings.language.selected_language).unwrap_or_else(|error| {
            let selected_language = &settings.language.selected_language;
            warn!(
                ?error,
                ?selected_language,
                "Impossible to load language: change to the default language (English) instead.",
            );
            i18n::REFERENCE_LANG.clone_into(&mut settings.language.selected_language);
            LocalizationHandle::load_expect(&settings.language.selected_language)
        });
    i18n.set_english_fallback(settings.language.use_english_fallback);
    #[cfg(target_os = "android")]
    crate::android_log::stage("i18n ready");

    // Create window
    #[cfg(target_os = "android")]
    crate::android_log::stage("creating window");
    #[cfg(target_os = "android")]
    let window_result = Window::new(&settings, &tokio_runtime, android_app);
    #[cfg(not(target_os = "android"))]
    let window_result = Window::new(&settings, &tokio_runtime);
    let (mut window, event_loop) = match window_result {
        Ok(ok) => ok,
        // Custom panic message when a graphics backend could not be found
        Err(Error::RenderError(RenderError::CouldNotFindAdapter)) => {
            #[cfg(target_os = "windows")]
            const POTENTIAL_FIX: &str =
                " Updating the graphics drivers on this system may resolve this issue.";
            #[cfg(target_os = "macos")]
            const POTENTIAL_FIX: &str = "";
            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            const POTENTIAL_FIX: &str =
                " Installing or updating vulkan drivers may resolve this issue.";

            panic!(
                "Failed to select a rendering backend! No compatible backends were found. We \
                 currently support vulkan, metal, dx12, and opengl.{} If the issue persists, \
                 please include the operating system and GPU details in your bug report to help \
                 us identify the cause.",
                POTENTIAL_FIX
            );
        },
        Err(error) => panic!("Failed to create window!: {:?}", error),
    };

    #[cfg(target_os = "android")]
    crate::android_log::stage("window created");

    let clipboard = crate::ui::ice::Clipboard::connect(window.window());

    let lazy_init = SpriteRenderContext::new(window.renderer_mut());

    #[cfg(feature = "egui-ui")]
    let egui_state = EguiState::new(&window);

    #[cfg(feature = "discord")]
    let discord = if settings.networking.enable_discord_integration {
        crate::discord::Discord::start(&tokio_runtime)
    } else {
        crate::discord::Discord::Inactive
    };

    let global_state = GlobalState {
        userdata_dir,
        config_dir,
        audio,
        profile,
        window,
        tokio_runtime,
        #[cfg(feature = "egui-ui")]
        egui_state,
        lazy_init,
        clock: Clock::new(std::time::Duration::from_secs_f64(
            1.0 / get_fps(settings.graphics.max_fps) as f64,
        )),
        settings,
        info_message: None,
        #[cfg(feature = "singleplayer")]
        singleplayer: SingleplayerState::None,
        i18n,
        clipboard,
        clear_shadows_next_frame: false,
        #[cfg(feature = "discord")]
        discord,
        args: args.clone(),
    };

    #[cfg(target_os = "android")]
    crate::android_log::stage("entering main loop");

    run::run(global_state, event_loop).unwrap();
}

#[cfg(target_os = "android")]
fn initialize_android_storage(app: &winit::platform::android::activity::AndroidApp) {
    const APK_ASSET_ROOT: &str = "veloren-assets";
    const ASSET_INDEX: &str = "veloren-assets-index.txt";
    const ASSET_VERSION: &str = "veloren-assets-version.txt";

    fn read_asset(manager: &ndk::asset::AssetManager, path: &str) -> io::Result<Vec<u8>> {
        let c_path = CString::new(path)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in asset path"))?;
        let mut asset = manager.open(&c_path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing packaged Android asset {path}"),
            )
        })?;
        let mut contents = Vec::with_capacity(asset.length());
        asset.read_to_end(&mut contents)?;
        Ok(contents)
    }

    let app_data_dir = app
        .internal_data_path()
        .expect("Android did not provide the app-private data directory");
    std::fs::create_dir_all(&app_data_dir)
        .expect("failed to create Android app-private data directory");

    let manager = app.asset_manager();
    let version = String::from_utf8(
        read_asset(&manager, &format!("{APK_ASSET_ROOT}/{ASSET_VERSION}"))
            .expect("Android APK is missing its generated Veloren asset version"),
    )
    .expect("Veloren Android asset version is not UTF-8");
    let version = version.trim();
    let manifest = String::from_utf8(
        read_asset(&manager, &format!("{APK_ASSET_ROOT}/{ASSET_INDEX}"))
            .expect("Android APK is missing its generated Veloren asset index"),
    )
    .expect("Veloren Android asset index is not UTF-8");

    let assets_dir = app_data_dir.join("assets");
    let canary_path = assets_dir.join("common/canary.canary");
    let installed_version = std::fs::read_to_string(assets_dir.join(ASSET_VERSION)).ok();
    let assets_are_current = installed_version
        .as_deref()
        .is_some_and(|installed| installed.trim() == version)
        && std::fs::read_to_string(&canary_path)
            .is_ok_and(|canary| canary.starts_with("VELOREN_CANARY_MAGIC"));

    if !assets_are_current {
        crate::android_log::stage("extracting bundled assets (first launch, may take minutes)");
        // Rebuild into a staging directory so a killed process never leaves a
        // marker that would make a partial extraction look complete. Remove the
        // previous copy first to avoid requiring another ~450 MB of free space.
        let staging_dir = app_data_dir.join(".veloren-assets-staging");
        if staging_dir.exists() {
            std::fs::remove_dir_all(&staging_dir)
                .expect("failed to remove an incomplete Android asset extraction");
        }
        if assets_dir.exists() {
            std::fs::remove_dir_all(&assets_dir)
                .expect("failed to remove outdated Android assets");
        }
        std::fs::create_dir_all(&staging_dir)
            .expect("failed to create Android asset extraction directory");

        for relative_path in manifest.lines().filter(|line| !line.is_empty()) {
            let relative = Path::new(relative_path);
            if relative.is_absolute()
                || relative.components().count() == 0
                || relative
                    .components()
                    .any(|component| !matches!(component, Component::Normal(_)))
                || relative_path == ASSET_INDEX
                || relative_path == ASSET_VERSION
            {
                panic!("invalid path in packaged Veloren asset index: {relative_path:?}");
            }

            let apk_path = format!("{APK_ASSET_ROOT}/{relative_path}");
            let c_path = CString::new(apk_path.as_str())
                .expect("NUL in generated Veloren Android asset path");
            let mut asset = manager
                .open(&c_path)
                .unwrap_or_else(|| panic!("missing Veloren asset in APK: {apk_path}"));
            let destination = staging_dir.join(relative);
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)
                    .unwrap_or_else(|error| panic!("failed to create {}: {error}", parent.display()));
            }
            let mut output = std::fs::File::create(&destination)
                .unwrap_or_else(|error| panic!("failed to create {}: {error}", destination.display()));
            io::copy(&mut asset, &mut output)
                .unwrap_or_else(|error| panic!("failed to extract {}: {error}", destination.display()));
        }

        let extracted_canary = std::fs::read_to_string(staging_dir.join("common/canary.canary"))
            .expect("Veloren asset bundle does not contain the canary file");
        if !extracted_canary.starts_with("VELOREN_CANARY_MAGIC") {
            panic!("The Android Veloren asset bundle has an invalid canary; check Git LFS.");
        }
        std::fs::write(staging_dir.join(ASSET_VERSION), version)
            .expect("failed to write Android asset version marker");
        std::fs::rename(&staging_dir, &assets_dir)
            .expect("failed to install extracted Android assets");
    }

    let userdata_dir = app_data_dir.join("userdata");
    std::fs::create_dir_all(&userdata_dir)
        .expect("failed to create Android user data directory");
    common_base::set_android_userdata_dir(userdata_dir);
    common::assets::set_android_assets_path(assets_dir);
}
