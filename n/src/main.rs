use bootloader_locator;
use core::slice;
use fork::{daemon, Fork};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Seek, Stdin, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs, thread};
use std::{
    error::Error,
    process::{exit, Command, Stdio},
};

fn main() -> Result<(), Box<Box<dyn Error>>> {
    println!("Linfinity AdenOS Build Tool");
    println!("<==============================>");
    println!(
        "Working dir: {}",
        env::current_dir().unwrap().to_str().unwrap()
    );

    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).cloned().unwrap_or(String::from("build"));
    let subcommand = subcommand.as_str();

    match subcommand {
        "build" => {
            build(false, None);
        }
        "run" => {
            let (bootable_image_path, system_image_path) = build(false, None);
            run(
                bootable_image_path,
                system_image_path,
                false,
                false,
                false,
                &args[2..],
            )
        }
        "kvm" => {
            let (bootable_image_path, system_image_path) = build(false, None);
            run(
                bootable_image_path,
                system_image_path,
                true,
                false,
                false,
                &args[2..],
            )
        }
        "debug" => {
            let (bootable_image_path, system_image_path) = build(false, None);
            run(
                bootable_image_path,
                system_image_path,
                false,
                true,
                false,
                &args[2..],
            )
        }
        "test" => test(args.get(2).cloned()),
        "flash" => flash(args.get(2).cloned().expect("Please provide device name")),
        _ => println!("Usage: n [build|debug|run|kvm|test|flash]"),
    };

    Ok(())
}

fn build_system_image(kernel_output_folder: &PathBuf, size_bytes: usize) -> String {
    let mut sysimage_output_path = kernel_output_folder.clone();
    sysimage_output_path.push("sysimage_fs.bin");
    let sysimage_fs_file = sysimage_output_path.clone();
    let sysimage_fs_file = sysimage_fs_file.to_str().unwrap();
    sysimage_output_path.pop();
    sysimage_output_path.push("sysimage.img");
    let sysimage_output_file = sysimage_output_path.to_str().unwrap();

    println!("Sysimage {}", sysimage_output_file);
    fs::write(sysimage_fs_file, vec![0 as u8; size_bytes - 1048576])
        .expect("Unable to create sysimage_fs file");
    fs::write(sysimage_output_file, vec![0 as u8; size_bytes])
        .expect("Unable to create sysimage file");

    let mkfs_command = Command::new("mkfs.vfat")
        .arg("-F32")
        .args(["-n", "AdenOS"])
        .arg(sysimage_fs_file)
        .status()
        .expect("Could not launch command 'mkfs.vfat'");
    if !mkfs_command.success() {
        panic!(
            "Command 'mkfs.vfat' failed: {}",
            mkfs_command.code().unwrap()
        );
    }

    let mut gpt_disk = gpt::GptConfig::new()
        .writable(true)
        .create(sysimage_output_path.clone())
        .expect(format!("Unable to open sysimage file: '{}'", sysimage_output_file).as_str());
    let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
        u32::try_from((size_bytes / 512) - 1).unwrap_or(0xFFFFFFFF),
    );
    mbr.overwrite_lba0(gpt_disk.device_mut())
        .expect("Could not create Protective MBR");
    gpt_disk
        .update_partitions(std::collections::BTreeMap::<u32, gpt::partition::Partition>::new())
        .expect("Failed to initialize blank partition table");
    gpt_disk
        .add_partition(
            "adenfs",
            (size_bytes as u64) - 1048576,
            gpt::partition_types::LINUX_FS,
            0,
            None,
        )
        .expect("Could not create sysimage partition");
    let gpt_partition = gpt_disk.partitions().get(&1).unwrap();
    let gpt_partition_offset = gpt_partition
        .bytes_start(gpt::disk::LogicalBlockSize::Lb512)
        .unwrap();

    let gpt_device = gpt_disk.device_mut();
    gpt_device
        .seek(std::io::SeekFrom::Start(gpt_partition_offset))
        .expect("Could not seek to partition offset");
    let mut fs_image = vec![0 as u8; size_bytes - 1048576];
    {
        let fs_file =
            fs::File::open(sysimage_fs_file).expect("Could not open sysimmage_fs file for reading");
        let mut fs_reader = BufReader::new(fs_file);
        fs_reader
            .seek(std::io::SeekFrom::Start(0))
            .expect("Could not seek to sysimage_fs begin");
        fs_reader
            .read_exact(&mut fs_image)
            .expect("Could not read sysimage_fs");
    }
    gpt_device
        .write_all(&fs_image)
        .expect("Could not write filesystem to sysimage partition");

    gpt_disk
        .write()
        .expect(format!("Cannot write partition table to '{}'", sysimage_output_file).as_str());

    let mut udisksctl_command = Command::new("udisksctl")
        .arg("loop-setup")
        .args(["-f", sysimage_output_file])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Could not launch command: 'udisksctl'");

    let udisksctl_stdout = udisksctl_command
        .stdout
        .as_mut()
        .expect("Could not capture output from command 'udisksctl'");
    let mut udisksctl_reader = BufReader::new(udisksctl_stdout);
    let mut loopback_path = String::new();
    udisksctl_reader
        .read_line(&mut loopback_path)
        .expect("Could not read output from command 'udisksctl'");
    let loopback_path = loopback_path.split(" as ").collect::<Vec<&str>>();
    let loopback_device = String::from(&loopback_path[1][..loopback_path[1].len() - 2]);
    let loopback_partition = loopback_device.clone() + "p1";

    let mut udisksctl_command = Command::new("udisksctl")
        .arg("mount")
        .args(["-b", loopback_partition.as_str()])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Could not launch command: 'udisksctl'");

    let udisksctl_stdout = udisksctl_command
        .stdout
        .as_mut()
        .expect("Could not capture output from command 'udisksctl'");
    let mut udisksctl_reader = BufReader::new(udisksctl_stdout);
    let mut mount_path = String::new();
    udisksctl_reader
        .read_line(&mut mount_path)
        .expect("Could not read output from command 'udisksctl'");
    let mount_path = mount_path.split(" at ").collect::<Vec<&str>>();
    let mount_path = PathBuf::from(&mount_path[1][..mount_path[1].len() - 1]);

    let mut sysimage_path = env::current_dir().unwrap();
    sysimage_path.push("sysimage");
    copy_dir_all(sysimage_path, mount_path, true)
        .expect("Could not copy sysimage files to sysimage");

    let mut udisksctl_command = Command::new("udisksctl")
        .arg("unmount")
        .args(["-b", loopback_partition.as_str()])
        .stdout(Stdio::piped())
        .status()
        .expect("Could not launch command: 'udisksctl'");
    if !udisksctl_command.success() {
        panic!(
            "Command 'udisksctl' failed: {}",
            udisksctl_command.code().unwrap()
        );
    }

    let udisksctl_command = Command::new("udisksctl")
        .arg("loop-delete")
        .args(["-b", loopback_device.as_str()])
        .stdout(Stdio::piped())
        .status()
        .expect("Could not launch command: 'udisksctl'");
    if !udisksctl_command.success() {
        panic!(
            "Command 'udisksctl' failed: {}",
            udisksctl_command.code().unwrap()
        );
    }

    return sysimage_output_file.to_string();
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>, root: bool) -> io::Result<()> {
    if !root {
        fs::create_dir_all(&dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()), false)?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn build(test: bool, integration: Option<String>) -> (String, String) {
    let bootloader_source_folder = "bootloader";

    let mut kernel_source_folder = env::current_dir().unwrap();
    kernel_source_folder.push("kernel");

    let kernel_build_command = Command::new("cargo")
        .arg("build")
        .args(if test { vec!["--test"] } else { vec![] })
        .args(if let Some(integration) = integration {
            vec![integration]
        } else {
            vec![]
        })
        .current_dir(kernel_source_folder.to_str().unwrap())
        .status()
        .expect("Launch failed: 'cargo build'");
    if !kernel_build_command.success() {
        panic!(
            "Command 'cargo build' exited with code {}",
            kernel_build_command.code().unwrap()
        );
    }

    let mut kernel_manifest_path = kernel_source_folder.clone();
    kernel_manifest_path.push("Cargo.toml");
    let kernel_manifest_path = kernel_manifest_path.to_str().unwrap();

    let mut kernel_target_folder = env::current_dir().unwrap();
    kernel_target_folder.push("target");
    let mut kernel_output_folder = kernel_target_folder.clone();
    kernel_output_folder.push("x86_64-adenos");
    kernel_output_folder.push("debug");
    let mut kernel_binary_path = kernel_output_folder.clone();
    kernel_binary_path.push("adenos");

    let system_image_path = build_system_image(&kernel_output_folder, 16777216);

    env::set_current_dir(bootloader_source_folder).unwrap();

    let bootloader_builder_command = Command::new("cargo")
        .arg("builder")
        .arg("--kernel-manifest")
        .arg(kernel_manifest_path)
        .arg("--kernel-binary")
        .arg(kernel_binary_path)
        .arg("--target-dir")
        .arg(kernel_target_folder)
        .arg("--out-dir")
        .arg(kernel_output_folder.clone())
        .status()
        .expect("Launch failed: 'cargo builder'");
    if !bootloader_builder_command.success() {
        panic!(
            "Command 'cargo builder' exited with status {}",
            bootloader_builder_command.code().unwrap()
        );
    }

    kernel_output_folder.push("boot-bios-adenos.img");
    (
        String::from(kernel_output_folder.to_str().unwrap()),
        system_image_path,
    )
}

fn run(
    bootable_image_path: String,
    system_image_path: String,
    kvm: bool,
    debug: bool,
    test: bool,
    additional_args: &[String],
) {
    let mut qemu_command = Command::new("qemu-system-x86_64");
    qemu_command
        .arg("-machine")
        .arg("q35")
        .arg("-hda")
        .arg(&bootable_image_path)
        .arg("-hdb")
        .arg(system_image_path)
        .arg("-serial")
        .arg("stdio")
        //.arg("-drive").arg(String::from("file=") + &bootable_image_path + ",if=none,id=nvm")
        //.arg("-device").arg("nvme,serial=deadbeef,drive=nvm")
        .args(if kvm { vec!["-enable-kvm"] } else { vec![] })
        .args(if debug { vec!["-s", "-S"] } else { vec![] })
        .args(if test {
            vec![
                "-device",
                "isa-debug-exit,iobase=0xf4,iosize=0x04",
                "-display",
                "none",
            ]
        } else {
            vec![]
        })
        .arg("-d")
        .arg("int")
        .args(additional_args);
    if debug {
        if let Ok(Fork::Child) = daemon(false, false) {
            let qemu_command_status = qemu_command
                .status()
                .expect("Launch failed: 'qemu-system-x86_64'");
            exit(qemu_command_status.code().unwrap());
        }
    } else {
        let qemu_command_status = qemu_command
            .status()
            .expect("Launch failed: 'qemu-system-x86_64'");
        exit(qemu_command_status.code().unwrap());
    }
}

fn test(integration: Option<String>) {
    let (bootable_image_path, system_image_path) = build(true, integration);
    run(
        bootable_image_path,
        system_image_path,
        false,
        false,
        true,
        &[],
    );
}

fn flash(device_path: String) {
    let (bootable_image_path, system_image_path) = build(false, None);
    let sudo_dd_command = Command::new("sudo")
        .arg("dd")
        .arg(format!("if={}", bootable_image_path))
        .arg(format!("of={}", device_path))
        .stdin(Stdio::inherit())
        .status()
        .expect("Launch failed: 'dd'");
    if !sudo_dd_command.success() {
        panic!(
            "Flashing AdenOS image '{}' to device '{}' failed",
            bootable_image_path, device_path
        );
    }
}
