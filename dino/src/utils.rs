use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::Result;
use bundler::run_bundle;
use glob::glob;

use crate::BUILD_DIR;
//get all files with certain extension in a directory
pub(crate) fn get_files_with_extension(dir: &str, ext: &[&str]) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    for ex in ext {
        let ts_glob = format!("{}/**/*{}", dir, ex);
        for entry in glob(&ts_glob)? {
            let path = entry?;
            //判断path是否包含.build
            if path.to_str().unwrap().contains(".build/") {
                continue;
            }
            files.insert(path);
        }
    }
    // println!("files: {:?}", files);
    Ok(files)
}
pub(crate) fn calc_project_hash(dir: &str) -> Result<String> {
    let ext = [".ts", ".json", ".js"];
    calc_hash_for_files(dir, &ext, 12)
}

pub(crate) fn calc_hash_for_files(dir: &str, ext: &[&str], len: usize) -> Result<String> {
    let files = get_files_with_extension(dir, ext)?;
    let mut hasher = blake3::Hasher::new();

    for file in files {
        hasher.update_reader(File::open(&file)?)?;
    }
    let mut result = hasher.finalize().to_string();
    result.truncate(len);
    Ok(result)
}

pub(crate) fn build_project(dir: &str) -> Result<String> {
    let hash = calc_project_hash(dir)?;
    let filename = format!("{}/{}.js", BUILD_DIR, hash);
    let dst = Path::new(&filename);
    if dst.exists() {
        eprint!("Build {} already exists", filename);
        return Ok(filename);
    }
    // println!("Building project: {}", filename);
    let content = run_bundle("main.ts", &Default::default())?;
    fs::create_dir_all(BUILD_DIR)?;
    fs::write(dst, content)?;
    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_files_with_extension() {
        let files = get_files_with_extension(
            "fixtures/prj",
            &[".ts", ".json", ".js", ".yml", ".gitignore"],
        )
        .unwrap();
        assert_eq!(
            files.into_iter().collect::<Vec<_>>(),
            [
                PathBuf::from("fixtures/prj/.gitignore"),
                PathBuf::from("fixtures/prj/a.ts"),
                PathBuf::from("fixtures/prj/config.yml"),
                PathBuf::from("fixtures/prj/test1/b.ts"),
                PathBuf::from("fixtures/prj/test1/c.js"),
                PathBuf::from("fixtures/prj/test2/test3/d.json"),
            ]
        );
    }
}
