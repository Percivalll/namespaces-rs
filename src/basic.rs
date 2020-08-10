use anyhow;
use regex;
#[derive(Debug, Clone)]
pub enum NsType {
    CGROUP,
    PID,
    USER,
    UTS,
    IPC,
    MNT,
    NET,
    UNK,
}
#[derive(Debug, Clone)]
pub struct Namespace {
    genre: NsType,
    procs: Vec<String>,
}
impl Namespace {
    pub fn new(genre: String, proc: String) -> Self {
        let genre = match genre.as_str() {
            "cgroup" => NsType::CGROUP,
            "pid" => NsType::PID,
            "user" => NsType::USER,
            "uts" => NsType::UTS,
            "ipc" => NsType::IPC,
            "mnt" => NsType::MNT,
            "net" => NsType::NET,
            _ => NsType::UNK,
        };
        let procs: Vec<String> = vec![proc];
        Namespace { genre, procs }
    }
    pub fn get_fd(&self) -> Option<i32> {
        for i in &self.procs {
            unsafe {
                let fd = libc::open(
                    std::ffi::CString::new(i.as_str()).unwrap().as_ptr() as *const libc::c_char,
                    libc::O_RDONLY,
                );
                if fd != -1 {
                    return Some(fd);
                }
            }
        }
        None
    }
}
pub type Namespaces = std::collections::HashMap<String, Namespace>;

pub fn get_namespaces() -> anyhow::Result<Namespaces> {
    let usize_pattern = regex::Regex::new(r"[1-9]\d*").unwrap();
    let mut namespaces = Namespaces::new();
    for proc_dir in std::fs::read_dir("/proc")?.filter_map(|f| match f {
        Ok(f) => match f.file_name().to_str().unwrap_or_default().parse::<u64>() {
            Ok(_) => Some(f),
            Err(_) => None,
        },
        Err(_) => None,
    }) {
        let ns_dir = match std::fs::read_dir(proc_dir.path().to_str().unwrap().to_string() + "/ns")
        {
            Ok(dir) => dir,
            Err(_) => continue,
        };
        for ns_file in ns_dir.filter_map(|f| match f {
            Ok(f) => Some(f),
            Err(_) => None,
        }) {
            let index = match std::fs::read_link(ns_file.path()) {
                Ok(link) => usize_pattern
                    .find(link.to_str().unwrap_or_default())
                    .unwrap()
                    .as_str()
                    .to_owned(),
                Err(_) => continue,
            };
            if namespaces.contains_key(&index) {
                namespaces
                    .get_mut(&index)
                    .unwrap()
                    .procs
                    .push(ns_file.path().to_str().unwrap_or_default().to_string());
            } else {
                namespaces.insert(
                    index,
                    Namespace::new(
                        ns_file.file_name().to_str().unwrap_or_default().to_string(),
                        ns_file.path().to_str().unwrap_or_default().to_string(),
                    ),
                );
            }
        }
    }
    Ok(namespaces)
}

pub fn get_specific_namespaces(genre: NsType) -> anyhow::Result<Namespaces> {
    let usize_pattern = regex::Regex::new(r"[1-9]\d*").unwrap();
    let mut namespaces = Namespaces::new();
    let ns_type = format!("{:?}", genre).to_lowercase();
    for proc_dir in std::fs::read_dir("/proc")?.filter_map(|f| match f {
        Ok(f) => match f.file_name().to_str().unwrap_or_default().parse::<u64>() {
            Ok(_) => Some(f),
            Err(_) => None,
        },
        Err(_) => None,
    }) {
        let index;
        if proc_dir.file_name().to_str().unwrap() == "1" {
            index = "origin".to_owned()
        } else {
            index = match std::fs::read_link(
                proc_dir.path().to_str().unwrap().to_string() + "/ns/" + ns_type.as_str(),
            ) {
                Ok(link) => usize_pattern
                    .find(link.to_str().unwrap_or_default())
                    .unwrap()
                    .as_str()
                    .to_owned(),
                Err(_) => continue,
            };
        }
        if namespaces.contains_key(&index) {
            namespaces
                .get_mut(&index)
                .unwrap()
                .procs
                .push(proc_dir.path().to_str().unwrap().to_string() + "/ns/" + ns_type.as_str());
        } else {
            namespaces.insert(
                index,
                Namespace::new(
                    ns_type.clone(),
                    proc_dir.path().to_str().unwrap().to_string() + "/ns/" + ns_type.as_str(),
                ),
            );
        }
    }
    Ok(namespaces)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_namespaces_test() {
        println!("{:?}", get_namespaces().unwrap());
    }
    #[test]
    fn get_specific_namespaces_test() {
        println!("{:?}", get_specific_namespaces(NsType::MNT).unwrap());
    }
}
