use crate::fetch::FileInfo;
use super::BuildInfo;


pub fn get_outdated(info: &BuildInfo) -> Option<Vec<(&str, String)>> {
    if !info.outfile.exists() || !get_recent_changes(&info.headers, info.outfile.modified().unwrap()).is_empty() {
        Some(info.sources.iter().map(|c| {
            (c.repr.as_str(), c.repr.replace(&info.src_dir, &info.out_dir).replace(".cpp", ".obj"))
        }).collect())
    } else {
        let src_changes = get_recent_changes(&info.sources, info.outfile.modified().unwrap());
        if src_changes.is_empty() { 
            None
        } else {
            Some(src_changes.into_iter().map(|c| {
                (c.repr.as_str(), c.repr.replace(&info.src_dir, &info.out_dir).replace(".cpp", ".obj"))
            }).collect())
        }
    }
}


fn get_recent_changes(sources: &[FileInfo], pivot: std::time::SystemTime) -> Vec<&FileInfo> {
    sources.iter().filter(|src| src.modified().unwrap() > pivot).collect()
}

