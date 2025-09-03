#[derive(Clone, Debug)]
/// Auth struct, used to authenticate with Azure Speech Services.
pub enum Auth {
    /// Authenticate against Azure cloud using `region` and `subscription` key.
    Subscription {
        region: String,
        subscription: String,
    },
    /// Authenticate against a self-hosted/container endpoint (host auth).
    /// Example hosts:
    /// - ws://localhost:5000
    /// - wss://my-gateway.example.com:5000
    Host { host: String },
}

impl Auth {
    /// Create a new Auth instance from a subscription key and a region.
    pub fn from_subscription(region: impl Into<String>, subscription: impl Into<String>) -> Self {
        Auth::Subscription {
            region: region.into(),
            subscription: subscription.into(),
        }
    }

    /// Create a new Auth instance using host authentication
    pub fn from_host(host: impl Into<String>) -> Self {
        Auth::Host { host: host.into() }
    }

    pub(crate) fn subscription_region(&self) -> Option<(&str, &str)> {
        match self {
            Auth::Subscription {
                region,
                subscription,
            } => Some((region.as_str(), subscription.as_str())),
            _ => None,
        }
    }
}
