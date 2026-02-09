use crate::server::websocket::{handle_websocket, ConnectionManager};
use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::net::TcpListener;
use tracing::{debug, error};

#[derive(Clone)]
pub struct ResourcePaths {
    pub base_path: PathBuf,
}

impl ResourcePaths {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn mobile_index_html(&self) -> PathBuf {
        self.base_path.join("src").join("mobile").join("index.html")
    }

    pub fn dist_file(&self, relative_path: &str) -> PathBuf {
        self.base_path.join(relative_path)
    }
}

fn content_type_for_path(path: &str) -> &'static str {
    if path == "manifest.json" || path.ends_with(".webmanifest") {
        "application/manifest+json; charset=utf-8"
    } else if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}

async fn serve_dist_file(relative_path: &str, paths: Arc<ResourcePaths>) -> Response<Full<Bytes>> {
    if relative_path.contains("..") || relative_path.contains('\\') {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from_static(b"Bad Request")))
            .unwrap();
    }

    let full_path = {
        let direct = paths.dist_file(relative_path);
        if direct.exists() {
            direct
        } else {
            paths.base_path.join("public").join(relative_path)
        }
    };
    let content_type = content_type_for_path(relative_path);
    let cache_control = if relative_path == "sw.js" || relative_path == "manifest.json" {
        "no-store, no-cache, must-revalidate, proxy-revalidate"
    } else if relative_path.ends_with(".html") {
        "no-store, no-cache, must-revalidate, proxy-revalidate"
    } else {
        "no-cache"
    };

    match fs::read(&full_path).await {
        Ok(bytes) => Response::builder()
            .header("Content-Type", content_type)
            .header("Cache-Control", cache_control)
            .body(Full::new(Bytes::from(bytes)))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from_static(b"File not found")))
            .unwrap(),
    }
}

fn extract_hostname(host_header: &str) -> &str {
    if let Some(stripped) = host_header.strip_prefix('[') {
        if let Some(end) = stripped.find(']') {
            return &stripped[..end];
        }
    }
    host_header.split(':').next().unwrap_or(host_header)
}

fn dev_mobile_html(hostname: &str) -> String {
    format!(
        r##"<!doctype html>
<html lang="en" translate="no">
  <head>
    <meta charset="UTF-8" />
    <meta name="google" content="notranslate" />
    <meta
      name="viewport"
      content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
    />
    <title>DropVoice</title>
    <link rel="manifest" href="/manifest.json" />
    <meta name="theme-color" content="#1a1a2e" />
    <meta name="apple-mobile-web-app-capable" content="yes" />
    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent" />
    <meta name="apple-mobile-web-app-title" content="DropVoice" />
    <link rel="apple-touch-icon" href="/icons/icon-192.png" />
    <script type="module">
      import RefreshRuntime from "http://{hostname}:5173/@react-refresh";
      RefreshRuntime.injectIntoGlobalHook(window);
      window.$RefreshReg$ = () => {{}};
      window.$RefreshSig$ = () => (type) => type;
      window.__vite_plugin_react_preamble_installed__ = true;
    </script>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="http://{hostname}:5173/@vite/client"></script>
    <script type="module" src="http://{hostname}:5173/src/mobile/index.tsx"></script>
    <script>
      if ("serviceWorker" in navigator) {{
        navigator.serviceWorker.register("/sw.js");
      }}
    </script>
  </body>
</html>
"##
    )
}

/// Handle HTTP requests
async fn handle_request(
    req: Request<hyper::body::Incoming>,
    connection_manager: ConnectionManager,
    paths: Arc<ResourcePaths>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path();
    let method = req.method();

    debug!("Request: {} {}", method, path);

    if method == Method::GET && path == "/ws" {
        return handle_websocket(req, connection_manager).await;
    }

    if method == Method::GET && path == "/" {
        // Development mode: always use Vite dev server for hot reload
        if cfg!(debug_assertions) {
            let host_header = req
                .headers()
                .get(hyper::header::HOST)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("localhost");
            let hostname = extract_hostname(host_header);
            return Ok(Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "no-store, no-cache, must-revalidate, proxy-revalidate")
                .header("Pragma", "no-cache")
                .header("Expires", "0")
                .body(Full::new(Bytes::from(dev_mobile_html(hostname))))
                .unwrap());
        }

        // Production mode: serve from dist
        let index = paths.mobile_index_html();
        if index.exists() {
            return Ok(Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "no-store, no-cache, must-revalidate, proxy-revalidate")
                .header("Pragma", "no-cache")
                .header("Expires", "0")
                .body(Full::new(Bytes::from(fs::read(&index).await.unwrap_or_default())))
                .unwrap());
        }

        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from_static(b"mobile/index.html not found")))
            .unwrap());
    }

    if method == Method::GET
        && (path == "/manifest.json"
            || path == "/sw.js"
            || path.starts_with("/icons/")
            || path.starts_with("/assets/")
            || path.starts_with("/mobile/")
            || path.starts_with("/src/"))
    {
        return Ok(serve_dist_file(&path[1..], paths).await);
    }

    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from_static(b"Not Found")))
        .unwrap())
}

/// Start the HTTP server with a pre-bound listener
pub async fn serve_with_listener(
    listener: TcpListener,
    connection_manager: ConnectionManager,
    paths: Arc<ResourcePaths>,
) -> anyhow::Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let conn_manager = connection_manager.clone();
        let paths = paths.clone();

        tokio::task::spawn(async move {
            let service =
                service_fn(|req| handle_request(req, conn_manager.clone(), paths.clone()));

            if let Err(err) = http1::Builder::new()
                .preserve_header_case(true)
                .title_case_headers(true)
                .serve_connection(io, service)
                .with_upgrades()
                .await
            {
                error!("Error serving connection: {}", err);
            }
        });
    }
}
