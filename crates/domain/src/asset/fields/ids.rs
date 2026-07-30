use arrayvec::ArrayString;
use std::fmt;
use std::ops;

const CLIENT_CAP: usize = 32;

// ── Client Asset ID ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClientAssetId(ArrayString<CLIENT_CAP>);

impl ClientAssetId {
    pub fn new(id: &str) -> Option<Self> {
        ArrayString::try_from(id).ok().map(Self)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ops::Deref for ClientAssetId {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ClientAssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Asset ID ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssetId(u64);

impl AssetId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId ={}", self.0)
    }
}
