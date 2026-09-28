use crate::{
    error::Error,
    log_info_ln, log_warn_ln,
};
use std::{path::{Path, PathBuf}, collections::{HashSet, HashMap}};
use super::{BuildInfo, fsutil};


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum UseType<'a> {
    #[default]
    None,
    Create(&'a Path),
    Use(&'a Path),
}

pub fn precompile_headers(info: &BuildInfo, verbose: bool, echo: bool) -> Result<HashMap<PathBuf, PathBuf>, Error> {
    let mut use_pch = HashMap::new();
    for pch in &info.pch {
        let _ = std::fs::create_dir(info.outdir.join("pch"));
        let inpch = info.srcdir.join(&pch.header); // path/to/header
        let incpp = info.outdir.join(format!("pch/pch_impl.{}", info.lang.src_ext())); // including cpp file (MSVC style)
        let infile = if info.toolchain.is_msvc_compatible() {
            &incpp
        } else {
            &inpch
        };
        let outfile = if info.toolchain.is_msvc_compatible() {
            // output file
            let _ = std::fs::write(&incpp, format!("#include \"{}\"", pch.header.display()));
            info.outdir.join("obj").join(&pch.header).with_extension("h.obj") // MSVC internally reates a .obj and .pch
        } else {
            info.outdir.join("pch").join(&pch.header).with_extension("h.gch") // GNU .gch
        };

        // if PCH requires rebuild
        if info.changed
            || !std::fs::exists(&outfile)?
            || (std::fs::metadata(&inpch)?.modified()? > std::fs::metadata(&outfile)?.modified()?)
        {
            log_info_ln!("precompiling header: {}", inpch.display());
            let output = info.toolchain.compiler().command(&infile, &outfile, &info, UseType::Create(&pch.header), verbose, echo)
                .spawn()
                .map_err(|_| Error::CompilerNotFound(info.toolchain))?
                .wait_with_output()
                .unwrap();
            if !info.toolchain.compiler().output(&output) {
                return Err(Error::CompilerFail(info.outfile.clone()));
            }
        }

        let used_by: HashSet<_> = if pch.used_by.is_empty() {
            fsutil::scan_for_filetype(Path::new("src"), &[ info.lang.src_ext().to_string() ])?.into_iter().collect()
        } else {
            let mut total = Vec::new();
            for used in &pch.used_by {
                if used.is_dir() {
                    total.extend(fsutil::scan_for_filetype(&used, &[ info.lang.src_ext().to_string() ])?);
                } else {
                    total.push(used.clone());
                }
            }
            total.into_iter().collect()
        };
        let ignored_by: HashSet<_> = {
            let mut total = Vec::new();
            for ignored in &pch.ignored_by {
                if ignored.is_dir() {
                    total.extend(fsutil::scan_for_filetype(&ignored, &[ info.lang.src_ext().to_string() ])?);
                } else {
                    total.push(ignored.clone());
                }
            }
            total.into_iter().collect()
        };
        for entry in used_by.difference(&ignored_by) {
            if let Some(_) = use_pch.insert(entry.to_owned(), pch.header.clone()) {
                log_warn_ln!("source '{}' included in multiple PCH sets - not disjoint", entry.display());
            }
        }
    }
    Ok(use_pch)
}
