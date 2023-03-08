use std::sync::Arc;

use anyhow::Result;
use headless_chrome::{protocol::cdp, Tab};

pub trait TabEvasions {
    fn bypass_webgl_vendor(&self) -> Result<()>; // fn replace_all_cow<'t, R: Replacer>(&self, bytes: Cow<'t, [u8]>, rep: R) -> Cow<'t, [u8]>;
}

impl TabEvasions for Arc<Tab> {
    fn bypass_webgl_vendor(&self) -> Result<()> {
        // let r = "const getParameter = WebGLRenderingContext.getParameter;
        // WebGLRenderingContext.prototype.getParameter = function(parameter) {
        //     // UNMASKED_VENDOR_WEBGL
        //     if (parameter === 37445) {
        //         return 'Google Inc. (NVIDIA)';
        //     }
        //     // UNMASKED_RENDERER_WEBGL
        //     if (parameter === 37446) {
        //         return 'ANGLE (NVIDIA, NVIDIA GeForce GTX 1050 Direct3D11 vs_5_0 ps_5_0, D3D11-27.21.14.5671)';
        //     }
        //     return getParameter(parameter);
        // };";

        // self.call_method(cdp::Page::AddScriptToEvaluateOnNewDocument {
        //     source: r.to_string(),
        //     world_name: None,
        //     include_command_line_api: None,
        // })?;
        Ok(())
    }
}
