use libdopamine;
mod maxrm_mcpatch;
use serde::{Deserialize, Serialize};
use std::{process::{exit, Command}, time::Instant};
use clap::{command, Parser};
use std::io::{self, Write};


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Doesn't print logs
    #[clap(short, long, default_value_t = false)]
    silent: bool,

    /// Stops automatic launch of Minecraft
    #[clap(short, long, default_value_t = false)]
    nolaunch: bool,

    /// Allows patching Minecraft Preview version
    #[clap(short, long, default_value_t = false)]
    preview: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AppxPackage {
    version: String,
    package_family_name: String,
    executable: String,
}

fn runcmd(command: &str) -> String {
    let output = Command::new("powershell.exe")
        .arg("-c")
        .arg(command)
        .output().unwrap();
    if !output.status.success() {
        return String::from("");
    }
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn main() {
    let args = Arguments::parse();
    macro_rules! println { () => {}; ($($arg:tt)*) => { if !args.silent { std::println!($($arg)*); let _ = io::stdout().flush(); } }; }
    macro_rules! print { () => {}; ($($arg:tt)*) => { if !args.silent { std::print!($($arg)*); let _ = io::stdout().flush(); } }; }
    //macro_rules! eprint { () => {}; ($($arg:tt)*) => { if !args.silent { std::eprint!($($arg)*); let _ = io::stdout().flush(); } }; }
    macro_rules! eprintln { () => {}; ($($arg:tt)*) => { if !args.silent { std::eprintln!($($arg)*); let _ = io::stdout().flush(); } }; }

    let package_name = if args.preview {
        "Microsoft.MinecraftWindowsBeta"
    } else {
        "Microsoft.MinecraftUWP"
    };
    let payload = format!("Get-AppxPackage -Name {} | ForEach-Object ", package_name) +
    "{ @{version=$_.Version; package_family_name=$_.PackageFamilyName; " +
    "executable=(Join-Path $_.InstallLocation (Get-AppxPackageManifest $_)" +
    ".Package.Applications.Application.Executable)} } | ConvertTo-Json";
    print!(
        "= Getting Minecraft{} install... ",
        if args.preview { " Preview" } else { "" }
    );
    let output = runcmd(&payload);
    if output.is_empty() {
        eprintln!("\n! Minecraft not found");
        exit(1);
    }
    let mcinstall: AppxPackage = match serde_json::from_str(&output) {
        Ok(data) => data,
        Err(_) => {
            eprintln!("\n! Error while getting Minecraft install");
            exit(1);
        }
    };
    println!("found version {}!", mcinstall.version);

    // Wait for Minecraft
    if !args.nolaunch {
        println!("* Launching Minecraft");
        runcmd(format!("powershell.exe explorer.exe shell:AppsFolder\\{}!App", mcinstall.package_family_name).as_str());
    }
    print!("= Waiting for Minecraft to launch... ");
    let (pid, process) = match libdopamine::process::wait_for_process(&mcinstall.executable.split("\\").last().unwrap()) {
        Ok(res) => res, Err(_) => {
            eprintln!("\n! Couldn't wait for Minecraft (likely OS error)");
            exit(1);
        }
    };
    println!("found at PID {}!", pid);

    // Get module address
    print!("= Waiting for module... ");
    let (module, _) = match libdopamine::module::wait_for_module(process, "Windows.ApplicationModel.Store.dll") {
        Ok(res) => res, Err(_) => {
            eprintln!("\n! Minecraft process was closed");
            let _ = libdopamine::process::close_process_handle(process); exit(1);
        }
    };
    println!("found at PID {:x}!", module.0 as usize);

    let start = Instant::now();
    // Dump module
    print!("= Dumping module... ");
    let (length, data) = match libdopamine::module::dump_module(process, module) {
        Ok(res) => res, Err(_) => {
            eprintln!("\n! Couldn't dump module, did Minecraft close?");
            let _ = libdopamine::process::close_process_handle(process); exit(1);
        }
    };
    println!("done (read {} bytes)!", length);

    // Inject new module data
    print!("= Patching module... ");
    let arch = match maxrm_mcpatch::check_machine(&data) {
        Ok(res) => res, Err(_) => {
            println!("\n! Couldn't find patches for platform, may be unsupported");
            let _ = libdopamine::process::close_process_handle(process); exit(1);
        }
    };
    print!("got architecture {}... ", arch);

    /*
    The reason why we don't check error here is because
    it's guaranteed to be one of the supported architectures
    by the patches. The check is there for external usage of
    the Max-RM hex patches.
    */
    let mut new_data = maxrm_mcpatch::patch_module(&arch, &data).unwrap();
    println!("done!");
    
    print!("= Injecting module... ");
    match libdopamine::module::inject_module(process, module, &mut new_data, true) {
        Ok(_) => {}, Err(_) => {
            eprintln!("\n! Couldn't inject module, did Minecraft close?");
            let _ = libdopamine::process::close_process_handle(process); exit(1);
        }
    };
    println!("done!");

    let end = Instant::now();
    println!("* Took {:.2?} to dump, patch and inject module", end - start);
    println!("* BEAMinject has sucessfully patched Minecraft");
    let _ = libdopamine::process::close_process_handle(process); exit(0);
}
