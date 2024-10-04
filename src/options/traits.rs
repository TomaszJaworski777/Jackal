#[allow(unused)]
pub trait OptionTrait {
    type ValueType;
    fn set(&mut self, new_value: &str);
    fn get(&self) -> Self::ValueType;
    fn print(&self, name: &str);
}

#[allow(unused)]
pub trait Tunable {}