mod policy_train;
mod value_train;
mod interleave;

fn main() {
    #[cfg(feature = "policy_trainer")] {
        policy_train::run();
    }

    #[cfg(feature = "value_trainer")] {
        value_train::run();
    }

    #[cfg(feature = "policy_interleave")] {
        use crate::interleave::interleave;

        _ = interleave();
    }
}
