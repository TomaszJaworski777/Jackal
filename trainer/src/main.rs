mod policy_train;
mod value_train;

fn main() {
    #[cfg(feature = "policy_trainer")] {
        policy_train::run();
    }

    #[cfg(feature = "value_trainer")] {
        value_train::run();
    }
}
