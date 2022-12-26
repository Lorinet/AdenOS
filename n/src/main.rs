use std::{process::{Command, exit, Stdio}, error::Error};
use std::env;
use bootloader_locator;

fn main() -> Result<(), Box<Box<dyn Error>>> {
    println!("Linfinity AdenOS Build Tool");
    println!("<==============================>");
    println!("Working dir: {}", env::current_dir().unwrap().to_str().unwrap());

    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).cloned().unwrap_or(String::from("build"));
    let subcommand = subcommand.as_str();

    match subcommand {
        "build" => { build(false, None); },
        "run" => run(build(false, None), false, false, false, &args[2..]),
        "kvm" => run(build(false, None), true, false, false, &args[2..]),
        "debug" => run(build(false, None), false, true, false, &args[2..]),
        "test" => test(args.get(2).cloned()),
        "flash" => flash(args.get(2).cloned().expect("Please provide device name")),
        _ => println!("Usage: n [build|debug|run|kvm|test|flash]"),
    };

    Ok(())
}

fn build(test: bool, integration: Option<String>) -> String {
    let bootloader_source_folder = bootloader_locator::locate_bootloader("bootloader").expect("Could not find manifest for crate 'bootloader'");
    let bootloader_source_folder = bootloader_source_folder.parent().expect("Invalid path for crate 'bootloader'").to_str().unwrap();

    let kernel_build_command = Command::new("cargo").arg("build")
    .args(if test { vec!["--test"] } else { vec![] })
    .args(if let Some(integration) = integration { vec![integration] } else { vec![] })
    .status().expect("Launch failed: 'cargo build'");
    if !kernel_build_command.success() {
        panic!("Command 'cargo build' exited with code {}", kernel_build_command.code().unwrap());
    }

    let mut kernel_manifest_path = env::current_dir().unwrap();
    kernel_manifest_path.push("Cargo.toml");
    let kernel_manifest_path = kernel_manifest_path.to_str().unwrap();

    let mut kernel_target_folder = env::current_dir().unwrap();
    kernel_target_folder.push("target");
    let mut kernel_output_folder = kernel_target_folder.clone();
    kernel_output_folder.push("x86_64-adenos");
    kernel_output_folder.push("debug");
    let mut kernel_binary_path = kernel_output_folder.clone();
    kernel_binary_path.push("adenos");

    env::set_current_dir(bootloader_source_folder).unwrap();

    let bootloader_builder_command = Command::new("cargo").arg("builder")
    .arg("--kernel-manifest").arg(kernel_manifest_path)
    .arg("--kernel-binary").arg(kernel_binary_path)
    .arg("--target-dir").arg(kernel_target_folder)
    .arg("--out-dir").arg(kernel_output_folder.clone()).status().expect("Launch failed: 'cargo builder'");
    if !bootloader_builder_command.success() {
        panic!("Command 'cargo builder' exited with status {}", bootloader_builder_command.code().unwrap());
    }

    let mut second_drive_img_path = kernel_output_folder.clone();
    second_drive_img_path.push("drive_2.img");

    kernel_output_folder.push("boot-bios-adenos.img");
    String::from(kernel_output_folder.to_str().unwrap())
}

fn run(bootable_image_path: String, kvm: bool, debug: bool, test: bool, additional_args: &[String]) {
    let qemu_command = Command::new("qemu-system-x86_64")
    .arg("-machine").arg("q35")
    .arg("-hda").arg(&bootable_image_path)
    .arg("-hdb").arg("/home/linfinity/bigfat.img")
    .arg("-serial").arg("stdio")
    //.arg("-drive").arg(String::from("file=") + &bootable_image_path + ",if=none,id=nvm")
    //.arg("-device").arg("nvme,serial=deadbeef,drive=nvm")
    .args(if kvm { vec!["-enable-kvm"] } else { vec![] })
    .args(if debug { vec!["-s", "-S"] } else { vec![] })
    .args(if test { vec!["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04", "-display", "none"] } else { vec![] })
    .arg("-d").arg("int")
    .args(additional_args).status().expect("Launch failed: 'qemu-system-x86_64'");
    exit(qemu_command.code().unwrap());
}

fn test(integration: Option<String>) {
    let bootable_image_path = build(true, integration);
    run(bootable_image_path, false, false, true, &[]);
}

fn flash(device_path: String) {
    let bootable_image_path = build(false, None);
    let sudo_dd_command = Command::new("sudo").arg("dd")
    .arg(format!("if={}", bootable_image_path))
    .arg(format!("of={}", device_path))
    .stdin(Stdio::inherit()).status().expect("Launch failed: 'dd'");
    if !sudo_dd_command.success() {
        panic!("Flashing AdenOS image '{}' to device '{}' failed", bootable_image_path, device_path);
    }
}