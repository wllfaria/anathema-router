// This is an attempt to implement a router into anathema, abstracting away navigation
// between different pages to a dedicated component
//
// There is a limitation in which when you navigate from one component to other, you need
// to press `Tab` once to focus it, which makes this slightly annoying to work with for now
//
// Ideally, if we had a way to request focus or force focus onto a component this would be
// fine. We would also like to figure a way to send in custom data based on each route.

use anathema::backend::Backend;
use anathema::component::Component;
use anathema::runtime::{Error, RuntimeBuilder};
use anathema::state::{List, State, Value};
use anathema::templates::ToSourceKind;

static RECEIVE_IDENT: &str = "navigate";

pub struct RouterBuilder {
    routes: Vec<String>,
    // path exists strictly for debugging purposes. we don't need to save
    // the router as a file as we can provide the string as a template to
    // anathema
    path: Option<String>,
}

impl RouterBuilder {
    pub fn add_route(mut self, route: &str) -> RouterBuilder {
        self.routes.push(route.to_string());
        self
    }

    pub fn generate_template(&self) -> String {
        let mut template = String::new();
        for route in &self.routes {
            let component = format!(
                r#"
if active_route == "{route}"
    @{route} ({RECEIVE_IDENT}->{RECEIVE_IDENT})
            "#
            );
            template.push_str(&component);
        }

        template
    }

    pub fn finish<T>(self, entrypoint: &str, runtime: &mut RuntimeBuilder<T>) -> Result<(), Error>
    where
        T: Backend,
    {
        let template = self.generate_template();

        let router = Router {};
        let router_state = RouterState {
            routes: List::from_iter(self.routes),
            active_route: entrypoint.to_string().into(),
        };

        if self.path.is_some() {
            std::fs::write(self.path.as_ref().unwrap(), &template).unwrap();
            runtime.register_component("router", self.path.unwrap(), router, router_state)?;
        } else {
            runtime.register_component("router", template.to_template(), router, router_state)?;
        }
        Ok(())
    }
}

pub struct Router;

impl Router {
    pub fn builder() -> RouterBuilder {
        RouterBuilder {
            routes: vec![],
            path: None,
        }
    }
}

#[derive(State)]
pub struct RouterState {
    active_route: Value<String>,
    routes: Value<List<String>>,
}

impl Component for Router {
    // TODO: figure a way to also navigate through messages, but this should
    // enable users to also pass data to routes. This can't be achieved through
    // receive as receive only accepts a CommonVal, which doesn't allow us to
    // have custom types
    //
    // Ideally, message would be something like an enum from Route -> T
    type Message = ();
    type State = RouterState;

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        if ident != RECEIVE_IDENT {
            return;
        }

        let route = value.to_string();
        state.active_route.set(route);
    }
}
