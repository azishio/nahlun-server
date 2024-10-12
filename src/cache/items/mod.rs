use strum::EnumDiscriminants;

// キャッシュデータの定義
#[derive(Clone, EnumDiscriminants)]
pub enum CachedData {
    // (仮)
    StringData(String),
}

impl CachedData {
    // キャッシュデータをバイト列に変換
    pub(crate) fn as_bytes(&self) -> (CachedDataDiscriminants, &[u8]) {
        match self {
            CachedData::StringData(s) => (CachedDataDiscriminants::StringData, s.as_bytes()),
        }
    }

    pub(crate) fn from_bytes(dtype: CachedDataDiscriminants, bytes: &[u8]) -> Self {
        match dtype {
            CachedDataDiscriminants::StringData => {
                CachedData::StringData(String::from_utf8(bytes.to_vec()).unwrap())
            }
        }
    }
}