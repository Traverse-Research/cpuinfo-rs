use cpuinfo_rs::CpuInfo;

fn main() {
    let info = CpuInfo::new();
    std::fs::write("info.txt", format!("{:#?}", info.processors()));
}
