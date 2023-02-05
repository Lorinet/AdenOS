use crate::*;

pub fn probe_partitions() -> Result<(), Error> {
    for (_name, _dev) in namespace::namespace().iter_mut_bf() {

    }
    Ok(())
}