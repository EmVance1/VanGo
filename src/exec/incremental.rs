use crate::fetch::FileInfo;
use super::BuildInfo;




pub fn get_outdated(info: &BuildInfo) -> Option<Vec<(&str, String)>> {
    if !info.outfile.exists() || !get_recent_changes(&info.headers, info.outfile.modified().unwrap()).is_empty() {
        Some(info.sources.iter().map(|c| {
            (c.repr.as_str(), transform_file(&c.repr, &info.src_dir, &info.out_dir))
        }).collect())
    } else {
        let src_changes = get_recent_changes(&info.sources, info.outfile.modified().unwrap());
        if src_changes.is_empty() { 
            None
        } else {
            Some(src_changes.into_iter().map(|c| {
                (c.repr.as_str(), transform_file(&c.repr, &info.src_dir, &info.out_dir))
            }).collect())
        }
    }
}


fn get_recent_changes(sources: &[FileInfo], pivot: std::time::SystemTime) -> Vec<&FileInfo> {
    sources.iter().filter(|src| src.modified().unwrap() > pivot).collect()
}

fn transform_file(path: &str, src_dir: &str, out_dir: &str) -> String {
    path.replace(src_dir, out_dir).replace(".cpp", ".obj")
}

