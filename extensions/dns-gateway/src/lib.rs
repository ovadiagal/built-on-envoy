// Copyright Built On Envoy
// SPDX-License-Identifier: Apache-2.0
// The full text of the Apache license is available in the LICENSE file at
// the root of the repo.

use envoy_proxy_dynamic_modules_rust_sdk::*;

mod dns_gateway;

// TODO(govadia): replace with `declare_all_init_functions` once the SDK is bumped to 1.38.0
// in which the macro supports any combination of filter types.
#[no_mangle]
pub extern "C" fn envoy_dynamic_module_on_program_init() -> *const std::os::raw::c_char {
    envoy_proxy_dynamic_modules_rust_sdk::NEW_NETWORK_FILTER_CONFIG_FUNCTION
        .get_or_init(|| new_network_filter_config_fn);
    envoy_proxy_dynamic_modules_rust_sdk::NEW_UDP_LISTENER_FILTER_CONFIG_FUNCTION
        .get_or_init(|| new_udp_listener_filter_config_fn);
    abi::envoy_dynamic_modules_abi_version.as_ptr() as *const std::os::raw::c_char
}

fn new_network_filter_config_fn<EC: EnvoyNetworkFilterConfig, ENF: EnvoyNetworkFilter>(
    _envoy_filter_config: &mut EC,
    filter_name: &str,
    filter_config: &[u8],
) -> Option<Box<dyn NetworkFilterConfig<ENF>>> {
    match filter_name {
        "cache_lookup" => Some(Box::new(
            dns_gateway::cache_lookup::CacheLookupFilterConfig::new(filter_config),
        )),
        _ => panic!("Unknown network filter name: {filter_name}"),
    }
}

fn new_udp_listener_filter_config_fn<
    EC: EnvoyUdpListenerFilterConfig,
    ELF: EnvoyUdpListenerFilter,
>(
    _envoy_filter_config: &mut EC,
    filter_name: &str,
    filter_config: &[u8],
) -> Option<Box<dyn UdpListenerFilterConfig<ELF>>> {
    match filter_name {
        "dns_gateway" => {
            let config = dns_gateway::DnsGatewayFilterConfig::new(filter_config)?;
            Some(Box::new(config))
        }
        _ => panic!("Unknown UDP listener filter name: {filter_name}"),
    }
}
