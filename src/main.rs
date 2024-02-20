fn main() {
    println!("Hello, world!");
    let file = netcdf::open("test_data/KTLX20240124_213824_V06");
    let var = &file.unwrap().variable("data").expect("Could not find variable 'data'");
}
