use crate::{dev::Error, devices};

pub fn probe_partitions() -> Result<(), Error> {
    for (_name, _dev) in devices::device_tree().iter_mut_bf() {

    }
    Ok(())
}