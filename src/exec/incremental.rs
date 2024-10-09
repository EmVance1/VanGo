use std::path::PathBuf;
use crate::fetch::FileInfo;
use super::BuildInfo;


#[derive(Debug, PartialEq)]
pub enum IncrementalBuild<'a> {
    BuildAll,
    BuildSelective(Vec<(&'a FileInfo, FileInfo)>),
    NoBuild,
}

impl<'a> IncrementalBuild<'a> {
    pub fn calc(info: &'a BuildInfo) -> Self {
        if !info.outfile.exists() { return IncrementalBuild::BuildAll; }
        if !get_recent_changes(&info.headers, info.outfile.modified().unwrap()).is_empty() {
            return IncrementalBuild::BuildAll;
        }

        let src_changes = get_recent_changes(&info.sources, info.outfile.modified().unwrap());
        if src_changes.is_empty() { 
            IncrementalBuild::NoBuild
        } else {
            IncrementalBuild::BuildSelective(src_changes.into_iter().map(|c| {
                (c, FileInfo::from_path(&PathBuf::from(c.repr.replace(&info.sdir, &info.odir).replace(".cpp", ".obj"))))
            }).collect())
        }
    }
}


fn get_recent_changes(sources: &[FileInfo], pivot: std::time::SystemTime) -> Vec<&FileInfo> {
    sources.iter().filter(|src| src.modified().unwrap() > pivot).collect()
}

