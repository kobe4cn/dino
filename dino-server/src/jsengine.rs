use std::collections::HashMap;

use anyhow::Result;

use axum::{body::Body, response::Response};
use dino_macros::{FromJs, IntoJs};
use rquickjs::{Context, Function, Object, Promise, Runtime};
use typed_builder::TypedBuilder;

#[allow(unused)]
#[derive(Clone)]
pub struct JsEngine {
    pub rt: Runtime,
    pub ctx: Context,
}
fn print(msg: String) {
    println!("{msg}");
}

#[derive(Debug, TypedBuilder, IntoJs)]
pub struct Req {
    #[builder(default)]
    pub headers: HashMap<String, String>,
    #[builder(default)]
    pub body: Option<String>,
    #[builder(setter(into))]
    pub method: String,
    #[builder(setter(into))]
    pub url: String,
    #[builder(default)]
    pub query: HashMap<String, String>,
    #[builder(default)]
    pub params: HashMap<String, String>,
}

#[derive(Debug, FromJs, serde::Serialize)]
pub struct Res {
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub status: u16,
}

impl From<Res> for Response {
    fn from(res: Res) -> Self {
        let mut builder = Response::builder().status(res.status);
        for (k, v) in res.headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = res.body {
            builder.body(body.into()).unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        }
    }
}

impl JsEngine {
    pub async fn init() -> Result<Self> {
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;
        Ok(Self { rt, ctx })
    }
    pub fn new(module: &str) -> Result<Self> {
        //using rquickjs for set js global object and run js code
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
    pub fn run(&self, name: &str, req: Req) -> Result<Res> {
        self.ctx.with(|ctx| {
            let global = ctx.globals();
            let handlers = global.get::<_, Object>("handlers")?;
            let function = handlers.get::<_, Function>(name)?;
            let result: Promise = function.call((req,))?;

            Ok::<_, anyhow::Error>(result.finish()?)
        })
    }
}
