use mdns_sd::{HostnameResolutionEvent, ServiceDaemon, ServiceInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "resolve" => {
            run_resolve(&args[2])?;
        }
        "serve" => {
            if args.len() < 6 {
                print_usage();
                std::process::exit(1);
            }

            let service_type_arg = &args[2];
            let instance_name = &args[3];
            let host_name_arg = &args[4];
            let port: u16 = args[5].parse()?;
            let ip = args.get(6).map(|s| s.as_str());

            run_serve(service_type_arg, instance_name, host_name_arg, port, ip)?;
        }
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn run_resolve(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Resolving domain name {}", name);
    let hostname = format!("{}.", name);

    let mdns = ServiceDaemon::new()?;
    let receiver = mdns.resolve_hostname(&hostname, None)?;

    while let Ok(event) = receiver.recv() {
        match event {
            HostnameResolutionEvent::AddressesFound(_, addresses) => {
                println!("Resolution successful!");
                println!("{:?}", addresses);
                break;
            }
            _ => {}
        }
    }

    mdns.shutdown()?;
    Ok(())
}

fn run_serve(
    service_type_arg: &str,
    instance_name: &str,
    host_name_arg: &str,
    port: u16,
    ip: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mdns = ServiceDaemon::new()?;

    let service_type = format!("{}.local.", service_type_arg);
    let service_hostname = format!("{}.local.", host_name_arg);

    let properties: [(&str, &str); 0] = [];

    let mut service_info = ServiceInfo::new(
        &service_type,
        instance_name,
        &service_hostname,
        ip.unwrap_or_default(),
        port,
        &properties[..],
    )?;

    if ip.is_none() {
        service_info = service_info.enable_addr_auto();
    }

    println!("Registering and publishing mDNS service...");
    println!("  {:<15} {}", "Service Type:", service_type);
    println!("  {:<15} {}", "Instance Name:", instance_name);
    println!("  {:<15} {}", "Hostname:", service_hostname);
    println!("  {:<15} {}", "Port:", port);
    println!("  {:<15} {}", "IP:", ip.unwrap_or("Auto-detected"));

    mdns.register(service_info)?;
    println!("Service successfully registered! Press Ctrl+C to exit.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

fn print_usage() {
    eprintln!(
        r#"Usage: cargo run --example tiny-mdns <COMMAND> [ARGS]

Commands:
  resolve <HOSTNAME.local>
      Resolve a .local domain name to an IP address
  serve <SERVICE_TYPE> <INSTANCE_NAME> <HOSTNAME> <PORT> [IP]
      Register an mDNS service in the local network

Examples:
  cargo run --example tiny-mdns resolve my-device.local
  cargo run --example tiny-mdns serve _http._tcp my-web my-device 8000"#
    );
}
