use std::fs;
use std::path::Path;

use containerd_shim_wasm::sandbox::{Error, oci};
use oci_spec::runtime::Spec;


pub fn load_spec(bundle: String) -> Result<oci::Spec, Error> {
    let mut spec = oci::load(Path::new(&bundle).join("config.json").to_str().unwrap())?;
    spec.canonicalize_rootfs(&bundle)
        .map_err(|e| Error::Others(format!("error canonicalizing rootfs in spec: {}", e)))?;
    Ok(spec)
}

pub fn env_to_wasi(spec: &Spec) -> Vec<String> {
    let default = vec![];
    let env = spec
        .process()
        .as_ref()
        .unwrap()
        .env()
        .as_ref()
        .unwrap_or(&default);
    env.to_vec()
}

pub fn arg_to_wasi(spec: &Spec) -> Vec<String> {
    let default = vec![];
    let args = spec
        .process()
        .as_ref()
        .unwrap()
        .args()
        .as_ref()
        .unwrap_or(&default);
    args.to_vec()
}

pub fn get_wasm_mounts(spec: &Spec) -> Vec<&str> {
    let mounts: Vec<&str> = match spec.mounts() {
        Some(mounts) => mounts
            .iter()
            .filter_map(|mount| {
                if let Some(typ) = mount.typ() {
                    if typ == "bind" || typ == "tmpfs" {
                        return mount.destination().to_str();
                    }
                }
                None
            })
            .collect(),
        _ => vec![],
    };
    mounts
}

pub fn get_wasm_annotations(spec: &Spec,annotation_key: &str) -> String {
    if let Some(map) = &spec.annotations() {
        if !map.is_empty() {
            let my_entry = map.get(annotation_key);
            let value: String = my_entry.map_or_else(String::default, |s| s.to_owned());
            return value;
        }
    }
    return String::new();
}


pub fn delete(bundle_path: String) -> Result<(), Error> {
    let socket_path = format!("{}.sock", bundle_path);

    if Path::new(&socket_path).exists() {
        println!("Attempting to delete: {}", socket_path);

        if let Err(e) = fs::remove_file(&socket_path) {
            eprintln!("Failed to remove socket: {}", e);
            return Err(Error::Others("Failed to remove socket".to_string()));
        }

        println!("Socket deleted successfully.");
    } else {
        println!("Socket not found: {}", socket_path);
    }

    Ok(())
}