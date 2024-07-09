#![doc = include_str!("../README.md")]

mod bindings;
pub use bindings::*;
use std::sync::{Arc, Once};

static INITIALIZED: Once = Once::new();

pub struct CpuInfo;

impl CpuInfo {
    pub fn new() -> Self {
        INITIALIZED.call_once(|| {
            unsafe { root::cpuinfo_initialize() };
        });

        Self {}
    }

    fn uarch(uarch: root::cpuinfo_uarch) -> Uarch {
        Uarch {
            uarch,
            name: uarch_to_string(uarch),
        }
    }

    pub fn uarches(&self) -> Vec<UarchInfo> {
        let count = unsafe { root::cpuinfo_get_uarchs_count() };
        let mut infos = vec![];

        for i in 0..count {
            let uarch_info = unsafe { root::cpuinfo_get_uarch(i) };
            infos.push(unsafe {
                UarchInfo {
                    uarch: Self::uarch((*uarch_info).uarch),
                    cpuid: (*uarch_info).cpuid,
                    processor_count: (*uarch_info).processor_count,
                    core_count: (*uarch_info).core_count,
                }
            })
        }

        infos
    }

    fn package(package: *const root::cpuinfo_package) -> Arc<Package> {
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

    fn cluster(cluster: *const root::cpuinfo_cluster, package: Arc<Package>) -> Arc<Cluster> {
        Arc::new(unsafe {
            Cluster {
                processor_start: (*cluster).processor_start,
                processor_count: (*cluster).processor_count,
                core_start: (*cluster).core_start,
                core_count: (*cluster).core_count,
                cluster_id: (*cluster).cluster_id,
                package: package.clone(),
                vendor: Self::vendor((*cluster).vendor),
                uarch: Self::uarch((*cluster).uarch),
                cpuid: (*cluster).cpuid,
                frequency: (*cluster).frequency,
            }
        })
    }

    fn vendor(vendor: root::cpuinfo_vendor) -> Vendor {
        Vendor {
            vendor,
            name: vendor_to_string(vendor),
        }
    }

    fn core(
        core: *const root::cpuinfo_core,
        cluster: Arc<Cluster>,
        package: Arc<Package>,
    ) -> Arc<Core> {
        Arc::new(unsafe {
            Core {
                processor_start: (*core).processor_start,
                processor_count: (*core).processor_count,
                core_id: (*core).core_id,
                cluster,
                package,
                vendor: Self::vendor((*core).vendor),
                uarch: Self::uarch((*core).uarch),
                cpuid: (*core).cpuid,
                frequency: (*core).frequency,
            }
        })
    }

    pub fn cores(&self) -> Vec<Arc<Core>> {
        let count = unsafe { root::cpuinfo_get_cores_count() };
        let mut cores = vec![];

        for i in 0..count {
            let core = unsafe { root::cpuinfo_get_core(i) };
            let package = Self::package(unsafe { (*core).package });
            let cluster = Self::cluster(unsafe { (*core).cluster }, package.clone());

            cores.push(Self::core(core, cluster.clone(), package.clone()))
        }

        cores
    }

    fn cache(cache: *const root::cpuinfo_cache) -> Option<Cache> {
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

    fn cache_info(cache_info: &root::cpuinfo_processor__bindgen_ty_1) -> CacheInfo {
        unsafe {
            CacheInfo {
                l1i: Self::cache(cache_info.l1i),
                l1d: Self::cache(cache_info.l1d),
                l2: Self::cache(cache_info.l2),
                l3: Self::cache(cache_info.l3),
                l4: Self::cache(cache_info.l4),
            }
        }
    }

    pub fn processors(&self) -> Vec<Processor> {
        let count = unsafe { root::cpuinfo_get_processors_count() };
        let mut processors = vec![];

        for i in 0..count {
            let processor = unsafe { root::cpuinfo_get_processor(i) };
            let package = Self::package(unsafe { (*processor).package });
            let cluster = Self::cluster(unsafe { (*processor).cluster }, package.clone());
            let core = Self::core(
                unsafe { (*processor).core },
                cluster.clone(),
                package.clone(),
            );

            processors.push(unsafe {
                Processor {
                    smt_id: (*processor).smt_id,
                    core,
                    cluster,
                    package,
                    windows_group_id: (*processor).windows_group_id,
                    windows_processor_id: (*processor).windows_processor_id,
                    apic_id: (*processor).apic_id,
                    cache: Self::cache_info(&(*processor).cache),
                }
            })
        }

        processors
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Processor {
    #[doc = " SMT (hyperthread) ID within a core"]
    pub smt_id: u32,
    #[doc = " Core containing this logical processor"]
    pub core: Arc<Core>,
    #[doc = " Cluster of cores containing this logical processor"]
    pub cluster: Arc<Cluster>,
    #[doc = " Physical package containing this logical processor"]
    pub package: Arc<Package>,
    #[doc = " Windows-specific ID for the group containing the logical processor."]
    pub windows_group_id: u16,
    #[doc = " Windows-specific ID of the logical processor within its group:\n - Bit <windows_processor_id> in the KAFFINITY mask identifies this\n logical processor within its group."]
    pub windows_processor_id: u16,
    #[doc = " APIC ID (unique x86-specific ID of the logical processor)"]
    pub apic_id: u32,
    pub cache: CacheInfo,
}

#[repr(C)]
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Vendor {
    vendor: root::cpuinfo_vendor,
    name: &'static str,
}

#[derive(Debug, Clone)]
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
    #[doc = " Value of CPUID leaf 1 EAX register for this core"]
    pub cpuid: u32,
    #[doc = " Clock rate (non-Turbo) of the core, in Hz"]
    pub frequency: u64,
}

#[derive(Debug, Clone)]
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
    #[doc = " Value of CPUID leaf 1 EAX register of the cores in the cluster"]
    pub cpuid: u32,
    #[doc = " Clock rate (non-Turbo) of the cores in the cluster, in Hz"]
    pub frequency: u64,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Uarch {
    #[doc = " Type of CPU microarchitecture"]
    pub uarch: root::cpuinfo_uarch,

    #[doc = " Type of CPU microarchitecture as text"]
    pub name: &'static str,
}

#[derive(Debug, Clone)]
pub struct UarchInfo {
    #[doc = " Type of CPU microarchitecture"]
    pub uarch: Uarch,
    #[doc = " Value of CPUID leaf 1 EAX register for the microarchitecture"]
    pub cpuid: u32,
    #[doc = " Number of logical processors with the microarchitecture"]
    pub processor_count: u32,
    #[doc = " Number of cores with the microarchitecture"]
    pub core_count: u32,
}

pub fn vendor_to_string(vendor: root::cpuinfo_vendor) -> &'static str {
    match vendor {
        root::cpuinfo_vendor_cpuinfo_vendor_unknown => "unknown",
        root::cpuinfo_vendor_cpuinfo_vendor_intel => "Intel",
        root::cpuinfo_vendor_cpuinfo_vendor_amd => "AMD",
        root::cpuinfo_vendor_cpuinfo_vendor_huawei => "Huawei",
        root::cpuinfo_vendor_cpuinfo_vendor_hygon => "Hygon",
        root::cpuinfo_vendor_cpuinfo_vendor_arm => "ARM",
        root::cpuinfo_vendor_cpuinfo_vendor_qualcomm => "Qualcomm",
        root::cpuinfo_vendor_cpuinfo_vendor_apple => "Apple",
        root::cpuinfo_vendor_cpuinfo_vendor_samsung => "Samsung",
        root::cpuinfo_vendor_cpuinfo_vendor_nvidia => "Nvidia",
        root::cpuinfo_vendor_cpuinfo_vendor_mips => "MIPS",
        root::cpuinfo_vendor_cpuinfo_vendor_ibm => "IBM",
        root::cpuinfo_vendor_cpuinfo_vendor_ingenic => "Ingenic",
        root::cpuinfo_vendor_cpuinfo_vendor_via => "VIA",
        root::cpuinfo_vendor_cpuinfo_vendor_cavium => "Cavium",
        root::cpuinfo_vendor_cpuinfo_vendor_broadcom => "Broadcom",
        root::cpuinfo_vendor_cpuinfo_vendor_apm => "Applied Micro",
        _ => "<unknown>",
    }
}

pub fn uarch_to_string(uarch: root::cpuinfo_uarch) -> &'static str {
    match uarch {
        root::cpuinfo_uarch_cpuinfo_uarch_unknown => "unknown",
        root::cpuinfo_uarch_cpuinfo_uarch_p5 => "P5",
        root::cpuinfo_uarch_cpuinfo_uarch_quark => "Quark",
        root::cpuinfo_uarch_cpuinfo_uarch_p6 => "P6",
        root::cpuinfo_uarch_cpuinfo_uarch_dothan => "Dothan",
        root::cpuinfo_uarch_cpuinfo_uarch_yonah => "Yonah",
        root::cpuinfo_uarch_cpuinfo_uarch_conroe => "Conroe",
        root::cpuinfo_uarch_cpuinfo_uarch_penryn => "Penryn",
        root::cpuinfo_uarch_cpuinfo_uarch_nehalem => "Nehalem",
        root::cpuinfo_uarch_cpuinfo_uarch_sandy_bridge => "Sandy Bridge",
        root::cpuinfo_uarch_cpuinfo_uarch_ivy_bridge => "Ivy Bridge",
        root::cpuinfo_uarch_cpuinfo_uarch_haswell => "Haswell",
        root::cpuinfo_uarch_cpuinfo_uarch_broadwell => "Broadwell",
        root::cpuinfo_uarch_cpuinfo_uarch_sky_lake => "Sky Lake",
        root::cpuinfo_uarch_cpuinfo_uarch_palm_cove => "Palm Cove",
        root::cpuinfo_uarch_cpuinfo_uarch_sunny_cove => "Sunny Cove",
        root::cpuinfo_uarch_cpuinfo_uarch_willamette => "Willamette",
        root::cpuinfo_uarch_cpuinfo_uarch_prescott => "Prescott",
        root::cpuinfo_uarch_cpuinfo_uarch_bonnell => "Bonnell",
        root::cpuinfo_uarch_cpuinfo_uarch_saltwell => "Saltwell",
        root::cpuinfo_uarch_cpuinfo_uarch_silvermont => "Silvermont",
        root::cpuinfo_uarch_cpuinfo_uarch_airmont => "Airmont",
        root::cpuinfo_uarch_cpuinfo_uarch_goldmont => "Goldmont",
        root::cpuinfo_uarch_cpuinfo_uarch_goldmont_plus => "Goldmont Plus",
        root::cpuinfo_uarch_cpuinfo_uarch_knights_ferry => "Knights Ferry",
        root::cpuinfo_uarch_cpuinfo_uarch_knights_corner => "Knights Corner",
        root::cpuinfo_uarch_cpuinfo_uarch_knights_landing => "Knights Landing",
        root::cpuinfo_uarch_cpuinfo_uarch_knights_hill => "Knights Hill",
        root::cpuinfo_uarch_cpuinfo_uarch_knights_mill => "Knights Mill",
        root::cpuinfo_uarch_cpuinfo_uarch_k5 => "K5",
        root::cpuinfo_uarch_cpuinfo_uarch_k6 => "K6",
        root::cpuinfo_uarch_cpuinfo_uarch_k7 => "K7",
        root::cpuinfo_uarch_cpuinfo_uarch_k8 => "K8",
        root::cpuinfo_uarch_cpuinfo_uarch_k10 => "K10",
        root::cpuinfo_uarch_cpuinfo_uarch_bulldozer => "Bulldozer",
        root::cpuinfo_uarch_cpuinfo_uarch_piledriver => "Piledriver",
        root::cpuinfo_uarch_cpuinfo_uarch_steamroller => "Steamroller",
        root::cpuinfo_uarch_cpuinfo_uarch_excavator => "Excavator",
        root::cpuinfo_uarch_cpuinfo_uarch_zen => "Zen",
        root::cpuinfo_uarch_cpuinfo_uarch_zen2 => "Zen 2",
        root::cpuinfo_uarch_cpuinfo_uarch_zen3 => "Zen 3",
        root::cpuinfo_uarch_cpuinfo_uarch_zen4 => "Zen 4",
        root::cpuinfo_uarch_cpuinfo_uarch_geode => "Geode",
        root::cpuinfo_uarch_cpuinfo_uarch_bobcat => "Bobcat",
        root::cpuinfo_uarch_cpuinfo_uarch_jaguar => "Jaguar",
        root::cpuinfo_uarch_cpuinfo_uarch_puma => "Puma",
        root::cpuinfo_uarch_cpuinfo_uarch_xscale => "XScale",
        root::cpuinfo_uarch_cpuinfo_uarch_arm7 => "ARM7",
        root::cpuinfo_uarch_cpuinfo_uarch_arm9 => "ARM9",
        root::cpuinfo_uarch_cpuinfo_uarch_arm11 => "ARM11",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a5 => "Cortex-A5",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a7 => "Cortex-A7",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a8 => "Cortex-A8",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a9 => "Cortex-A9",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a12 => "Cortex-A12",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a15 => "Cortex-A15",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a17 => "Cortex-A17",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a32 => "Cortex-A32",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a35 => "Cortex-A35",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a53 => "Cortex-A53",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a55r0 => "Cortex-A55r0",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a55 => "Cortex-A55",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a57 => "Cortex-A57",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a65 => "Cortex-A65",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a72 => "Cortex-A72",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a73 => "Cortex-A73",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a75 => "Cortex-A75",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a76 => "Cortex-A76",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a77 => "Cortex-A77",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a78 => "Cortex-A78",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a510 => "Cortex-A510",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a710 => "Cortex-A710",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_a715 => "Cortex-A715",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_x1 => "Cortex-X1",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_x2 => "Cortex-X2",
        root::cpuinfo_uarch_cpuinfo_uarch_cortex_x3 => "Cortex-X3",
        root::cpuinfo_uarch_cpuinfo_uarch_neoverse_n1 => "Neoverse N1",
        root::cpuinfo_uarch_cpuinfo_uarch_neoverse_e1 => "Neoverse E1",
        root::cpuinfo_uarch_cpuinfo_uarch_neoverse_v1 => "Neoverse V1",
        root::cpuinfo_uarch_cpuinfo_uarch_neoverse_n2 => "Neoverse N2",
        root::cpuinfo_uarch_cpuinfo_uarch_neoverse_v2 => "Neoverse V2",
        root::cpuinfo_uarch_cpuinfo_uarch_scorpion => "Scorpion",
        root::cpuinfo_uarch_cpuinfo_uarch_krait => "Krait",
        root::cpuinfo_uarch_cpuinfo_uarch_kryo => "Kryo",
        root::cpuinfo_uarch_cpuinfo_uarch_falkor => "Falkor",
        root::cpuinfo_uarch_cpuinfo_uarch_saphira => "Saphira",
        root::cpuinfo_uarch_cpuinfo_uarch_denver => "Denver",
        root::cpuinfo_uarch_cpuinfo_uarch_denver2 => "Denver 2",
        root::cpuinfo_uarch_cpuinfo_uarch_carmel => "Carmel",
        root::cpuinfo_uarch_cpuinfo_uarch_exynos_m1 => "Exynos M1",
        root::cpuinfo_uarch_cpuinfo_uarch_exynos_m2 => "Exynos M2",
        root::cpuinfo_uarch_cpuinfo_uarch_exynos_m3 => "Exynos M3",
        root::cpuinfo_uarch_cpuinfo_uarch_exynos_m4 => "Exynos M4",
        root::cpuinfo_uarch_cpuinfo_uarch_exynos_m5 => "Exynos M5",
        root::cpuinfo_uarch_cpuinfo_uarch_swift => "Swift",
        root::cpuinfo_uarch_cpuinfo_uarch_cyclone => "Cyclone",
        root::cpuinfo_uarch_cpuinfo_uarch_typhoon => "Typhoon",
        root::cpuinfo_uarch_cpuinfo_uarch_twister => "Twister",
        root::cpuinfo_uarch_cpuinfo_uarch_hurricane => "Hurricane",
        root::cpuinfo_uarch_cpuinfo_uarch_monsoon => "Monsoon",
        root::cpuinfo_uarch_cpuinfo_uarch_mistral => "Mistral",
        root::cpuinfo_uarch_cpuinfo_uarch_vortex => "Vortex",
        root::cpuinfo_uarch_cpuinfo_uarch_tempest => "Tempest",
        root::cpuinfo_uarch_cpuinfo_uarch_lightning => "Lightning",
        root::cpuinfo_uarch_cpuinfo_uarch_thunder => "Thunder",
        root::cpuinfo_uarch_cpuinfo_uarch_firestorm => "Firestorm",
        root::cpuinfo_uarch_cpuinfo_uarch_icestorm => "Icestorm",
        root::cpuinfo_uarch_cpuinfo_uarch_avalanche => "Avalanche",
        root::cpuinfo_uarch_cpuinfo_uarch_blizzard => "Blizzard",
        root::cpuinfo_uarch_cpuinfo_uarch_thunderx => "ThunderX",
        root::cpuinfo_uarch_cpuinfo_uarch_thunderx2 => "ThunderX2",
        root::cpuinfo_uarch_cpuinfo_uarch_pj4 => "PJ4",
        root::cpuinfo_uarch_cpuinfo_uarch_brahma_b15 => "Brahma B15",
        root::cpuinfo_uarch_cpuinfo_uarch_brahma_b53 => "Brahma B53",
        root::cpuinfo_uarch_cpuinfo_uarch_xgene => "X-Gene",
        root::cpuinfo_uarch_cpuinfo_uarch_dhyana => "Dhyana",
        root::cpuinfo_uarch_cpuinfo_uarch_taishan_v110 => "TaiShan v110",
        _ => "<unknown>",
    }
}
