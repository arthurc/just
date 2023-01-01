use std::{env, fs::File, path::PathBuf, process::Command};

use just_jimage::Archive;
use memmap::Mmap;

fn modules_path() -> PathBuf {
    env::var("JAVA_HOME")
        .map(|s| PathBuf::from(s).join("lib/modules"))
        .unwrap()
}

fn with_archive(f: impl FnOnce(Archive<Mmap>)) {
    let file = File::open(modules_path().clone()).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };

    f(Archive::parse(mmap).unwrap());
}

#[test]
fn test_parse_archive_header() {
    let jimage_command = Command::new("jimage")
        .arg("info")
        .arg(modules_path())
        .output()
        .unwrap();

    with_archive(|archive| {
        assert_eq!(
            std::str::from_utf8(&jimage_command.stdout).unwrap(),
            format!("{}", archive.header())
        )
    });
}

#[test]
fn test_read_resource() {
    with_archive(|archive| {
        let object_class = archive
            .by_name("/java.base/java/lang/Object.class")
            .unwrap();

        assert_eq!("java.base", object_class.module());
        assert_eq!("class", object_class.extension());
        assert_eq!("java/lang", object_class.parent());
        assert_eq!("Object", object_class.base());
        assert_eq!(
            "/java.base/java/lang/Object.class",
            object_class.full_name()
        );
    });
}

#[test]
fn test_resource_without_parent() {
    with_archive(|archive| {
        let module_info = archive.by_name("/java.base/module-info.class").unwrap();

        assert_eq!("java.base", module_info.module());
        assert_eq!("class", module_info.extension());
        assert_eq!("", module_info.parent());
        assert_eq!("module-info", module_info.base());
        assert_eq!("/java.base/module-info.class", module_info.full_name());
    });
}
