use super::BuildInfo;
use crate::fetch::FileInfo;


pub enum BuildLevel<'a> {
    UpToDate,
    LinkOnly,
    CompileAndLink(Vec<(&'a str, String)>),
}


pub fn get_build_level(info: &BuildInfo) -> BuildLevel {
    let objdir = format!("{}obj/", info.outdir);

    let pairs: Vec<_> = info
        .sources
        .iter()
        .map(|c| (c, FileInfo::from_str(&transform_file( &c.repr, &info.srcdir, &objdir, info.toolchain.is_msvc()))))
        .collect();

    // IF BINARY EXISTS
    if info.outfile.exists() {
        // IF ANY HEADER IS NEWER THAN THE BINARY
        if !get_recent_changes(&info.headers, info.outfile.modified().unwrap()).is_empty() {
            BuildLevel::CompileAndLink(pairs
                .into_iter()
                .map(|(src, obj)| (src.repr.as_str(), obj.repr))
                .collect())

        // NO HEADER IS NEWER THAN THE BINARY
        } else {
            // IF ANY SOURCE IS NEWER THAN ITS OBJ | (AND THE BINARY BY TRANSITIVITY)
            let build: Vec<_> = pairs.into_iter()
                .filter(|(src, obj)| !obj.exists() || (src.modified().unwrap() > obj.modified().unwrap()))
                .map(   |(src, obj)| (src.repr.as_str(), obj.repr))
                .collect();

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
        // IF ANY SOURCE IS NEWER THAN ITS OBJ
        let build: Vec<_> = pairs.into_iter()
            .filter(|(src, obj)| !obj.exists() || (src.modified().unwrap() > obj.modified().unwrap()))
            .map(   |(src, obj)| (src.repr.as_str(), obj.repr))
            .collect();

        if build.is_empty() {
            BuildLevel::LinkOnly
        } else {
            BuildLevel::CompileAndLink(build)
        }
    }
}


fn get_recent_changes(sources: &[FileInfo], pivot: std::time::SystemTime) -> Vec<&FileInfo> {
    sources.iter().filter(|src| src.modified().unwrap() > pivot).collect()
}

fn transform_file(path: &str, src_dir: &str, obj_dir: &str, msvc: bool) -> String {
    if msvc {
        path.replace(src_dir, obj_dir)
            .replace(".cpp", ".obj")
            .replace(".c", ".obj")
    } else {
        path.replace(src_dir, obj_dir)
            .replace(".cpp", ".o")
            .replace(".c", ".o")
    }
}

