mod common;

use khronos_egl as egl;

fn main() {
    let egl = unsafe {
        egl::DynamicInstance::<egl::EGL1_5>::load_required().expect("unable to load libEGL.so.1")
    };
    common::run(&egl);
}
