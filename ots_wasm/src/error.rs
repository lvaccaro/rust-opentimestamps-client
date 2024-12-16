use wasm_bindgen::prelude::*;
//use ots_core::error::Error as OtsError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Generic(String),

    #[error("{0:?}")]
    JsVal(JsValue),
}

impl From<Error> for JsValue {
    fn from(val: Error) -> JsValue {
        if let Error::JsVal(e) = val {
            e
        } else {
            format!("{}", val).into()
        }
    }
}
