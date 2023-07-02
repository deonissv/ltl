use gloo_utils::format::JsValueSerdeExt;
use js_sys::{Int32Array, Uint16Array};
use wasm_bindgen::prelude::*;

use crate::ltl_engine::config;
use crate::neighbourhood::Neighbourhood;
use crate::rnd::Rng;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    config: config::Config,
}

impl Config {
    pub fn config(&self) -> config::Config {
        self.config.clone()
    }
}

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(
        rr: u8,
        cc: u8,
        mm: u8,
        ss: Uint16Array,
        bb: Uint16Array,
        nn: Neighbourhood,
    ) -> Self {
        Config {
            config: config::Config {
                rr,
                cc,
                mm,
                ss: (ss.at(0).unwrap(), ss.at(1).unwrap()),
                bb: (bb.at(0).unwrap(), bb.at(1).unwrap()),
                nn: nn.to_neighbourhood(),
            },
        }
    }

    #[wasm_bindgen(getter)]
    pub fn rr(&self) -> u8 {
        self.config.rr
    }

    #[wasm_bindgen(getter)]
    pub fn cc(&self) -> u8 {
        self.config.cc
    }

    #[wasm_bindgen(getter)]
    pub fn mm(&self) -> u8 {
        self.config.mm
    }

    #[wasm_bindgen(getter)]
    pub fn ss(&self) -> Int32Array {
        let arr = JsValue::from_serde(&[self.config.ss.0, self.config.ss.1]).unwrap();
        Int32Array::new(&arr)
    }

    #[wasm_bindgen(getter)]
    pub fn bb(&self) -> Int32Array {
        let arr = JsValue::from_serde(&[self.config.bb.0, self.config.bb.1]).unwrap();
        Int32Array::new(&arr)
    }

    #[wasm_bindgen(getter)]
    pub fn nn(&self) -> Neighbourhood {
        Neighbourhood::from_neighbourhood(self.config.nn.clone())
    }

    #[wasm_bindgen(static_method_of=Config)]
    pub fn randomize() -> Self {
        let mut r = Rng {};
        Config {
            config: config::Config::randomize(&mut r),
        }
    }
}
