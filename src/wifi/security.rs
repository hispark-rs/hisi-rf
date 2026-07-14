//! Typed station security profiles.

/// IEEE 802.11 management-frame protection policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagementFrameProtection {
    /// PMF is negotiated when the access point supports it.
    Optional,
    /// PMF is mandatory. WPA3-Personal always uses this policy.
    Required,
}

/// SAE password-element derivation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaePwe {
    /// Legacy hunting-and-pecking derivation.
    HuntAndPeck,
    /// Hash-to-element derivation.
    HashToElement,
    /// Accept either derivation advertised by the access point.
    Both,
}

/// Validated Personal-mode security selected for one station connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonalSecurity {
    /// WPA2-Personal with CCMP and optional PMF.
    Wpa2,
    /// WPA3-Personal/SAE with mandatory PMF.
    Wpa3 { sae_pwe: SaePwe },
}

impl PersonalSecurity {
    /// Management-frame protection implied by this security profile.
    pub const fn management_frame_protection(self) -> ManagementFrameProtection {
        match self {
            Self::Wpa2 => ManagementFrameProtection::Optional,
            Self::Wpa3 { .. } => ManagementFrameProtection::Required,
        }
    }
}
