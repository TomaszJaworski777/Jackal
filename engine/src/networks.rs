mod value_network;
mod policy_network;
mod layers;
mod inputs;

use crate::networks::value_network::ValueNetwork;
use crate::networks::policy_network::PolicyNetwork;

#[allow(non_upper_case_globals)]
pub static ValueNetwork: ValueNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/v600cos3072WDL-TD-OB-007b-Q.network"))
};

#[allow(non_upper_case_globals)]
pub static PolicyNetwork: PolicyNetwork = unsafe {
    std::mem::transmute(*include_bytes!("../../resources/networks/p200exp1024see006.network"))
};