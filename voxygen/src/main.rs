#![deny(unsafe_code)]
#![recursion_limit = "2048"]

#[cfg(all(
    target_os = "windows",
    not(feature = "tracy-memory"),
    not(feature = "hot-egui"),
    not(feature = "hot-anim"),
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg_attr(feature = "tracy-memory", global_allocator)]
#[cfg(feature = "tracy-memory")]
static GLOBAL: common_base::tracy_client::ProfiledAllocator<std::alloc::System> =
    common_base::tracy_client::ProfiledAllocator::new(std::alloc::System, 128);

#[cfg(not(target_os = "android"))]
fn main() { veloren_voxygen::app::run(); }

// Android loads the library and invokes `android_main` instead of running this
// desktop binary entry point. Keeping a no-op `main` also lets Cargo inspect
// all targets for that triple without inventing a second startup path.
#[cfg(target_os = "android")]
fn main() {}
