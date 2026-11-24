use serde::Serialize;
use tic_tac_twist_engine::{Game, GameConfig, GameSnapshot, SearchResult};
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AiTurnResponse {
    result: SearchResult,
    state: GameSnapshot,
}

fn to_js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[wasm_bindgen]
pub fn default_config(board_size: u8) -> Result<JsValue, JsValue> {
    let cfg = GameConfig::default_classic(board_size);
    serde_wasm_bindgen::to_value(&cfg).map_err(to_js_err)
}

#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new(config_js: JsValue) -> Result<WasmGame, JsValue> {
        let config: GameConfig = serde_wasm_bindgen::from_value(config_js).map_err(to_js_err)?;
        let inner = Game::new(config).map_err(to_js_err)?;
        Ok(Self { inner })
    }

    pub fn state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.snapshot()).map_err(to_js_err)
    }

    pub fn play_move(&mut self, cell: u8) -> Result<JsValue, JsValue> {
        self.inner.play_move(cell).map_err(to_js_err)?;
        self.state()
    }

    pub fn play_ai_turn(&mut self) -> Result<JsValue, JsValue> {
        let result = self.inner.play_ai_turn().map_err(to_js_err)?;
        let resp = AiTurnResponse {
            result,
            state: self.inner.snapshot(),
        };
        serde_wasm_bindgen::to_value(&resp).map_err(to_js_err)
    }

    pub fn undo(&mut self) -> Result<JsValue, JsValue> {
        self.inner.undo().map_err(to_js_err)?;
        self.state()
    }

    pub fn reset(&mut self) -> Result<JsValue, JsValue> {
        self.inner.reset();
        self.state()
    }
}
