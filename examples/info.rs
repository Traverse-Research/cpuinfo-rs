use cpuinfo_rs::CpuInfo;

fn main() {
    let info = CpuInfo::new();
    dbg!(info.processors());
}
