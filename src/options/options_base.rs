use super::{
    check_bool::CheckBool, spin_float::SpinOptionFloat, spin_float_tunable::SpinOptionFloatTunable,
    spin_int::SpinOptionInt, OptionTrait,
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
    "Hash"            => hash:          SpinOptionInt, 32, 1, 131072;
    "Threads"         => threads:       SpinOptionInt, 1, 1, 32;
    "MoveOverhead"    => move_overhead: SpinOptionInt, 100, 0, 500;
    "MultiPV"         => multi_pv:      SpinOptionInt, 1, 1, 256;
    "UCI_ShowWDL"     => show_wdl:      CheckBool,     false;
    "UCI_AnalyseMode" => analyse_mode:  CheckBool,     false;

    "Contempt"            => contempt:             SpinOptionFloat, 5.0, -10000.0, 10000.0;
    "MaxReasonableS"      => max_reasonable_s:     SpinOptionFloat, 2.0, 0.0, 100.0;
    "ContemptMax"         => contempt_max:         SpinOptionFloat, 1000.0, 0.0, 10000.0;
    "BookExitBias"        => book_exit_bias:       SpinOptionFloat, 0.65, 0.0, 1.0;
    "DrawRateTarget"      => draw_rate_target:     SpinOptionFloat, 0.0, 0.0, 1.0;
    "DrawRateReference"   => draw_rate_reference:  SpinOptionFloat, 0.65, 0.0, 1.0;
    "DrawScore"           => draw_score:           SpinOptionFloat, 0.4, -1.0, 1.0;
    "DrawScoreOpp"        => draw_score_opp:       SpinOptionFloat, 0.5, -1.0, 1.0;
    "ContemptAttenuation" => contempt_attenuation: SpinOptionFloat, 1.0, -10.0, 10.0;

    "PolicySacBonus"         => policy_sac_bonus:         SpinOptionFloat, 0.1, 0.0, 1.0;
    "MaterialReductionBonus" => material_reduction_bonus: SpinOptionFloat, 0.25, 0.0, 10.0;

    "RootCpuctValue"      => root_cpuct_value:      SpinOptionFloatTunable, 1.01, 0.1, 5.0, 0.055, 0.002;
    "CpuctValue"          => cpuct_value:           SpinOptionFloatTunable, 0.62, 0.1, 5.0, 0.055, 0.002;
    "CpuctVisitsScale"    => cpuct_visits_scale:    SpinOptionFloatTunable, 61.95, 1.0, 512.0, 3.15, 0.002;
    "CpuctVarianceScale"  => cpuct_variance_scale:  SpinOptionFloatTunable, 0.2, 0.0, 2.0, 0.02, 0.002;
    "CpuctVarianceWeight" => cpuct_variance_weight: SpinOptionFloatTunable, 0.86, 0.0, 2.0, 0.084, 0.002;
    "ExplorationTau"      => exploration_tau:       SpinOptionFloatTunable, 0.56, 0.0, 1.0, 0.04, 0.002;
    "RootPST"             => root_pst:              SpinOptionFloatTunable, 3.25, 0.1, 10.0, 0.4, 0.002;
    "CommonPST"           => common_pst:            SpinOptionFloatTunable, 1.23, 0.1, 10.0, 0.4, 0.002;
    "HashPercentage"      => hash_percentage:       SpinOptionFloatTunable, 0.11, 0.01, 5.0, 0.025, 0.002;
);
