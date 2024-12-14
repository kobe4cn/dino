use anyhow::Result;
use arc_swap::ArcSwap;
use axum::http::Method;
use matchit::{Match, Router};
use std::{ops::Deref, sync::Arc};

use crate::{AppError, ProjectRouters};

#[derive(Clone)]
pub struct SwappableAppRouter {
    pub inners: Arc<ArcSwap<AppRouterInner>>,
}
pub struct AppRouterInner {
    pub code: String,
    pub router: Router<MethodRoute>,
}

#[derive(Clone)]
pub struct AppRouter(Arc<AppRouterInner>);

#[derive(Debug, Default, Clone)]
pub struct MethodRoute {
    get: Option<String>,
    head: Option<String>,
    delete: Option<String>,
    options: Option<String>,
    patch: Option<String>,
    post: Option<String>,
    put: Option<String>,
    trace: Option<String>,
    connect: Option<String>,
}
impl AppRouterInner {
    pub fn new(code: String, router: Router<MethodRoute>) -> Self {
        Self { code, router }
    }
}
impl Deref for AppRouter {
    type Target = AppRouterInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl SwappableAppRouter {
    pub fn new(code: String, routes: ProjectRouters) -> Result<Self> {
        let router = Self::get_router(routes)?;
        let inner = AppRouterInner::new(code, router);
        Ok(Self {
            inners: Arc::new(ArcSwap::from_pointee(inner)),
        })
    }
    pub fn swap(&self, code: String, routes: ProjectRouters) -> Result<()> {
        let router = Self::get_router(routes)?;
        let inner = AppRouterInner::new(code, router);
        self.inners.store(Arc::new(inner));
        Ok(())
    }

    pub fn load(&self) -> AppRouter {
        AppRouter(self.inners.load_full())
    }

    fn get_router(routes: ProjectRouters) -> Result<Router<MethodRoute>> {
        let mut router = Router::new();
        for (path, methods) in routes {
            let mut method_route = MethodRoute::default();
            for method in methods {
                match method.method {
                    Method::GET => method_route.get = Some(method.handler),
                    Method::HEAD => method_route.head = Some(method.handler),
                    Method::DELETE => method_route.delete = Some(method.handler),
                    Method::OPTIONS => method_route.options = Some(method.handler),
                    Method::PATCH => method_route.patch = Some(method.handler),
                    Method::POST => method_route.post = Some(method.handler),
                    Method::PUT => method_route.put = Some(method.handler),
                    Method::TRACE => method_route.trace = Some(method.handler),
                    Method::CONNECT => method_route.connect = Some(method.handler),
                    _ => {
                        panic!("Unsupported method");
                    }
                }
            }
            // println!("path: {path}, method_route: {method_route:?}");
            // let str=method_route.
            router.insert(path, method_route)?;
        }
        Ok(router)
    }
}
#[allow(elided_named_lifetimes)]
impl AppRouter {
    pub fn match_it<'m, 'p>(
        &'m self,
        method: Method,
        path: &'p str,
    ) -> Result<Match<&'m str>, AppError>
    where
        'p: 'm,
    {
        // println!("============={:?}", &path);
        let Ok(ret) = self.router.at(path) else {
            return Err(AppError::RouterPathNotFound(path.to_string()));
        };

        // println!("============={:?}", ret);

        let s = match method {
            Method::GET => ret.value.get.as_deref(),
            Method::HEAD => ret.value.get.as_deref(),
            Method::DELETE => ret.value.get.as_deref(),
            Method::OPTIONS => ret.value.get.as_deref(),
            Method::PATCH => ret.value.get.as_deref(),
            Method::POST => ret.value.get.as_deref(),
            Method::PUT => ret.value.get.as_deref(),
            Method::TRACE => ret.value.get.as_deref(),
            Method::CONNECT => ret.value.get.as_deref(),
            _ => None,
        }
        .ok_or_else(|| AppError::RouterMethodNotAllow(method))?;

        Ok(Match {
            value: s,
            params: ret.params,
        })
    }
}

// impl Deref for AppRouter {
//     type Target = Router<MethodRoute>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

#[cfg(test)]
mod tests {
    use crate::ProjectConfig;

    use super::*;
    use axum::http::Method;

    #[test]
    fn app_router_match_should_work() {
        let config = include_str!("../fixtures/config.yml");
        let config: ProjectConfig = serde_yaml::from_str(config).unwrap();
        let code = "".to_string();
        let router = SwappableAppRouter::new(code, config.routes).unwrap();
        let app_router = router.load();
        let matched = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(matched.value, "hello");
        assert_eq!(matched.params.get("id"), Some("1"));
    }
    #[test]
    fn app_router_swap_should_work() {
        let config = include_str!("../fixtures/config.yml");
        let config: ProjectConfig = serde_yaml::from_str(config).unwrap();
        let code = "".to_string();
        let router = SwappableAppRouter::new(code, config.routes).unwrap();
        let app_router = router.load();
        let matched = app_router.match_it(Method::GET, "/api/hello/1").unwrap();
        assert_eq!(matched.value, "hello");
        assert_eq!(matched.params.get("id"), Some("1"));

        let new_config = include_str!("../fixtures/config1.yml");
        let new_config: ProjectConfig = serde_yaml::from_str(new_config).unwrap();
        let code = "".to_string();
        router.swap(code, new_config.routes).unwrap();
        let app_router = router.load();
        let matched = app_router.match_it(Method::GET, "/api/goodbye/2").unwrap();
        assert_eq!(matched.value, "handler1");
    }
}
