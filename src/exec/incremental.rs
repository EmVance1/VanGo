use super::BuildInfo;
use crate::fetch::FileInfo;

pub enum BuildLevel<'a> {
    UpToDate,
    LinkOnly,
    CompileAndLink(Vec<(&'a str, String)>),
}

pub fn get_build_level(info: &BuildInfo) -> BuildLevel {
    let pairs: Vec<_> = info
        .sources
        .iter()
        .map(|c| {
            (
                c,
                FileInfo::from_str(&transform_file(
                    &c.repr,
                    &info.srcdir,
                    &info.outdir,
                    info.toolset.is_msvc(),
                )),
            )
        })
        .collect();

    if info.outfile.exists() {
        if !get_recent_changes(&info.headers, info.outfile.modified().unwrap()).is_empty() {
            BuildLevel::CompileAndLink(
                pairs
                    .into_iter()
                    .map(|(src, obj)| (src.repr.as_str(), obj.repr))
                    .collect(),
            )
        } else {
            let mut build = Vec::new();
            for (src, obj) in pairs {
                if !obj.exists()
                    || src.modified().unwrap() > obj.modified().unwrap()
                    || src.modified().unwrap() > info.outfile.modified().unwrap()
                {
                    build.push((src.repr.as_str(), obj.repr))
                }
            }
            if build.is_empty() {
                if !get_recent_changes(&info.relink, info.outfile.modified().unwrap()).is_empty() {
                    BuildLevel::LinkOnly
                } else {
                    BuildLevel::UpToDate
                }
            } else {
                BuildLevel::CompileAndLink(build)
            }
        }
    } else {
        let mut build = Vec::new();
        for (src, obj) in pairs {
            if !obj.exists() || src.modified().unwrap() > obj.modified().unwrap() {
                build.push((src.repr.as_str(), obj.repr))
            }
        }
        if build.is_empty() {
            BuildLevel::LinkOnly
        } else {
            BuildLevel::CompileAndLink(build)
        }
    }
}

fn get_recent_changes(sources: &[FileInfo], pivot: std::time::SystemTime) -> Vec<&FileInfo> {
    sources
        .iter()
        .filter(|src| src.modified().unwrap() > pivot)
        .collect()
}

fn transform_file(path: &str, src_dir: &str, out_dir: &str, msvc: bool) -> String {
    if msvc {
        path.replace(src_dir, out_dir)
            .replace(".cpp", ".obj")
            .replace(".c", ".obj")
    } else {
        path.replace(src_dir, out_dir)
            .replace(".cpp", ".o")
            .replace(".c", ".o")
    }
}
