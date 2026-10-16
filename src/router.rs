use crate::types::{
    BoxFuture, Handler, HttpMethod, HttpRequest, HttpResponse, Route, Router, StatusCode,
};
use std::collections::HashMap;

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    // Create a new route
    fn route(path: &str, handler: Handler, method: HttpMethod) -> Route {
        Route {
            path: path.to_string(),
            method: method,
            handler,
        }
    }

    // Handles  incoming HTTP requests and return the appropriate response
    pub async fn handle_request(&self, request: HttpRequest) -> HttpResponse {
        let mut path_found = false; // Flag to check if the path was found but the method was not allowed

        // let mut method_found = false;
        let mut allowed_methods: Vec<HttpMethod> = Vec::new(); // Vector to store allowed methods for the found path

        for route in &self.routes {
            if route.path_matches(&request) {
                path_found = true;
                allowed_methods.push(route.method.clone()); // Store the allowed method for the found path

                if route.method == request.request_lines.method {
                    return (route.handler)(request).await;
                }
            }
        }

        if path_found {
            return HttpResponse {
                status_code: StatusCode::METHOD_NOT_ALLOWED,
                headers: HashMap::from([(
                    "Allow".to_string(),
                    allowed_methods
                        .iter()
                        .map(|m| format!("{:?}", m))
                        .collect::<Vec<String>>()
                        .join(", "),
                )]),
                body: "Method Not Allowed".as_bytes().to_vec(),
            };
        }
        HttpResponse {
            status_code: StatusCode::NOT_FOUND,
            headers: HashMap::new(),
            body: "Not Found".as_bytes().to_vec(),
        }
    }

    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::GET);
    }

    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::POST);
    }

    pub fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::PUT);
    }

    pub fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::DELETE);
    }

    pub fn patch<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::PATCH);
    }

    pub fn options<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::OPTIONS);
    }

    pub fn head<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        self.register_route(path, handler, HttpMethod::HEAD);
    }

    // Register a route with the specified path, handler, and HTTP method
    pub fn register_route<F, Fut>(&mut self, path: &str, handler: F, method: HttpMethod)
    where
        F: Fn(HttpRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HttpResponse> + Send + 'static,
    {
        let handler = Box::new(move |request: HttpRequest| Box::pin(handler(request)) as BoxFuture);

        self.add_route(Self::route(path, handler, method));
    }
}
