use super::{
    check_bool::CheckBool, spin_float_tunable::SpinOptionFloatTunable, spin_int::SpinOptionInt,
    OptionTrait,
};

macro_rules! create_option_structs {
    ($($option_name:expr => $name:ident: $type:ty, $($values:expr), +;)*) => {
        pub struct EngineOptions {
            $($name: $type,)*
        }

        impl Default for EngineOptions {
            fn default() -> Self {
                Self::new()
            }
        }

        #[allow(unused)]
        impl EngineOptions {
            pub fn new() -> Self {
                Self {
                    $($name: <$type>::new($($values),+),)*
                }
            }

            pub fn set(&mut self, key: &str, new_value: &str) {
                match key {
                    $($option_name => Self::update_option(&mut self.$name, new_value),)*
                    _ => println!("Option {} doesn't exist.", key),
                }
            }

            pub fn print(&self) {
                $(
                    self.$name.print($option_name);
                )*
            }

            #[inline]
            $(
                pub fn $name(&self) -> <$type as OptionTrait>::ValueType {
                    self.$name.get()
                }
            )*

            fn update_option<T: OptionTrait>(option: &mut T, new_value: &str) {
                option.set(new_value);
            }
        }
    };
}

create_option_structs!(
    "Hash"             => hash:               SpinOptionInt,          32, 1, 131072;
    "Threads"          => threads:            SpinOptionInt,          1, 1, 1024;
    "MoveOverhead"     => move_overhead:      SpinOptionInt,          10, 0, 500;
    "UCI_ShowWDL"      => show_wdl:           CheckBool,              false;
    "RootCpuctValue"   => root_cpuct_value:   SpinOptionFloatTunable, 0.96, 0.1, 5.0, 0.055, 0.002;
    "CpuctValue"       => cpuct_value:        SpinOptionFloatTunable, 0.64, 0.1, 5.0, 0.055, 0.002;
    "CpuctVisitsScale" => cpuct_visits_scale: SpinOptionFloatTunable, 64.0, 1.0, 512.0, 3.15, 0.002;
    "RootPST"          => root_pst:           SpinOptionFloatTunable, 3.5, 0.1, 10.0, 0.4, 0.002;
    "ExplorationTau"   => exploration_tau:    SpinOptionFloatTunable, 0.58, 0.0, 1.0, 0.04, 0.002;
    "HashPercentage"   => hash_percentage:    SpinOptionFloatTunable, 0.011, 0.001, 0.5, 0.0025, 0.002;
);
