use wasm_bindgen::prelude::*;

use crate::ltl_engine::neighbourhood;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum Neighbourhood {
    Neumann,
    Moore,
}

impl Neighbourhood {
    pub fn to_neighbourhood(&self) -> neighbourhood::Neighbourhood {
        match self {
            Neighbourhood::Neumann => neighbourhood::Neighbourhood::Neumann,
            Neighbourhood::Moore => neighbourhood::Neighbourhood::Moore,
        }
    }

    pub fn from_neighbourhood(neighbourhood: neighbourhood::Neighbourhood) -> Self {
        match neighbourhood {
            neighbourhood::Neighbourhood::Neumann => Neighbourhood::Neumann,
            neighbourhood::Neighbourhood::Moore => Neighbourhood::Moore,
        }
    }
}
