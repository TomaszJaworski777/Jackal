macro_rules! create_option_structs {
    ($($name:ident: $type:ty => $new_expr:expr, $option_name:expr),* $(,)?) => {
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
                    $($name: $new_expr,)*
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
    move_overhead: SpinOptionInt => SpinOptionInt::new(10, 0, 500, 1.0, 0.0), "MoveOverhead",
    hash: SpinOptionInt => SpinOptionInt::new(16, 1, 65536, 1.0, 0.0), "Hash",
    cpuct_value: SpinOptionFloatTunable => SpinOptionFloatTunable::new(1.41, 0.1, 5.0, 1.0, 0.0), "CpuctValue",
);

#[allow(unused)]
pub trait OptionTrait {
    type ValueType;
    fn set(&mut self, new_value: &str);
    fn get(&self) -> Self::ValueType;
    fn print(&self, name: &str);
}

#[allow(unused)]
pub trait Tunable { }

#[allow(unused)]
pub struct SpinOptionInt {
    value: i64,
    default: i64,
    min: i64,
    max: i64,
    step: f32,
    r: f32,
}

impl SpinOptionInt {
    fn new(value: i64, min: i64, max: i64, step: f32, r: f32) -> Self {
        Self {
            value,
            default: value,
            min,
            max,
            step,
            r,
        }
    }

    fn set_value(&mut self, new_value: i64) {
        if new_value >= self.min && new_value <= self.max {
            self.value = new_value;
        } else {
            println!("Value out of range.");
        }
    }

    #[inline]
    fn get(&self) -> i64 {
        self.value
    }
}

impl OptionTrait for SpinOptionInt {
    type ValueType = i64;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<i64>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    #[inline]
    fn get(&self) -> i64 {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type spin default {:?} min {:?} max {:?}",
            name, self.default, self.min, self.max
        );
    }
}

#[allow(unused)]
pub struct SpinOptionFloat {
    value: f32,
    default: f32,
    min: f32,
    max: f32
}

#[allow(unused)]
impl SpinOptionFloat {
    fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            default: value,
            min,
            max
        }
    }

    fn set_value(&mut self, new_value: i32) {
        let adjusted = new_value as f32 / 100.0;
        if adjusted >= self.min && adjusted <= self.max {
            self.value = adjusted;
        } else {
            println!("Value out of range.");
        }
    }

    #[inline]
    fn get(&self) -> f32 {
        self.value
    }
}

impl OptionTrait for SpinOptionFloat {
    type ValueType = f32;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<i32>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    #[inline]
    fn get(&self) -> f32 {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type spin default {:?} min {:?} max {:?}",
            name,
            (self.default * 100.0) as i32,
            (self.min * 100.0) as i32,
            (self.max * 100.0) as i32
        );
    }
}

#[allow(unused)]
pub struct SpinOptionIntTunable {
    value: i64,
    default: i64,
    min: i64,
    max: i64,
    step: f32,
    r: f32,
}

#[allow(unused)]
impl SpinOptionIntTunable {
    fn new(value: i64, min: i64, max: i64, step: f32, r: f32) -> Self {
        Self {
            value,
            default: value,
            min,
            max,
            step,
            r,
        }
    }

    fn set_value(&mut self, new_value: i64) {
        if new_value >= self.min && new_value <= self.max {
            self.value = new_value;
        } else {
            println!("Value out of range.");
        }
    }

    #[inline]
    fn get(&self) -> i64 {
        self.value
    }
}

impl OptionTrait for SpinOptionIntTunable {
    type ValueType = i64;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<i64>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    #[inline]
    fn get(&self) -> i64 {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type spin default {:?} min {:?} max {:?}",
            name, self.default, self.min, self.max
        );
    }
}

impl Tunable for SpinOptionIntTunable {
    
}

#[allow(unused)]
pub struct SpinOptionFloatTunable {
    value: f32,
    default: f32,
    min: f32,
    max: f32,
    step: f32,
    r: f32,
}

#[allow(unused)]
impl SpinOptionFloatTunable {
    fn new(value: f32, min: f32, max: f32, step: f32, r: f32) -> Self {
        Self {
            value,
            default: value,
            min,
            max,
            step,
            r,
        }
    }

    fn set_value(&mut self, new_value: i32) {
        let adjusted = new_value as f32 / 100.0;
        if adjusted >= self.min && adjusted <= self.max {
            self.value = adjusted;
        } else {
            println!("Value out of range.");
        }
    }

    #[inline]
    fn get(&self) -> f32 {
        self.value
    }
}

impl OptionTrait for SpinOptionFloatTunable {
    type ValueType = f32;

    fn set(&mut self, new_value: &str) {
        if let Ok(parsed_value) = new_value.parse::<i32>() {
            self.set_value(parsed_value);
        } else {
            println!("Invalid value for option.");
        }
    }

    #[inline]
    fn get(&self) -> f32 {
        self.get()
    }

    fn print(&self, name: &str) {
        println!(
            "option name {} type spin default {:?} min {:?} max {:?}",
            name,
            (self.default * 100.0) as i32,
            (self.min * 100.0) as i32,
            (self.max * 100.0) as i32
        );
    }
}

impl Tunable for SpinOptionFloatTunable {
    
}
