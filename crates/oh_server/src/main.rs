#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] {
        println!("{}", oh_server::USAGE);
        return;
    }
    if let Err(error) = run(args).await {
        eprintln!("startup error: {error}");
        std::process::exit(1);
    }
}
async fn run(args: Vec<String>) -> Result<(), String> {
    let options = oh_server::Options::parse(&args)?;
    let (shutdown, stopped) = tokio::sync::watch::channel(false);
    let prepared = if options.load_save.is_some() {
        Some(oh_server::Host::load_with_save(
            &options.pack_root,
            stopped.clone(),
            options.load_save.as_deref(),
            options.force,
        )?)
    } else {
        None
    };
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, options.port))
        .await
        .map_err(|e| {
            format!(
                "cannot bind 127.0.0.1:{}: {e}; choose another --port",
                options.port
            )
        })?;
    let host = match prepared {
        Some(host) => host,
        None => oh_server::Host::load(&options.pack_root, stopped)?,
    };
    let url = format!("http://127.0.0.1:{}/", options.port);
    println!(
        "OpenHOI4: {url}\nPack: {} ({}) from {}\nOpen this address in a browser. Ctrl+C to shut down. --help for usage.",
        host.pack.id,
        host.pack.hash,
        options.pack_root.display()
    );
    if options.open {
        let address = url.clone();
        // Some OS browser launchers wait for the browser to exit. Keep serving
        // HTTP and handling Ctrl+C while the OS integration runs separately.
        std::thread::Builder::new()
            .name("openhoi-browser".into())
            .spawn(move || match oh_server::open_browser(&address) {
                Ok(()) => println!("Default browser launcher exited successfully."),
                Err(error) => eprintln!("{error}"),
            })
            .map_err(|e| format!("cannot start browser launcher: {e}; open {url} manually"))?;
    }
    axum::serve(listener, oh_server::router(host))
        .with_graceful_shutdown(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    shutdown.send_replace(true);
                    println!("Shutting down OpenHOI.");
                }
                Err(e) => eprintln!("shutdown signal error: {e}"),
            }
        })
        .await
        .map_err(|e| e.to_string())?;
    println!("OpenHOI stopped normally.");
    Ok(())
}
