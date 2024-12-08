use anyhow::Result;

use rquickjs::{Context, Function, Object, Runtime};

#[allow(unused)]
pub struct JsEngine {
    rt: Runtime,
    ctx: Context,
}
fn print(msg: String) {
    println!("{msg}");
}
impl JsEngine {
    pub fn new(module: &str) -> Result<Self> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        ctx.with(|ctx| {
            let global = ctx.globals();
            let module: Object = ctx.eval(module)?;
            global.set("handlers", module)?;
            global.set(
                "print",
                Function::new(ctx.clone(), print)?.with_name("print")?,
            )?;
            Ok::<_, anyhow::Error>(())
        })?;

        Ok(Self { rt, ctx })
    }
    pub fn run(&self, code: &str) -> Result<()> {
        self.ctx.with(|ctx| {
            let _: () = ctx.eval_promise(code)?.finish()?;
            // let print = ctx.globals().get::<_, Function>("print")?;
            // print.call::<_, ()>(result)?;
            // println!("==================={:?}", result);
            Ok::<_, anyhow::Error>(())
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_js_engine() -> Result<()> {
        let engine = JsEngine::new(
            r#"
            (function(){async function hello(){print("hello world");return"hello";}return{hello:hello};})();
        "#,
        )?;
        engine.run("await handlers.hello()")?;
        Ok(())
    }
}
