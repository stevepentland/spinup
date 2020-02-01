mod standard;
mod testing;

cfg_if::cfg_if! {
    if #[cfg(test)] {
        pub use testing::{internal_runner, get_root, passed_command, passed_args, reset, called_root};
    } else {
        pub use standard::{internal_runner, get_root};
    }
}
