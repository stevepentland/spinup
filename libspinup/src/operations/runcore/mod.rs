cfg_if::cfg_if! {
    if #[cfg(test)] {
        mod testing;

        pub use testing::testing_runner as internal_runner;
        pub use testing::{passed_command, passed_args, reset};
    } else {
        mod standard;
        pub use standard::standard_runner as internal_runner;
    }
}
