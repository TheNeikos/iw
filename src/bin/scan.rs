use iw;

fn main() {
    for arg in std::env::args().skip(1) {
        let scan_results = iw::scan_wifi(&arg);
        println!("{:#?}", scan_results);
    }
}
