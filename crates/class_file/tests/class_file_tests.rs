use std::fs::File;

use just_class_file::{AccessFlags, ClassFile, Parser};

fn with_class_file(f: impl FnOnce(ClassFile)) {
    f(
        Parser::new(File::open("tests/classes/my/MyClass.class").unwrap())
            .parse()
            .unwrap(),
    );
}

#[test]
fn test_super_class() {
    with_class_file(|class_file| {
        assert_eq!(Some("java/lang/Object"), class_file.super_class().unwrap())
    });
}

#[test]
fn test_class_name() {
    with_class_file(|class_file| assert_eq!("my/MyClass", class_file.class_name().unwrap()));
}

#[test]
fn test_field_name() {
    with_class_file(|class_file| {
        assert_eq!(
            "myField",
            class_file.field_name(&class_file.fields[0]).unwrap()
        )
    });
}

#[test]
fn test_int_field_type() {
    with_class_file(|class_file| {
        assert_eq!(
            "I",
            class_file.field_descriptor(&class_file.fields[0]).unwrap()
        )
    });
}

#[test]
fn test_field_access_flags() {
    with_class_file(|class_file| {
        assert_eq!(
            AccessFlags::FINAL | AccessFlags::PRIVATE,
            class_file.fields[0].access_flags
        )
    });
}

#[test]
fn test_constructor_name() {
    with_class_file(|class_file| {
        assert_eq!(
            "<init>",
            class_file.method_name(&class_file.methods[0]).unwrap()
        )
    });
}

#[test]
fn test_constructor_descriptor() {
    with_class_file(|class_file| {
        assert_eq!(
            "()V",
            class_file
                .method_descriptor(&class_file.methods[0])
                .unwrap()
        )
    });
}

#[test]
fn test_method_name() {
    with_class_file(|class_file| {
        assert_eq!(
            "add",
            class_file.method_name(&class_file.methods[1]).unwrap()
        )
    });
}

#[test]
fn test_method_descriptor() {
    with_class_file(|class_file| {
        assert_eq!(
            "(I)F",
            class_file
                .method_descriptor(&class_file.methods[1])
                .unwrap()
        )
    });
}

#[test]
fn test_method_access_flags() {
    with_class_file(|class_file| {
        assert_eq!(AccessFlags::PUBLIC, class_file.methods[1].access_flags)
    });
}
