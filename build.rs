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

/// Targets for which bindings will be generated.
const BINDGEN_SUPPORTED_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    // "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-freebsd",
    "aarch64-linux-android",
    "aarch64-unknown-linux-gnu",
    "aarch64-pc-windows-msvc",
    // "aarch64-apple-darwin",
    "armv7-unknown-linux-gnueabihf",
];

fn main() {
    let mut build = cc::Build::new();

    let base = "vendor/cpuinfo";

    build.include(format!("{base}/src"));
    build.include(format!("{base}/include"));

    build.define("CPUINFO_LOG_LEVEL", "2");

    let target = env::var("TARGET").unwrap();

    // Tried to replicate this as much as possible from BUILD.bazel
    #[rustfmt::skip]
    let sources: &[&[&str]] = match target.as_str() {
        "aarch64-apple-darwin" => &[COMMON_SRCS, MACH_SRCS, MACH_ARM_SRCS],
        "aarch64-apple-ios" | "aarch64-apple-ios-sim" => &[COMMON_SRCS, MACH_SRCS, MACH_ARM_SRCS],
        "aarch64-linux-android" => &[COMMON_SRCS, ARM_SRCS, LINUX_SRCS, LINUX_ARM_SRCS, LINUX_ARM64_SRCS, ANDROID_ARM_SRCS],
        "aarch64-pc-windows-msvc" => &[COMMON_SRCS, ARM_SRCS, WINDOWS_ARM_SRCS],
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => &[COMMON_SRCS, ARM_SRCS, LINUX_SRCS, LINUX_ARM_SRCS, LINUX_ARM64_SRCS],
        "arm-unknown-linux-gnueabi" | "arm-unknown-linux-gnueabihf" => &[COMMON_SRCS, ARM_SRCS, LINUX_SRCS, LINUX_ARM_SRCS, LINUX_ARM32_SRCS],
        "armv7-linux-androideabi" => &[COMMON_SRCS, ARM_SRCS, LINUX_SRCS, LINUX_ARM_SRCS, LINUX_ARM32_SRCS, ANDROID_ARM_SRCS],
        "armv7-unknown-linux-gnueabihf" => &[COMMON_SRCS, ARM_SRCS, LINUX_SRCS, LINUX_ARM_SRCS, LINUX_ARM32_SRCS],
        "i686-linux-android" => &[COMMON_SRCS, X86_SRCS, LINUX_SRCS, LINUX_X86_SRCS],
        "riscv64gc-unknown-linux-gnu" => &[COMMON_SRCS, RISCV_SRCS, LINUX_SRCS, LINUX_RISCV_SRCS],
        "s390x-unknown-linux-gnu" => &[COMMON_SRCS, LINUX_SRCS],
        "thumbv7neon-linux-androideabi" => &[COMMON_SRCS, ARM_SRCS, LINUX_SRCS, LINUX_ARM_SRCS, LINUX_ARM32_SRCS, ANDROID_ARM_SRCS],
        "x86_64-apple-darwin" => &[COMMON_SRCS, X86_SRCS, MACH_SRCS, MACH_X86_SRCS],
        "x86_64-apple-ios" => &[COMMON_SRCS, X86_SRCS, MACH_SRCS, MACH_X86_SRCS],
        "x86_64-linux-android" => &[COMMON_SRCS, X86_SRCS, LINUX_SRCS, LINUX_X86_SRCS],
        "x86_64-pc-windows-msvc" | "x86_64-pc-windows-gnu" => &[COMMON_SRCS, X86_SRCS, WINDOWS_X86_SRCS],
        "x86_64-unknown-freebsd" => &[COMMON_SRCS, X86_SRCS, FREEBSD_SRCS, FREEBSD_X86_SRCS],
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => &[COMMON_SRCS, X86_SRCS, LINUX_SRCS, LINUX_X86_SRCS],
        _ => panic!("Unsupported platform {target}"),
    };

    let source_files: Vec<String> = sources
        .iter()
        .flat_map(|i| i.iter())
        .map(|s| format!("{base}/{s}"))
        .collect();

    for source_file in &source_files {
        build.file(source_file);
    }

    if !build.get_compiler().is_like_msvc() {
        build
            .flag("-std=gnu99")
            .flag("-Wno-vla")
            .flag("-D_GNU_SOURCE=1")
            .flag("-DCPUINFO_INTERNAL=")
            .flag("-DCPUINFO_PRIVATE=");
    }

    build.compile("cpuinfo");

    generate_bindings();
}

#[cfg(feature = "generate_bindings")]
fn generate_bindings() {
    for target in BINDGEN_SUPPORTED_TARGETS {
        let t = target.replace("-", "_");
        let output_file = format!("src/bindings_{t}.rs");

        let bindings = bindgen::Builder::default()
            .header("vendor/cpuinfo/include/cpuinfo.h")
            .raw_line("#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]")
            .raw_line("#![allow(dead_code)]")
            .clang_arg(format!("--target={target}"))
            .clang_args(&["-xc++", "-std=c++11"])
            .layout_tests(false)
            .generate()
            .expect("Unable to generate bindings!");

        bindings
            .write_to_file(std::path::Path::new(&output_file))
            .expect("Unable to write bindings!");
    }
}

#[cfg(not(feature = "generate_bindings"))]
fn generate_bindings() {}
