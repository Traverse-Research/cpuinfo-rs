use cpuinfo_rs::CpuInfo;

fn main() {
    let info = CpuInfo::new();
    let _ = std::fs::write("info.txt", format!("{:#?}", info.processors()));
}
