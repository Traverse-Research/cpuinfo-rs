#![allow(dead_code)]

use std::env;

// Source code common to all platforms.
const COMMON_SRCS: &[&str] = &["src/api.c", "src/cache.c", "src/init.c", "src/log.c"];

// Architecture-specific sources and headers.
const X86_SRCS: &[&str] = &[
    "src/x86/cache/descriptor.c",
    "src/x86/cache/deterministic.c",
    "src/x86/cache/init.c",
    "src/x86/info.c",
    "src/x86/init.c",
    "src/x86/isa.c",
    "src/x86/name.c",
    "src/x86/topology.c",
    "src/x86/uarch.c",
    "src/x86/vendor.c",
];

const ARM_SRCS: &[&str] = &["src/arm/cache.c", "src/arm/uarch.c"];

const RISCV_SRCS: &[&str] = &["src/riscv/uarch.c"];

// Platform-specific sources and headers
const LINUX_SRCS: &[&str] = &[
    "src/linux/cpulist.c",
    "src/linux/multiline.c",
    "src/linux/processors.c",
    "src/linux/smallfile.c",
];

const MOCK_LINUX_SRCS: &[&str] = &["src/linux/mockfile.c"];

const MACH_SRCS: &[&str] = &["src/mach/topology.c"];

const FREEBSD_SRCS: &[&str] = &["src/freebsd/topology.c"];

const EMSCRIPTEN_SRCS: &[&str] = &["src/emscripten/init.c"];

const LINUX_X86_SRCS: &[&str] = &["src/x86/linux/cpuinfo.c", "src/x86/linux/init.c"];

const LINUX_ARM_SRCS: &[&str] = &[
    "src/arm/linux/chipset.c",
    "src/arm/linux/clusters.c",
    "src/arm/linux/cpuinfo.c",
    "src/arm/linux/hwcap.c",
    "src/arm/linux/init.c",
    "src/arm/linux/midr.c",
];

// needs LINUX_ARM_SRCS
const LINUX_ARM32_SRCS: &[&str] = &["src/arm/linux/aarch32-isa.c"];

// needs LINUX_ARM_SRCS
const LINUX_ARM64_SRCS: &[&str] = &["src/arm/linux/aarch64-isa.c"];

const LINUX_RISCV_SRCS: &[&str] = &[
    "src/riscv/linux/init.c",
    "src/riscv/linux/riscv-isa.c",
    "src/riscv/linux/riscv-hw.c",
];

const ANDROID_ARM_SRCS: &[&str] = &["src/arm/android/properties.c"];

const WINDOWS_X86_SRCS: &[&str] = &["src/x86/windows/init.c"];

const WINDOWS_ARM_SRCS: &[&str] = &[
    "src/arm/windows/init-by-logical-sys-info.c",
    "src/arm/windows/init.c",
];

const MACH_X86_SRCS: &[&str] = &["src/x86/mach/init.c"];

const MACH_ARM_SRCS: &[&str] = &["src/arm/mach/init.c"];

const FREEBSD_X86_SRCS: &[&str] = &["src/x86/freebsd/init.c"];

fn main() {
    let mut build = cc::Build::new();

    let base = "c:/users/jasper/cpuinfo/";

    build.include(format!("{base}/src"));
    build.include(format!("{base}/include"));

    build.define("CPUINFO_LOG_LEVEL", "2");

    // Add the files we build
    let source_files: Vec<String> = COMMON_SRCS
        .iter()
        .chain(X86_SRCS.iter())
        .chain(WINDOWS_X86_SRCS.iter())
        .map(|s| format!("{base}/{s}"))
        .collect();

    for source_file in &source_files {
        build.file(&source_file);
    }

    let target = env::var("TARGET").unwrap();
    // if target.contains("darwin") {
    //     build
    //         .flag("-std=c++11")
    //         .flag("-Wno-missing-field-initializers")
    //         .flag("-Wno-sign-compare")
    //         .flag("-Wno-deprecated")
    //         .cpp_link_stdlib("c++")
    //         .cpp_set_stdlib("c++")
    //         .cpp(true);
    // } else if target.contains("linux") {
    //     build.flag("-std=c++11").cpp_link_stdlib("stdc++").cpp(true);
    // }

    // build.debug(false).flag("-DNDEBUG").cpp(true);

    build.compile("cpuinfo");

    #[cfg(feature = "generate_bindings")]
    generate_bindings("src/bindings.rs")
}

#[cfg(feature = "generate_bindings")]
fn generate_bindings(output_file: &str) {
    let bindings = bindgen::Builder::default()
        .header("c:/users/jasper/cpuinfo/include/cpuinfo.h")
        .enable_cxx_namespaces()
        .rustfmt_bindings(true)
        .clang_args(&["-xc++", "-std=c++11"])
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings!");

    bindings
        .write_to_file(std::path::Path::new(output_file))
        .expect("Unable to write bindings!");
}

#[cfg(not(feature = "generate_bindings"))]
fn generate_bindings(_: &str) {}
