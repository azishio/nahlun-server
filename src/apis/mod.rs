//! パスごとに呼び出される処理を定義

mod private;
mod public;
mod tile;

/// パスごとの処理内容をimplするための構造体
#[derive(Clone)]
pub struct ServerImpl {}

impl AsRef<ServerImpl> for ServerImpl {
    fn as_ref(&self) -> &ServerImpl {
        self
    }
}
