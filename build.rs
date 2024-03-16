use std::process::Command;

const BUILD_DIR: &str = "target/native";

fn build_lib(src: &str, lib: &str) {
	Command::new("gcc")
		.args(&[src, "-Ofast", "-Werror", "-Wall", "-Wpedantic", "-c", "-o"])
		.arg(&format!("{}/{}.o", BUILD_DIR, lib))
		.status()
		.expect(format!("Failed to compile {}", lib).as_str());

	Command::new("ar")
		.args(&["-r"])
		.arg(&format!("{}/lib{}.a", BUILD_DIR, lib))
		.arg(&format!("{}/{}.o", BUILD_DIR, lib))
		.status()
		.expect(format!("Failed to create archive for {}", lib).as_str());

	println!("cargo:rustc-link-lib=static={}", lib);
	println!("cargo:rerun-if-changed={}", src);
}

fn main() {
	Command::new("mkdir")
		.args(&["-p", BUILD_DIR])
		.status().unwrap();

	build_lib("src/network/netlink.c", "netlink");
	build_lib("src/common/sysfs/inotify.c", "sysfsinotify");

	println!("cargo:rustc-link-search=native={}", BUILD_DIR);
}
