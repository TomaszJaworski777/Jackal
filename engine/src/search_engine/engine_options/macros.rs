#[macro_export]
macro_rules! create_options {
    (
        $name:ident {
            Options {
                $(
                    [$option_key:literal] $option:ident : $option_ty:ty =>
                        $option_default:expr $(, $option_min:expr, $option_max:expr)?;
                )+
            }
            Tunables {
                $(
                    $(#[$meta:meta])*
                    $tunable:ident : $tunable_ty:ty =>
                        $tunable_default:expr,
                        $tunable_min:expr,
                        $tunable_max:expr,
                        $tunable_c:expr,
                        $tunable_r:expr;
                )+
            }
        }
    ) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $($option: $option_ty,)+
            $($tunable: $tunable_ty,)+
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    $($option: $option_default,)+
                    $($tunable: $tunable_default,)+
                }
            }

            $(
            pub const fn $option(&self) -> $option_ty {
                self.$option
            }
            )+

            $(
            $(#[$meta])*
            pub const fn $tunable(&self) -> $tunable_ty {
                self.$tunable
            }
            )+

            pub fn set_option(&mut self, name: &str, value: &str) -> Result<(), String> {
                $(
                if name.eq_ignore_ascii_case($option_key) {
                    match value.parse::<$option_ty>() {
                        Ok(new_value) => {
                            $(
                            if new_value < $option_min || new_value > $option_max {
                                return Err(format!("Value out of range for {}", name));
                            }
                            )?

                            if self.$option == new_value {
                                return Err(format!("Value of {} is already {}", name, new_value));
                            }

                            self.$option = new_value;
                            return Ok(());
                        }
                        Err(_) => return Err(format!("Incorrect param type for {}", name)),
                    }
                } else
                )+

                $(
                if name.eq_ignore_ascii_case(stringify!($tunable)) {
                    match value.parse::<$tunable_ty>() {
                        Ok(new_value) => {
                            if new_value < $tunable_min || new_value > $tunable_max {
                                return Err(format!("Value out of range for {}", name));
                            }

                            if self.$tunable == new_value {
                                return Err(format!("Value of {} is already {}", name, new_value));
                            }

                            self.$tunable = new_value;
                            return Ok(());
                        }
                        Err(_) => return Err(format!("Incorrect param type for {}", name)),
                    }
                } else
                )+

                {
                    Err(format!("Unknown option '{}'", name))
                }
            }


            pub fn print_options(&self) {
                $(
                {
                    let uci_type = match stringify!($option_ty) {
                        "bool" => "check",
                        "i64"  => "spin",
                        _      => "string",
                    };

                    let mut default_str = $option_default.to_string();
                    if default_str.is_empty() {
                        default_str = "<empty>".to_string();
                    }

                    print!(
                        "option name {} type {} default {}",
                        $option_key, uci_type, default_str
                    );

                    $( print!(" min {} max {}", $option_min, $option_max); )?
                    println!();
                }
                )+

                #[cfg(feature = "dev")]
                {
                    $(
                    {
                        let uci_type = match stringify!($tunable_ty) {
                            "bool" => "check",
                            "i64"  => "spin",
                            _      => "string",
                        };

                        let mut default_str = $tunable_default.to_string();
                        if default_str.is_empty() {
                            default_str = "<empty>".to_string();
                        }

                        print!(
                            "option name {} type {} default {}",
                            stringify!($tunable), uci_type, default_str
                        );

                        print!(" min {} max {}", $tunable_min, $tunable_max);
                        println!();
                    }
                    )+
                }
            }

            pub fn print_tunables(&self) {
                $(
                {
                    let kind = if stringify!($tunable_ty) == "i64" { "int" } else { "float" };
                    println!(
                        "{}, {}, {}, {}, {}, {}, {}",
                        stringify!($tunable), kind,
                        self.$tunable, $tunable_min, $tunable_max,
                        $tunable_c, $tunable_r
                    );
                }
                )+
            }
        }
    };
}
