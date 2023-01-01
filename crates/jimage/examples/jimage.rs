use std::{env, fs::File, path::PathBuf};

use just_jimage::Archive;
use memmap::Mmap;

fn main() {
    pretty_env_logger::init();

    let path = env::var("JAVA_HOME")
        .map(|s| PathBuf::from(s).join("lib/modules"))
        .unwrap();
    let file = File::open(path).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };

    let archive = Archive::parse(&mmap).unwrap();

    println!("Header:");
    print!("{}", archive.header());
    println!();

    let mut names = archive
        .resources()
        .map(|r| r.full_name())
        .collect::<Vec<_>>();
    names.sort();

    let mut old_module = String::from("");
    for name in names {
        let Some(resource) = archive.by_name(&name) else {
            log::warn!("Resource not found: {}", name);
            continue;
        };

        let module = resource.module();
        if should_skip_module(module) {
            continue;
        }

        if old_module != module {
            println!();
            println!("Module: {}", module);
            old_module = module.to_owned();
        }

        println!("    {}", strip_module(&name));
    }
}

fn strip_module(full_name: &str) -> &str {
    full_name.splitn(3, "/").skip(2).next().unwrap()
}

fn should_skip_module(module: &str) -> bool {
    ["", "modules", "packages"].contains(&module)
}
