use std::fmt;

/// Event resulting from a transition to a new tunnel state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "state", content = "details")]
pub enum TunnelStateTransition {
    /// No connection is established and network is unsecured.
    Disconnected,
    /// Network is secured but tunnel is still connecting.
    Connecting,
    /// Tunnel is connected.
    Connected,
    /// Disconnecting tunnel.
    Disconnecting(ActionAfterDisconnect),
    /// Tunnel is disconnected but secured by blocking all connections.
    Blocked(BlockReason),
}

/// Action that will be taken after disconnection is complete.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAfterDisconnect {
    Nothing,
    Block,
    Reconnect,
}

impl TunnelStateTransition {
    pub fn is_blocked(&self) -> bool {
        match self {
            TunnelStateTransition::Blocked(_) => true,
            _ => false,
        }
    }
}

/// Reason for entering the blocked state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "reason", content = "details")]
pub enum BlockReason {
    /// Authentication with remote server failed.
    AuthFailed(Option<String>),
    /// Failed to configure IPv6 because it's disabled in the platform.
    Ipv6Unavailable,
    /// Failed to set security policy.
    SetSecurityPolicyError,
    /// Failed to start connection to remote server.
    StartTunnelError,
    /// No relay server matching the current filter parameters.
    NoMatchingRelay,
}

impl fmt::Display for BlockReason {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            BlockReason::AuthFailed(ref reason) => {
                return write!(
                    formatter,
                    "Authentication with remote server failed: {}",
                    match reason {
                        Some(ref reason) => reason.as_str(),
                        None => "No reason provided",
                    }
                );
            }
            BlockReason::Ipv6Unavailable => {
                "Failed to configure IPv6 because it's disabled in the platform"
            }
            BlockReason::SetSecurityPolicyError => "Failed to set security policy",
            BlockReason::StartTunnelError => "Failed to start connection to remote server",
            BlockReason::NoMatchingRelay => "No relay server matches the current settings",
        };

        write!(formatter, "{}", description)
    }
}
