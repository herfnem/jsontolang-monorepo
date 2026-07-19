use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_assembly(code: &str) -> Result<JsValue, JsValue> {
    let result = parser::parse_program(code);
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub struct WasmVm {
    inner: core::Vm,
}

#[wasm_bindgen]
impl WasmVm {
    #[wasm_bindgen(constructor)]
    pub fn new(code: &str) -> Result<WasmVm, JsValue> {
        let parsed = parser::parse_program(code);
        if !parsed.errors.is_empty() {
            let err_msg = parsed
                .errors
                .iter()
                .map(|e| format!("Line {}: {}", e.line, e.message))
                .collect::<Vec<String>>()
                .join("\n");
            return Err(JsValue::from_str(&err_msg));
        }

        let inner = core::Vm::new(parsed.instructions, parsed.labels);
        Ok(WasmVm { inner })
    }

    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        match self.inner.step() {
            Ok(state) => {
                serde_wasm_bindgen::to_value(&state).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    pub fn run(&mut self, max_steps: usize) -> Result<JsValue, JsValue> {
        let state = self.inner.run(max_steps);
        serde_wasm_bindgen::to_value(&state).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.state)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn get_instructions(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.instructions)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
