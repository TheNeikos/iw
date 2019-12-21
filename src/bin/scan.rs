use iw;

fn main() {
    let interfaces = iw::interfaces().unwrap();

    for interface in interfaces {
        println!("{:?}", interface.get_name());
        println!("{:?}", interface.get_connected_essid());
    }
}
