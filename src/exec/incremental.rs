use std::path::{Path, PathBuf};
use super::BuildInfo;


pub enum BuildLevel<'a> {
    UpToDate,
    LinkOnly,
    CompileAndLink(Vec<(&'a Path, PathBuf)>),
}


pub fn get_build_level(info: &BuildInfo) -> BuildLevel {
    if info.outfile.exists() {
        let pivot = info.outfile.metadata().unwrap().modified().unwrap();

        // FULL REBUILD IF ANY HEADER IS NEWER THAN THE BINARY
        if any_changed(&info.headers, pivot) {
            BuildLevel::CompileAndLink(info.sources.iter()
                .map(|src| (src.as_path(), transform_file(src, &info.outdir, info.toolchain.is_msvc())))
                .collect())

        // NO HEADER IS NEWER THAN THE BINARY
        } else {
            // RECOMPILE ANY SOURCE THAT IS NEWER THAN THE BINARY
            let pairs: Vec<_> = info.sources.iter()
                .filter_map(|src| {
                    if src.metadata().unwrap().modified().unwrap() > pivot {
                        Some((src.as_path(), transform_file(&src, &info.outdir, info.toolchain.is_msvc())))
                    } else {
                        None
                    }
                })
                .collect();

            if pairs.is_empty() {
                if any_changed(&info.relink, pivot) {
                    BuildLevel::LinkOnly
                } else {
                    BuildLevel::UpToDate
                }
            } else {
                BuildLevel::CompileAndLink(pairs)
            }
        }
    } else {
        // RECOMPILE ANY SOURCE THAT IS NEWER THAN ITS OBJECT
        let pairs: Vec<_> = info.sources.iter()
            .filter_map(|src| {
                let obj = transform_file(&src, &info.outdir, info.toolchain.is_msvc());
                if !obj.exists() || (src.metadata().unwrap().modified().unwrap() > obj.metadata().unwrap().modified().unwrap()) {
                    Some((src.as_path(), obj))
                } else {
                    None
                }
            })
            .collect();

        if pairs.is_empty() {
            BuildLevel::LinkOnly
        } else {
            BuildLevel::CompileAndLink(pairs)
        }
    }
}


fn any_changed(sources: &[PathBuf], pivot: std::time::SystemTime) -> bool {
    sources.iter().any(|src| src.metadata().unwrap().modified().unwrap() > pivot)
}

fn transform_file(path: &Path, odir: &Path, msvc: bool) -> PathBuf {
    odir.join("obj").join(path.strip_prefix("src").unwrap()).with_extension(if msvc { "obj" } else { "o" })
}

