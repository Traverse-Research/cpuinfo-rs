#![doc = include_str!("../README.md")]

#[cfg(all(target_arch = "aarch64", target_os = "android"))]
mod bindings_aarch64_linux_android;
#[cfg(all(target_arch = "aarch64", target_os = "android"))]
use bindings_aarch64_linux_android::*;

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod bindings_aarch64_apple_darwin;
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
use bindings_aarch64_apple_darwin::*;

#[cfg(all(target_arch = "aarch64", target_os = "windows", target_env = "msvc"))]
mod bindings_aarch64_pc_windows_msvc;
#[cfg(all(target_arch = "aarch64", target_os = "windows", target_env = "msvc"))]
use bindings_aarch64_pc_windows_msvc::*;

#[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
mod bindings_aarch64_unknown_linux_gnu;
#[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
use bindings_aarch64_unknown_linux_gnu::*;

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
mod bindings_x86_64_pc_windows_msvc;
#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
use bindings_x86_64_pc_windows_msvc::*;

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
mod bindings_x86_64_unknown_linux_gnu;
#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
use bindings_x86_64_unknown_linux_gnu::*;

#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
mod bindings_x86_64_apple_darwin;
#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
use bindings_x86_64_apple_darwin::*;

#[cfg(all(target_arch = "arm", target_os = "linux", target_env = "gnu"))]
mod bindings_armv7_unknown_linux_gnueabihf;
#[cfg(all(target_arch = "arm", target_os = "linux", target_env = "gnu"))]
use bindings_armv7_unknown_linux_gnueabihf::*;

use std::borrow::Cow;
use std::sync::{Arc, Once};

static INITIALIZED: Once = Once::new();

pub struct CpuInfo;

impl Default for CpuInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuInfo {
    pub fn new() -> Self {
        INITIALIZED.call_once(|| {
            unsafe { cpuinfo_initialize() };
        });

        Self {}
    }

    fn uarch(uarch: cpuinfo_uarch) -> Uarch {
        Uarch {
            uarch,
            name: uarch_to_string(uarch).into(),
        }
    }

    pub fn uarches(&self) -> Vec<UarchInfo> {
        let count = unsafe { cpuinfo_get_uarchs_count() };
        let mut infos = vec![];

        for i in 0..count {
            let uarch_info = unsafe { cpuinfo_get_uarch(i) };
            infos.push(unsafe {
                #[cfg(target_arch = "x86_64")]
                let cpuid = Some((*uarch_info).cpuid);
                #[cfg(not(target_arch = "x86_64"))]
                let cpuid = None;

                #[cfg(target_arch = "aarch64")]
                let midr = Some((*uarch_info).midr);
                #[cfg(not(target_arch = "aarch64"))]
                let midr = None;

                UarchInfo {
                    uarch: Self::uarch((*uarch_info).uarch),
                    cpuid,
                    midr,
                    processor_count: (*uarch_info).processor_count,
                    core_count: (*uarch_info).core_count,
                }
            })
        }

        infos
    }

    fn package(package: *const cpuinfo_package) -> Arc<Package> {
        Arc::new(unsafe {
            Package {
                name: std::ffi::CStr::from_bytes_until_nul(bytemuck::cast_slice(
                    &(*package).name[..],
                ))
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
                processor_start: (*package).processor_start,
                processor_count: (*package).processor_count,
                core_start: (*package).core_start,
                core_count: (*package).core_count,
                cluster_start: (*package).cluster_start,
                cluster_count: (*package).cluster_count,
            }
        })
    }

    fn cluster(cluster: *const cpuinfo_cluster, package: Arc<Package>) -> Arc<Cluster> {
        Arc::new(unsafe {
            #[cfg(target_arch = "x86_64")]
            let cpuid = Some((*cluster).cpuid);
            #[cfg(not(target_arch = "x86_64"))]
            let cpuid = None;

            #[cfg(target_arch = "aarch64")]
            let midr = Some((*cluster).midr);
            #[cfg(not(target_arch = "aarch64"))]
            let midr = None;

            Cluster {
                processor_start: (*cluster).processor_start,
                processor_count: (*cluster).processor_count,
                core_start: (*cluster).core_start,
                core_count: (*cluster).core_count,
                cluster_id: (*cluster).cluster_id,
                package: package.clone(),
                vendor: Self::vendor((*cluster).vendor),
                uarch: Self::uarch((*cluster).uarch),
                cpuid,
                midr,
                frequency: (*cluster).frequency,
            }
        })
    }

    fn vendor(vendor: cpuinfo_vendor) -> Vendor {
        Vendor {
            vendor,
            name: vendor_to_string(vendor).into(),
        }
    }

    fn core(core: *const cpuinfo_core, cluster: Arc<Cluster>, package: Arc<Package>) -> Arc<Core> {
        Arc::new(unsafe {
            Core {
                cpuid: cluster.cpuid,
                midr: cluster.midr,

                processor_start: (*core).processor_start,
                processor_count: (*core).processor_count,
                core_id: (*core).core_id,
                cluster,
                package,
                vendor: Self::vendor((*core).vendor),
                uarch: Self::uarch((*core).uarch),
                frequency: (*core).frequency,
            }
        })
    }

    pub fn cores(&self) -> Vec<Arc<Core>> {
        let count = unsafe { cpuinfo_get_cores_count() };
        let mut cores = vec![];

        for i in 0..count {
            let core = unsafe { cpuinfo_get_core(i) };
            let package = Self::package(unsafe { (*core).package });
            let cluster = Self::cluster(unsafe { (*core).cluster }, package.clone());

            cores.push(Self::core(core, cluster.clone(), package.clone()))
        }

        cores
    }

    fn cache(cache: *const cpuinfo_cache) -> Option<Cache> {
        if cache.is_null() {
            return None;
        }

        Some(unsafe {
            Cache {
                size: (*cache).size,
                associativity: (*cache).associativity,
                sets: (*cache).sets,
                partitions: (*cache).partitions,
                line_size: (*cache).line_size,
                flags: (*cache).flags,
                processor_start: (*cache).processor_start,
                processor_count: (*cache).processor_count,
            }
        })
    }

    fn cache_info(cache_info: &cpuinfo_processor__bindgen_ty_1) -> CacheInfo {
        CacheInfo {
            l1i: Self::cache(cache_info.l1i),
            l1d: Self::cache(cache_info.l1d),
            l2: Self::cache(cache_info.l2),
            l3: Self::cache(cache_info.l3),
            l4: Self::cache(cache_info.l4),
        }
    }

    pub fn processors(&self) -> Vec<Processor> {
        let count = unsafe { cpuinfo_get_processors_count() };
        let mut processors = vec![];

        for i in 0..count {
            let processor = unsafe { cpuinfo_get_processor(i) };
            let package = Self::package(unsafe { (*processor).package });
            let cluster = Self::cluster(unsafe { (*processor).cluster }, package.clone());
            let core = Self::core(
                unsafe { (*processor).core },
                cluster.clone(),
                package.clone(),
            );

            processors.push(unsafe {
                #[cfg(target_os = "linux")]
                let linux_id = Some((*processor).linux_id);
                #[cfg(not(target_os = "linux"))]
                let linux_id = None;

                #[cfg(target_os = "windows")]
                let (windows_group_id, windows_processor_id) = {
                    (
                        Some((*processor).windows_group_id),
                        Some((*processor).windows_processor_id),
                    )
                };
                #[cfg(not(target_os = "windows"))]
                let (windows_group_id, windows_processor_id) = (None, None);

                #[cfg(target_arch = "x86_64")]
                let apic_id = Some((*processor).apic_id);
                #[cfg(not(target_arch = "x86_64"))]
                let apic_id = None;

                Processor {
                    smt_id: (*processor).smt_id,
                    core,
                    cluster,
                    package,
                    linux_id,
                    windows_group_id,
                    windows_processor_id,
                    apic_id,
                    cache: Self::cache_info(&(*processor).cache),
                }
            })
        }

        processors
    }
}

#[repr(C)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cache {
    #[doc = " Cache size in bytes"]
    pub size: u32,
    #[doc = " Number of ways of associativity"]
    pub associativity: u32,
    #[doc = " Number of sets"]
    pub sets: u32,
    #[doc = " Number of partitions"]
    pub partitions: u32,
    #[doc = " Line size in bytes"]
    pub line_size: u32,
    #[doc = " Binary characteristics of the cache (unified cache, inclusive cache,\n cache with complex indexing).\n\n @see CPUINFO_CACHE_UNIFIED, CPUINFO_CACHE_INCLUSIVE,\n CPUINFO_CACHE_COMPLEX_INDEXING"]
    pub flags: u32,
    #[doc = " Index of the first logical processor that shares this cache"]
    pub processor_start: u32,
    #[doc = " Number of logical processors that share this cache"]
    pub processor_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Processor {
    #[doc = " SMT (hyperthread) ID within a core"]
    pub smt_id: u32,
    #[doc = " Core containing this logical processor"]
    pub core: Arc<Core>,
    #[doc = " Cluster of cores containing this logical processor"]
    pub cluster: Arc<Cluster>,
    #[doc = " Physical package containing this logical processor"]
    pub package: Arc<Package>,
    #[doc = " Linux-specific ID for the logical processor:\n - Linux kernel exposes information about this logical processor in /sys/devices/system/cpu/cpu<linux_id>/ \n - Bit <linux_id> in the cpu_set_t identifies this logical processor"]
    pub linux_id: Option<i32>,
    #[doc = " Windows-specific ID for the group containing the logical processor."]
    pub windows_group_id: Option<u16>,
    #[doc = " Windows-specific ID of the logical processor within its group:\n - Bit <windows_processor_id> in the KAFFINITY mask identifies this\n logical processor within its group."]
    pub windows_processor_id: Option<u16>,
    #[doc = " APIC ID (unique x86-specific ID of the logical processor)"]
    pub apic_id: Option<u32>,
    pub cache: CacheInfo,
}

#[repr(C)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheInfo {
    #[doc = " Level 1 instruction cache"]
    pub l1i: Option<Cache>,
    #[doc = " Level 1 data cache"]
    pub l1d: Option<Cache>,
    #[doc = " Level 2 unified or data cache"]
    pub l2: Option<Cache>,
    #[doc = " Level 3 unified or data cache"]
    pub l3: Option<Cache>,
    #[doc = " Level 4 unified or data cache"]
    pub l4: Option<Cache>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vendor {
    pub vendor: cpuinfo_vendor,
    pub name: Cow<'static, str>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Core {
    #[doc = " Index of the first logical processor on this core."]
    pub processor_start: u32,
    #[doc = " Number of logical processors on this core"]
    pub processor_count: u32,
    #[doc = " Core ID within a package"]
    pub core_id: u32,
    #[doc = " Cluster containing this core"]
    pub cluster: Arc<Cluster>,
    #[doc = " Physical package containing this core."]
    pub package: Arc<Package>,
    #[doc = " Vendor of the CPU microarchitecture for this core"]
    pub vendor: Vendor,
    #[doc = " CPU microarchitecture for this core"]
    pub uarch: Uarch,
    #[doc = " Value of CPUID leaf 1 EAX register for this core (x86-specific ID)"]
    pub cpuid: Option<u32>,
    #[doc = " Value of Main ID Register (MIDR) for this core (arm-specific ID)"]
    pub midr: Option<u32>,
    #[doc = " Clock rate (non-Turbo) of the core, in Hz"]
    pub frequency: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cluster {
    #[doc = " Index of the first logical processor in the cluster"]
    pub processor_start: u32,
    #[doc = " Number of logical processors in the cluster"]
    pub processor_count: u32,
    #[doc = " Index of the first core in the cluster"]
    pub core_start: u32,
    #[doc = " Number of cores on the cluster"]
    pub core_count: u32,
    #[doc = " Cluster ID within a package"]
    pub cluster_id: u32,
    #[doc = " Physical package containing the cluster"]
    pub package: Arc<Package>,
    #[doc = " CPU microarchitecture vendor of the cores in the cluster"]
    pub vendor: Vendor,
    #[doc = " CPU microarchitecture of the cores in the cluster"]
    pub uarch: Uarch,
    #[doc = " Value of CPUID leaf 1 EAX register for this core (x86-specific ID)"]
    pub cpuid: Option<u32>,
    #[doc = " Value of Main ID Register (MIDR) for this core (arm-specific ID)"]
    pub midr: Option<u32>,
    #[doc = " Clock rate (non-Turbo) of the cores in the cluster, in Hz"]
    pub frequency: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Package {
    #[doc = " SoC or processor chip model name"]
    pub name: String,
    #[doc = " Index of the first logical processor on this physical package"]
    pub processor_start: u32,
    #[doc = " Number of logical processors on this physical package"]
    pub processor_count: u32,
    #[doc = " Index of the first core on this physical package"]
    pub core_start: u32,
    #[doc = " Number of cores on this physical package"]
    pub core_count: u32,
    #[doc = " Index of the first cluster of cores on this physical package"]
    pub cluster_start: u32,
    #[doc = " Number of clusters of cores on this physical package"]
    pub cluster_count: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Uarch {
    #[doc = " Type of CPU microarchitecture"]
    pub uarch: cpuinfo_uarch,

    #[doc = " Type of CPU microarchitecture as text"]
    pub name: Cow<'static, str>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UarchInfo {
    #[doc = " Type of CPU microarchitecture"]
    pub uarch: Uarch,
    #[doc = " Value of CPUID leaf 1 EAX register for the microarchitecture"]
    pub cpuid: Option<u32>,
    #[doc = " Value of Main ID Register (MIDR) for this core (arm-specific ID)"]
    pub midr: Option<u32>,
    #[doc = " Number of logical processors with the microarchitecture"]
    pub processor_count: u32,
    #[doc = " Number of cores with the microarchitecture"]
    pub core_count: u32,
}

pub fn vendor_to_string(vendor: cpuinfo_vendor) -> &'static str {
    #[allow(non_upper_case_globals)]
    match vendor {
        cpuinfo_vendor_cpuinfo_vendor_unknown => "unknown",
        cpuinfo_vendor_cpuinfo_vendor_intel => "Intel",
        cpuinfo_vendor_cpuinfo_vendor_amd => "AMD",
        cpuinfo_vendor_cpuinfo_vendor_huawei => "Huawei",
        cpuinfo_vendor_cpuinfo_vendor_hygon => "Hygon",
        cpuinfo_vendor_cpuinfo_vendor_arm => "ARM",
        cpuinfo_vendor_cpuinfo_vendor_qualcomm => "Qualcomm",
        cpuinfo_vendor_cpuinfo_vendor_apple => "Apple",
        cpuinfo_vendor_cpuinfo_vendor_samsung => "Samsung",
        cpuinfo_vendor_cpuinfo_vendor_nvidia => "Nvidia",
        cpuinfo_vendor_cpuinfo_vendor_mips => "MIPS",
        cpuinfo_vendor_cpuinfo_vendor_ibm => "IBM",
        cpuinfo_vendor_cpuinfo_vendor_ingenic => "Ingenic",
        cpuinfo_vendor_cpuinfo_vendor_via => "VIA",
        cpuinfo_vendor_cpuinfo_vendor_cavium => "Cavium",
        cpuinfo_vendor_cpuinfo_vendor_broadcom => "Broadcom",
        cpuinfo_vendor_cpuinfo_vendor_apm => "Applied Micro",
        _ => "<unknown>",
    }
}

pub fn uarch_to_string(uarch: cpuinfo_uarch) -> &'static str {
    #[allow(non_upper_case_globals)]
    match uarch {
        cpuinfo_uarch_cpuinfo_uarch_unknown => "unknown",
        cpuinfo_uarch_cpuinfo_uarch_p5 => "P5",
        cpuinfo_uarch_cpuinfo_uarch_quark => "Quark",
        cpuinfo_uarch_cpuinfo_uarch_p6 => "P6",
        cpuinfo_uarch_cpuinfo_uarch_dothan => "Dothan",
        cpuinfo_uarch_cpuinfo_uarch_yonah => "Yonah",
        cpuinfo_uarch_cpuinfo_uarch_conroe => "Conroe",
        cpuinfo_uarch_cpuinfo_uarch_penryn => "Penryn",
        cpuinfo_uarch_cpuinfo_uarch_nehalem => "Nehalem",
        cpuinfo_uarch_cpuinfo_uarch_sandy_bridge => "Sandy Bridge",
        cpuinfo_uarch_cpuinfo_uarch_ivy_bridge => "Ivy Bridge",
        cpuinfo_uarch_cpuinfo_uarch_haswell => "Haswell",
        cpuinfo_uarch_cpuinfo_uarch_broadwell => "Broadwell",
        cpuinfo_uarch_cpuinfo_uarch_sky_lake => "Sky Lake",
        cpuinfo_uarch_cpuinfo_uarch_palm_cove => "Palm Cove",
        cpuinfo_uarch_cpuinfo_uarch_sunny_cove => "Sunny Cove",
        cpuinfo_uarch_cpuinfo_uarch_willamette => "Willamette",
        cpuinfo_uarch_cpuinfo_uarch_prescott => "Prescott",
        cpuinfo_uarch_cpuinfo_uarch_bonnell => "Bonnell",
        cpuinfo_uarch_cpuinfo_uarch_saltwell => "Saltwell",
        cpuinfo_uarch_cpuinfo_uarch_silvermont => "Silvermont",
        cpuinfo_uarch_cpuinfo_uarch_airmont => "Airmont",
        cpuinfo_uarch_cpuinfo_uarch_goldmont => "Goldmont",
        cpuinfo_uarch_cpuinfo_uarch_goldmont_plus => "Goldmont Plus",
        cpuinfo_uarch_cpuinfo_uarch_knights_ferry => "Knights Ferry",
        cpuinfo_uarch_cpuinfo_uarch_knights_corner => "Knights Corner",
        cpuinfo_uarch_cpuinfo_uarch_knights_landing => "Knights Landing",
        cpuinfo_uarch_cpuinfo_uarch_knights_hill => "Knights Hill",
        cpuinfo_uarch_cpuinfo_uarch_knights_mill => "Knights Mill",
        cpuinfo_uarch_cpuinfo_uarch_k5 => "K5",
        cpuinfo_uarch_cpuinfo_uarch_k6 => "K6",
        cpuinfo_uarch_cpuinfo_uarch_k7 => "K7",
        cpuinfo_uarch_cpuinfo_uarch_k8 => "K8",
        cpuinfo_uarch_cpuinfo_uarch_k10 => "K10",
        cpuinfo_uarch_cpuinfo_uarch_bulldozer => "Bulldozer",
        cpuinfo_uarch_cpuinfo_uarch_piledriver => "Piledriver",
        cpuinfo_uarch_cpuinfo_uarch_steamroller => "Steamroller",
        cpuinfo_uarch_cpuinfo_uarch_excavator => "Excavator",
        cpuinfo_uarch_cpuinfo_uarch_zen => "Zen",
        cpuinfo_uarch_cpuinfo_uarch_zen2 => "Zen 2",
        cpuinfo_uarch_cpuinfo_uarch_zen3 => "Zen 3",
        cpuinfo_uarch_cpuinfo_uarch_zen4 => "Zen 4",
        cpuinfo_uarch_cpuinfo_uarch_geode => "Geode",
        cpuinfo_uarch_cpuinfo_uarch_bobcat => "Bobcat",
        cpuinfo_uarch_cpuinfo_uarch_jaguar => "Jaguar",
        cpuinfo_uarch_cpuinfo_uarch_puma => "Puma",
        cpuinfo_uarch_cpuinfo_uarch_xscale => "XScale",
        cpuinfo_uarch_cpuinfo_uarch_arm7 => "ARM7",
        cpuinfo_uarch_cpuinfo_uarch_arm9 => "ARM9",
        cpuinfo_uarch_cpuinfo_uarch_arm11 => "ARM11",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a5 => "Cortex-A5",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a7 => "Cortex-A7",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a8 => "Cortex-A8",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a9 => "Cortex-A9",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a12 => "Cortex-A12",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a15 => "Cortex-A15",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a17 => "Cortex-A17",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a32 => "Cortex-A32",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a35 => "Cortex-A35",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a53 => "Cortex-A53",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a55r0 => "Cortex-A55r0",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a55 => "Cortex-A55",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a57 => "Cortex-A57",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a65 => "Cortex-A65",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a72 => "Cortex-A72",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a73 => "Cortex-A73",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a75 => "Cortex-A75",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a76 => "Cortex-A76",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a77 => "Cortex-A77",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a78 => "Cortex-A78",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a510 => "Cortex-A510",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a710 => "Cortex-A710",
        cpuinfo_uarch_cpuinfo_uarch_cortex_a715 => "Cortex-A715",
        cpuinfo_uarch_cpuinfo_uarch_cortex_x1 => "Cortex-X1",
        cpuinfo_uarch_cpuinfo_uarch_cortex_x2 => "Cortex-X2",
        cpuinfo_uarch_cpuinfo_uarch_cortex_x3 => "Cortex-X3",
        cpuinfo_uarch_cpuinfo_uarch_neoverse_n1 => "Neoverse N1",
        cpuinfo_uarch_cpuinfo_uarch_neoverse_e1 => "Neoverse E1",
        cpuinfo_uarch_cpuinfo_uarch_neoverse_v1 => "Neoverse V1",
        cpuinfo_uarch_cpuinfo_uarch_neoverse_n2 => "Neoverse N2",
        cpuinfo_uarch_cpuinfo_uarch_neoverse_v2 => "Neoverse V2",
        cpuinfo_uarch_cpuinfo_uarch_scorpion => "Scorpion",
        cpuinfo_uarch_cpuinfo_uarch_krait => "Krait",
        cpuinfo_uarch_cpuinfo_uarch_kryo => "Kryo",
        cpuinfo_uarch_cpuinfo_uarch_falkor => "Falkor",
        cpuinfo_uarch_cpuinfo_uarch_saphira => "Saphira",
        cpuinfo_uarch_cpuinfo_uarch_denver => "Denver",
        cpuinfo_uarch_cpuinfo_uarch_denver2 => "Denver 2",
        cpuinfo_uarch_cpuinfo_uarch_carmel => "Carmel",
        cpuinfo_uarch_cpuinfo_uarch_exynos_m1 => "Exynos M1",
        cpuinfo_uarch_cpuinfo_uarch_exynos_m2 => "Exynos M2",
        cpuinfo_uarch_cpuinfo_uarch_exynos_m3 => "Exynos M3",
        cpuinfo_uarch_cpuinfo_uarch_exynos_m4 => "Exynos M4",
        cpuinfo_uarch_cpuinfo_uarch_exynos_m5 => "Exynos M5",
        cpuinfo_uarch_cpuinfo_uarch_swift => "Swift",
        cpuinfo_uarch_cpuinfo_uarch_cyclone => "Cyclone",
        cpuinfo_uarch_cpuinfo_uarch_typhoon => "Typhoon",
        cpuinfo_uarch_cpuinfo_uarch_twister => "Twister",
        cpuinfo_uarch_cpuinfo_uarch_hurricane => "Hurricane",
        cpuinfo_uarch_cpuinfo_uarch_monsoon => "Monsoon",
        cpuinfo_uarch_cpuinfo_uarch_mistral => "Mistral",
        cpuinfo_uarch_cpuinfo_uarch_vortex => "Vortex",
        cpuinfo_uarch_cpuinfo_uarch_tempest => "Tempest",
        cpuinfo_uarch_cpuinfo_uarch_lightning => "Lightning",
        cpuinfo_uarch_cpuinfo_uarch_thunder => "Thunder",
        cpuinfo_uarch_cpuinfo_uarch_firestorm => "Firestorm",
        cpuinfo_uarch_cpuinfo_uarch_icestorm => "Icestorm",
        cpuinfo_uarch_cpuinfo_uarch_avalanche => "Avalanche",
        cpuinfo_uarch_cpuinfo_uarch_blizzard => "Blizzard",
        cpuinfo_uarch_cpuinfo_uarch_thunderx => "ThunderX",
        cpuinfo_uarch_cpuinfo_uarch_thunderx2 => "ThunderX2",
        cpuinfo_uarch_cpuinfo_uarch_pj4 => "PJ4",
        cpuinfo_uarch_cpuinfo_uarch_brahma_b15 => "Brahma B15",
        cpuinfo_uarch_cpuinfo_uarch_brahma_b53 => "Brahma B53",
        cpuinfo_uarch_cpuinfo_uarch_xgene => "X-Gene",
        cpuinfo_uarch_cpuinfo_uarch_dhyana => "Dhyana",
        cpuinfo_uarch_cpuinfo_uarch_taishan_v110 => "TaiShan v110",
        _ => "<unknown>",
    }
}
