use std::path::{Path, PathBuf};
use super::BuildInfo;


pub enum BuildLevel<'a> {
    UpToDate,
    LinkOnly,
    CompileAndLink(Vec<(&'a Path, PathBuf)>),
}


pub fn get_build_level<'a>(info: &'a BuildInfo) -> BuildLevel<'a> {
    let objdir = info.outdir.join("obj");

    let pairs: Vec<_> = info
        .sources
        .iter()
        .map(|src| (src.as_path(), transform_file(src, &info.srcdir, &objdir, info.toolchain.is_msvc())))
        .collect();

    // IF BINARY EXISTS
    if info.outfile.exists() {
        // IF ANY HEADER IS NEWER THAN THE BINARY
        if any_changed(&info.headers, info.outfile.metadata().unwrap().modified().unwrap()) {
            BuildLevel::CompileAndLink(pairs
                .into_iter()
                .collect())

        // NO HEADER IS NEWER THAN THE BINARY
        } else {
            // IF ANY SOURCE IS NEWER THAN ITS OBJ | (AND THE BINARY BY TRANSITIVITY)
            let build: Vec<_> = pairs.into_iter()
                .filter(|(src, obj)| !obj.exists() || (src.metadata().unwrap().modified().unwrap() > obj.metadata().unwrap().modified().unwrap()))
                .collect();

            if build.is_empty() {
                if any_changed(&info.relink, info.outfile.metadata().unwrap().modified().unwrap()) {
                    BuildLevel::LinkOnly
                } else {
                    BuildLevel::UpToDate
                }
            } else {
                BuildLevel::CompileAndLink(build)
            }
        }
    } else {
        // IF ANY SOURCE IS NEWER THAN ITS OBJ
        let build: Vec<_> = pairs.into_iter()
            .filter(|(src, obj)| !obj.exists() || (src.metadata().unwrap().modified().unwrap() > obj.metadata().unwrap().modified().unwrap()))
            .collect();

        if build.is_empty() {
            BuildLevel::LinkOnly
        } else {
            BuildLevel::CompileAndLink(build)
        }
    }
}


fn any_changed(sources: &[PathBuf], pivot: std::time::SystemTime) -> bool {
    sources.iter().any(|src| src.metadata().unwrap().modified().unwrap() > pivot)
}

fn transform_file(path: &Path, src_dir: &Path, obj_dir: &Path, msvc: bool) -> PathBuf {
    obj_dir.join(path.strip_prefix(src_dir).unwrap()).with_extension(if msvc { "obj" } else { "o" })
}

