use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use ip_network::IpNetwork;
use ip_network_table::IpNetworkTable;

/// A cheaply-clonable set of blocked networks
#[derive(Clone, Default)]
pub struct BlockedNetworks {
    /// A set of IP networks to be blocked
    networks: Arc<IpNetworkTable<()>>,
    /// If true, block all non-globally-routable networks, in addition to `networks`
    block_private: bool,
}

impl BlockedNetworks {
    pub(crate) fn new(
        block_networks: impl AsRef<[IpNetwork]>,
        block_private_networks: bool,
    ) -> Self {
        let mut networks = IpNetworkTable::new();
        for network in IpNetwork::collapse_addresses(block_networks.as_ref()) {
            // Omit redundant blocked_networks if block_private_networks = true
            if block_private_networks && !network.is_global() {
                continue;
            }
            networks.insert(network, ());
        }
        Self {
            networks: networks.into(),
            block_private: block_private_networks,
        }
    }

    /// Returns true iff no networks are blocked.
    pub fn is_empty(&self) -> bool {
        !self.block_private && self.networks.is_empty()
    }

    /// Returns true iff the given address is blocked.
    pub fn is_blocked(&self, addr: &impl IpAddrLike) -> bool {
        let ip_addr = addr.as_ip_addr();
        if self.block_private && !IpNetwork::from(ip_addr).is_global() {
            return true;
        }
        self.networks.longest_match(ip_addr).is_some()
    }

    /// Removes and returns any addresses with blocked IPs from the given Vec.
    pub fn remove_blocked<T: IpAddrLike>(&self, addrs: &mut Vec<T>) -> Vec<T> {
        if self.is_empty() {
            return vec![];
        }
        let (blocked, allowed) = std::mem::take(addrs)
            .into_iter()
            .partition(|addr| self.is_blocked(addr));
        *addrs = allowed;
        blocked
    }
}

/// AsIpAddr can be implemented to make an "IP-address-like" type compatible
/// with [`BlockedNetworks`].
pub trait IpAddrLike {
    fn as_ip_addr(&self) -> IpAddr;
}

impl IpAddrLike for IpAddr {
    fn as_ip_addr(&self) -> IpAddr {
        *self
    }
}

impl IpAddrLike for SocketAddr {
    fn as_ip_addr(&self) -> IpAddr {
        self.ip()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_is_empty() {
        assert!(BlockedNetworks::default().is_empty());
        assert!(!BlockedNetworks::new([cidr("1.1.1.1/32")], false).is_empty());
        assert!(!BlockedNetworks::new([], true).is_empty());
        assert!(!BlockedNetworks::new([cidr("1.1.1.1/32")], true).is_empty());
    }

    #[test]
    fn test_is_blocked_networks() {
        let blocked = BlockedNetworks::new([cidr("123.123.0.0/16"), cidr("2001::/96")], false);
        assert!(blocked.is_blocked(&ip("123.123.123.123")));
        assert!(blocked.is_blocked(&ip("2001::1000")));
        assert!(!blocked.is_blocked(&ip("123.100.100.100")));
        assert!(!blocked.is_blocked(&ip("2002::1000")));
    }

    #[test]
    fn test_is_blocked_private() {
        let redundant_private_cidr = cidr("10.0.0.0/8");
        let blocked = BlockedNetworks::new([redundant_private_cidr], true);
        assert!(blocked.is_blocked(&ip("127.0.0.1")));
        assert!(blocked.is_blocked(&ip("10.10.10.10")));
        assert!(blocked.is_blocked(&ip("::1")));
        assert!(blocked.is_blocked(&ip("fc00::f00d")));
        assert!(!blocked.is_blocked(&ip("123.123.123.123")));
        assert!(!blocked.is_blocked(&ip("2600::beef")));
    }

    #[test]
    fn test_remove_blocked_socket_addrs() {
        let blocked_networks =
            BlockedNetworks::new([cidr("123.123.0.0/16"), cidr("2600:f00d::/32")], true);

        let allowed: Vec<SocketAddr> = ["123.200.0.1:443", "[2600:beef::1000]:80"]
            .iter()
            .map(|addr| addr.parse().unwrap())
            .collect();
        let blocked: Vec<SocketAddr> = [
            "127.0.0.1:3000",
            "123.123.123.123:443",
            "[::1]:8080",
            "[2600:f00d::4]:80",
        ]
        .iter()
        .map(|addr| addr.parse().unwrap())
        .collect();

        let mut addrs = [allowed.clone(), blocked.clone()].concat();
        let actual_blocked = blocked_networks.remove_blocked(&mut addrs);

        assert_eq!(addrs, allowed);
        assert_eq!(actual_blocked, blocked);
    }

    pub(crate) fn cidr(net: &str) -> IpNetwork {
        IpNetwork::from_str_truncate(net)
            .unwrap_or_else(|err| panic!("invalid cidr {net:?}: {err:?}"))
    }

    pub(crate) fn ip(addr: &str) -> IpAddr {
        addr.parse()
            .unwrap_or_else(|err| panic!("invalid ip addr {addr:?}: {err:?}"))
    }
}
