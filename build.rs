// Build script for integrating SvelteKit frontend build
use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/svelte.config.js");
    println!("cargo:rerun-if-changed=frontend/vite.config.ts");
    
    // Only build frontend in release mode or if explicitly requested
    let should_build_frontend = env::var("CARGO_CFG_RELEASE").is_ok() 
        || env::var("BUILD_FRONTEND").is_ok();
    
    if should_build_frontend {
        build_frontend();
    } else {
        println!("cargo:warning=Skipping frontend build in debug mode. Set BUILD_FRONTEND=1 to force build.");
        ensure_build_directory();
    }
}

fn build_frontend() {
    let frontend_dir = Path::new("frontend");
    
    if !frontend_dir.exists() {
        println!("cargo:warning=Frontend directory not found, skipping build");
        return;
    }
    
    println!("cargo:warning=Building SvelteKit frontend...");
    
    // Check if node_modules exists, if not run npm install
    let node_modules_dir = frontend_dir.join("node_modules");
    if !node_modules_dir.exists() {
        println!("cargo:warning=Installing frontend dependencies...");
        let npm_install = Command::new("npm")
            .args(&["install"])
            .current_dir(frontend_dir)
            .status();
        
        match npm_install {
            Ok(status) if status.success() => {
                println!("cargo:warning=Frontend dependencies installed successfully");
            }
            Ok(status) => {
                println!("cargo:warning=npm install failed with status: {}", status);
                return;
            }
            Err(e) => {
                println!("cargo:warning=Failed to run npm install: {}", e);
                return;
            }
        }
    }
    
    // Run the frontend build
    let build_result = Command::new("npm")
        .args(&["run", "build"])
        .current_dir(frontend_dir)
        .status();
    
    match build_result {
        Ok(status) if status.success() => {
            println!("cargo:warning=Frontend build completed successfully");
        }
        Ok(status) => {
            println!("cargo:warning=Frontend build failed with status: {}", status);
            ensure_build_directory();
        }
        Err(e) => {
            println!("cargo:warning=Failed to run frontend build: {}", e);
            ensure_build_directory();
        }
    }
}

fn ensure_build_directory() {
    let build_dir = Path::new("frontend/build");
    
    if !build_dir.exists() {
        println!("cargo:warning=Creating placeholder build directory");
        std::fs::create_dir_all(build_dir).expect("Failed to create build directory");
        
        // Create a basic index.html if it doesn't exist
        let index_path = build_dir.join("index.html");
        if !index_path.exists() {
            let placeholder_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Axum SSE Demo - Development</title>
    <style>
        body { 
            font-family: Arial, sans-serif; 
            max-width: 800px; 
            margin: 0 auto; 
            padding: 20px;
            background-color: #1a1a1a;
            color: #ffffff;
        }
        .time-display { 
            font-size: 2em; 
            text-align: center; 
            margin: 20px 0;
            padding: 20px;
            border: 1px solid #333;
            background-color: #2a2a2a;
        }
        .banner {
            background-color: #dc3545;
            color: white;
            padding: 10px;
            text-align: center;
            display: none;
        }
        .nav { margin: 20px 0; }
        .nav a { color: #6c757d; margin-right: 20px; text-decoration: none; }
        .nav a:hover { color: #ffffff; }
    </style>
</head>
<body>
    <div class="banner" id="connectionBanner">Connection lost - attempting to reconnect...</div>
    <nav class="nav">
        <a href="/">Main</a>
        <a href="/about">About</a>
    </nav>
    <h1>Axum SSE Time Broadcasting</h1>
    <div class="time-display" id="timeDisplay">Connecting...</div>
    <p><strong>Development Mode:</strong> This is a placeholder page. The full SvelteKit frontend will be built in release mode.</p>
    <p>To build the frontend manually: <code>cd frontend && npm run build</code></p>
    <p>To force frontend build during development: <code>BUILD_FRONTEND=1 cargo build</code></p>
    
    <script>
        const timeDisplay = document.getElementById('timeDisplay');
        const banner = document.getElementById('connectionBanner');
        
        function connectSSE() {
            const eventSource = new EventSource('/api/time/stream');
            
            eventSource.addEventListener('time-update', function(event) {
                const timeData = JSON.parse(event.data);
                timeDisplay.textContent = timeData.formatted_time;
                banner.style.display = 'none';
            });
            
            eventSource.onerror = function(event) {
                banner.style.display = 'block';
                setTimeout(connectSSE, 5000);
            };
        }
        
        connectSSE();
    </script>
</body>
</html>"#;
            
            std::fs::write(&index_path, placeholder_html)
                .expect("Failed to create placeholder index.html");
        }
    }
}