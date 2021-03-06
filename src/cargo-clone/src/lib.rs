// Copyright 2015 Jan Likar.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate cargo;
extern crate walkdir;

macro_rules! bail {
    ($($fmt:tt)*) => (
        return Err(human(&format_args!($($fmt)*)))
    )
}

pub mod ops {
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::env;

    use cargo::util::{CargoResult, Config, human};
    use cargo::util::to_semver::ToSemver;
    use cargo::core::package_id::PackageId;
    use cargo::core::source::{Source, SourceId};
    use cargo::core::registry::Registry;
    use cargo::core::dependency::Dependency;
    use cargo::sources::RegistrySource;

    use walkdir::WalkDir;

    pub fn clone(krate: Option<&str>,
                 srcid: &SourceId,
                 prefix: Option<&str>,
                 vers: Option<&str>,
                 config: Config)
                 -> CargoResult<()> {

        let krate = match krate {
                Some(ref k) => k,
                None => bail!("specify which package to clone!"),
        };

        let mut src = if srcid.is_registry() {
            RegistrySource::new(&srcid, &config)
        }
        else if srcid.is_path(){
            //PathSource::new(PATH , &srcid, &config)
            unimplemented!();
        }
        else {
            //GitSource::new(&srcid, &config)
            unimplemented!();
        };

        try!(src.update());

        let vers = match vers {
            Some(v) => {
                match v.to_semver() {
                    Ok(v) => v,
                    Err(e) => bail!("{}", e),
                }
            },
            None => {
                let dep = try!(Dependency::parse(krate, vers.as_ref().map(|s| &s[..]), &srcid));
                let summaries = try!(src.query(&dep));

                let latest = summaries.iter().max_by_key(|s| s.version());

                match latest {
                    Some(l) => l.version().to_semver().unwrap(),
                    None => bail!("package '{}' not found", krate),
                }
            },
        };

        let pkgid = try!(PackageId::new(&krate, vers, srcid));
        let krate = try!(src.download(&pkgid.clone()));

        // If prefix was not supplied, clone into current dir
        let mut dest_path = match prefix {
            Some(path) => PathBuf::from(path),
            None => try!(env::current_dir())
        };

        dest_path.push(krate.name());

        try!(clone_directory(&krate.root(), &dest_path));

        Ok(())
    }

    fn clone_directory(from: &Path, to: &Path) -> CargoResult<()> {
        for entry in WalkDir::new(from) {
            let entry = entry.unwrap();
            let file_type = entry.file_type();
            let mut to = to.to_owned();
            to.push(entry.path().strip_prefix(from).unwrap());

            if file_type.is_file() && entry.file_name() != ".cargo-ok" {
                // .cargo-ok is not wanted in this context
                try!(fs::copy(&entry.path(), &to));
            }
            else if file_type.is_dir() {
                try!(fs::create_dir(&to));
            }
        }

        Ok(())
    }
}
