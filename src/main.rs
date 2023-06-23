use anyhow::Result;
/// Performs a simple backup on a specified directory.
///
/// The files it backs up are determined by the following rules.
///
/// * It traverses the directory specified looking for files that shoud be backed up
/// * If a .gitignore file is found then the files and driectories specified in it will not be backed up.
/// * If a .trackup_ignore file is found  then the files and driectories specified in it will not be backed up.
/// This file will take precendence over any .gitignore file.
/// * Files are only backed up up if they are newer then the one in the backup.  
// Use the rebackup crate (https://docs.rs/rebackup/latest/rebackup/).
use clap::Parser;
use rebackup::{walk, WalkerConfig};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf, Prefix};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The source directory to be backed up
    source: PathBuf,

    // The backup file
    backup: PathBuf,

    /// The target directory where the backup is made. Does not need to be specified if
    /// setup as an environment varaiable (TODO)
    #[arg(short, long)]
    target: Option<String>,
}

fn main() {
    println!("Backing up (TODO)");

    let cli = Args::parse();

    //let source_dir_path = PathBuf::from(source_dir_name);
    let source_dir_path = cli.source;
    //let backup_path =         PathBuf::from("/c/Users/T440s/Documents/rust-projects/rackup/test_data/backup");

    let backup_dir_path = cli.backup; // TODO as environment variable

    perform_backup(source_dir_path, backup_dir_path);
}

fn perform_backup(source_dir_path: PathBuf, backup_dir_path: PathBuf) {
    // NOTE: This can be shortened to `WalkerConfig::new(vec![])`
    //       (expanded here for explanations purpose)
    let config = WalkerConfig {
        rules: vec![],
        follow_symlinks: false,
        drop_empty_dirs: false,
    };

    let source_files_list =
        walk(&source_dir_path, &config).expect("Failed to build the files list");

    // let mut components = source_files_list[0].components();
    // for c in components {
    //     println!("component:{:?}", c);
    // }

    for source_file_path in source_files_list {
        let backup_file_path = create_backup_file_path(&source_file_path, &backup_dir_path);

        if is_newer(&source_file_path, &backup_file_path) {
            if let Err(err) = copy_file(&source_file_path, &backup_file_path) {
                eprintln!(
                    "Error copying {}: {}",
                    source_file_path.to_string_lossy(),
                    err
                );
            } else {
                println!(
                    "File {} copied successfully.",
                    source_file_path.to_string_lossy()
                );
            }
        }
    }
}

/// Checks if the `source_file`is newer then the `backup_file`.
/// If the `backup_file`does not exist then this always returns `true`.
///
fn is_newer(source_file: &PathBuf, backup_file: &std::path::PathBuf) -> bool {
    // Check if the backup file exists. If it does not return true as the source file
    // is "newer"
    if !backup_file.exists() || !backup_file.is_file() {
        return true;
    }

    // Check the modifed times of the files to find the newest
    if let (Ok(source_metadata), Ok(backup_metadata)) =
        (fs::metadata(source_file), fs::metadata(backup_file))
    {
        if let (Ok(source_modified), Ok(existing_modified)) =
            (source_metadata.modified(), backup_metadata.modified())
        {
            return source_modified > existing_modified;
        }
    }
    false
}

/// Copies over the backup file.
fn copy_file(source_file_path: &PathBuf, backup_file_path: &PathBuf) -> io::Result<()> {
    // Create the directory/directories the file is in if they have not already been created.
    let mut dir = backup_file_path.clone();
    dir.pop();
    fs::create_dir_all(dir)?;

    // Open the source file for reading, but only if it is a file
    // (directories hve been created before).
    if source_file_path.is_file() {
        let mut source_file_content = Vec::new();
        let mut source_file = fs::File::open(source_file_path)?;
        source_file.read_to_end(&mut source_file_content)?;

        // Create or open the existing file for writing
        let mut backup_file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(backup_file_path)?;

        // Write the contents of the checked file to the existing file
        backup_file.write_all(&source_file_content)?;
    } else {
        // Is just a directory so create it
        fs::create_dir_all(backup_file_path)?;
    }

    Ok(())
}

// Create the path of the file being backed up, i.e.:
// with source file: C:/Users/bob/Documents/test.txt
// and backup directory C:/Users/bob/Backup it will create a PathBuf of
//      C:/Users/bob/Backup/C/Users/bob/Documents/test.txt

fn create_backup_file_path(source_file_path: &Path, backup_dir_path: &Path) -> PathBuf {
    let components = source_file_path.components();

    let mut backup_file_path = PathBuf::from(backup_dir_path);

    let mut sub_path = String::new();

    for component in components {
        match component {
            Component::Prefix(p) => match p.kind() {
                Prefix::Verbatim(_osstr) | Prefix::DeviceNS(_osstr) => {
                    //sub_path.push_str(osstr.to_str().unwrap_or("?"))
                    sub_path.push_str(""); // Ignored
                }
                Prefix::VerbatimUNC(hostname, sharename) | Prefix::UNC(hostname, sharename) => {
                    sub_path.push_str(hostname.to_str().unwrap_or("?"));
                    sub_path.push('/');
                    sub_path.push_str(sharename.to_str().unwrap_or("?"));
                }
                Prefix::Disk(disk_chr) | Prefix::VerbatimDisk(disk_chr) => {
                    sub_path.push(disk_chr as char);
                }
            },
            Component::RootDir => sub_path.push('/'),
            Component::Normal(c) => {
                sub_path.push_str(c.to_str().unwrap());
                sub_path.push('/');
            }
            _ => sub_path.push_str("unknown"),
        };
    }

    // Remove the trailing "/"
    sub_path.pop();

    backup_file_path.push(sub_path);

    backup_file_path
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::time;
    use std::fs::File;
    use std::io::{self, Write};
    //use tempfile::tempdir;

    //use crate::is_newer;

    #[test]
    fn test_is_newer_where_backup_file_does_not_exist() -> Result<(), std::io::Error> {
        // Set up test data
        let test_dir = tempfile::tempdir()?;

        let source_path = test_dir.path().join("source_test_data");
        let mut source_file = File::create(source_path)?;
        writeln!(source_file, "Some test data")?;

        // test the is_newer function
        let backup_path = test_dir.path().join("backup");
        let source_path_created = test_dir.path().join("source_test_data");
        let newer = is_newer(&source_path_created, &backup_path);
        assert!(newer);

        //Cleanup
        drop(source_file);
        test_dir.close()?;

        Ok(())
    }

    #[test]
    fn test_is_newer_where_source_older() -> Result<(), std::io::Error> {
        // Set up test data
        let test_dir = tempfile::tempdir()?;

        // Create backup file after source file
        let source_path = test_dir.path().join("source_test_data");
        let mut source_file = File::create(source_path)?;
        writeln!(source_file, "Some test data")?;

        let backup_path = test_dir.path().join("backup");
        let mut backup_file = File::create(backup_path)?;
        writeln!(backup_file, "Some test data")?;

        // Test the is_newer function
        let source_path_created = test_dir.path().join("source_test_data");
        let backup_path_created = test_dir.path().join("backup");
        let newer = is_newer(&source_path_created, &backup_path_created);
        assert!(!newer); // Backup file is younger than source file

        //Cleanup
        drop(source_file);
        drop(backup_file);
        test_dir.close()?;

        Ok(())
    }
    #[test]
    fn test_is_newer_where_source_younger() -> Result<(), std::io::Error> {
        // Set up test data
        let test_dir = tempfile::tempdir()?;

        // Crate backup file before source file
        let backup_path = test_dir.path().join("backup");
        let mut backup_file = File::create(backup_path)?;
        writeln!(backup_file, "Some test data")?;

        std::thread::sleep(time::Duration::from_millis(250));

        let source_path = test_dir.path().join("source_test_data");
        let mut source_file = File::create(source_path)?;
        writeln!(source_file, "Some test data")?;

        // Test the is_newer function
        let source_path_created = test_dir.path().join("source_test_data");
        let backup_path_created = test_dir.path().join("backup");
        let newer = is_newer(&source_path_created, &backup_path_created);
        assert!(newer); // Backup file is older then source file

        //Cleanup
        drop(source_file);
        drop(backup_file);
        test_dir.close()?;

        Ok(())
    }

    #[test]
    fn test_first_backup() -> Result<(), std::io::Error> {
        // Set up test data
        let test_content = "Some test content".to_string();

        let test_dir = tempfile::tempdir()?;

        let source_path = test_dir.path().join("source_test_data");
        let mut source_file = File::create(source_path)?;
        write!(source_file, "{}", test_content)?;

        // Test the back_file function
        let backup_path = test_dir.path().join("backup");
        let source_path_created = test_dir.path().join("source_test_data");
        copy_file(&source_path_created, &backup_path)?;

        let mut created_backup_file = File::open(backup_path)?;
        let mut buf = String::new();
        created_backup_file.read_to_string(&mut buf)?;
        assert_eq!(buf, test_content);

        //Cleanup
        drop(source_file);
        test_dir.close()?;

        Ok(())
    }
    #[test]
    fn test_subsequent_backup() -> Result<(), std::io::Error> {
        // Set up test data

        let test_dir = tempfile::tempdir()?;

        let backup_path = test_dir.path().join("backup");
        let mut backup_file = File::create(backup_path)?;
        write!(
            backup_file,
            "Some really old data that should be overwritten."
        )?;
        assert!(test_dir.path().join("backup").exists());

        let test_content = "Some test content".to_string();
        let source_path = test_dir.path().join("source_test_data");
        let mut source_file = File::create(source_path)?;
        write!(source_file, "{}", test_content)?;

        // Test the copy_file function
        let backup_path = test_dir.path().join("backup");
        let source_path_created = test_dir.path().join("source_test_data");
        copy_file(&source_path_created, &backup_path)?;

        let mut created_backup_file = File::open(backup_path)?;
        let mut buf = String::new();
        created_backup_file.read_to_string(&mut buf)?;
        assert_eq!(buf, test_content);

        //Cleanup
        drop(source_file);
        test_dir.close()?;

        Ok(())
    }

    #[test]
    fn test_create_backup_path() {
        // With source file: C:/Users/bob/Documents/test.txt
        // and backup directory C:/Users/bob/Backup it will create a path of
        //  C:/Users/bob/Backup/c/Users/bob/Documents/test.txt

        let mut source_file_path = PathBuf::from("C:/Users/bob/Documents/test.txt");
        let mut backup_dir_path = PathBuf::from("C:/Users/bob/Backup");

        let mut backup_path = create_backup_file_path(&source_file_path, &backup_dir_path);

        assert_eq!(
            PathBuf::from("C:/Users/bob/Backup/C/Users/bob/Documents/test.txt"),
            backup_path
        );

        // With other drives
        // With source file: D:/Users/bob/Documents/test.txt
        // and backup directory G:/Users/bob/Backup it will create a path of
        //  G:/Users/bob/Backup/D/Users/bob/Documents/test.txt
        source_file_path = PathBuf::from("D:/Users/bob/Documents/test.txt");
        backup_dir_path = PathBuf::from("G:/Users/bob/Backup");

        backup_path = create_backup_file_path(&source_file_path, &backup_dir_path);

        assert_eq!(
            PathBuf::from("G:/Users/bob/Backup/D/Users/bob/Documents/test.txt"),
            backup_path
        );
    }

    #[test]
    fn test_perform_new_backup() -> Result<(), std::io::Error> {
        // Create a temporary directory/file stucture to back up. Each file contains a with the name of the file.
        // Structure is:
        // TempDir
        //   -> TestUser
        //       -> DocumentsA
        //          --> fileAA.txt
        //          --> fileBA.txt
        //       --> DocumentsB
        //           --> fileBA.pdf
        //           --> fileBB.doc
        //           --> fileBC.txt
        //       --> DocumentsC
        //           --> (empty)
        //
        let test_dir = setup_file_structure()?;

        // Test perform_backup()
        perform_backup(
            test_dir.path().join("TestUser"),
            test_dir.path().join("Backup"),
        );

        // Check if the files and directories have been created.
        // First get the path of the temp directory.
        let tail = test_dir.path().to_str().unwrap().to_string();
        // Assuming that the temp dir used for test in the C: drive. For the backup path remove
        // the C: and replace it with C
        let tail_norm = tail.replace(":", "");
        // Get the full backup path, i.e.
        // <temp test dir>/Backup/<temp test dir with C: changed to C>
        let full_backup_path = test_dir.path().join("Backup").join(tail_norm);

        assert!(test_dir.path().join("Backup").exists());
        assert!(full_backup_path.join("TestUser/DocumentsA").exists());
        assert!(full_backup_path.join("TestUser/DocumentsB").exists());
        assert!(full_backup_path.join("TestUser/DocumentsC").exists());

        assert!(full_backup_path
            .join("TestUser/DocumentsA/fileAA.txt")
            .exists());
        assert!(full_backup_path
            .join("TestUser/DocumentsA/fileAB.txt")
            .exists());

        assert!(full_backup_path
            .join("TestUser/DocumentsB/fileBA.pdf")
            .exists());
        assert!(full_backup_path
            .join("TestUser/DocumentsB/fileBB.doc")
            .exists());
        assert!(full_backup_path
            .join("TestUser/DocumentsB/fileBC.txt")
            .exists());

        assert!(full_backup_path.join("TestUser/DocumentsC").exists());

        // Sample if the files contain the data
        let p = full_backup_path.join("TestUser/DocumentsA/fileAA.txt");
        let mut contents = String::new();
        let mut file = fs::File::open(p)?;
        file.read_to_string(&mut contents)?;
        assert_eq!(contents, "fileAA.txt");

        let p = full_backup_path.join("TestUser/DocumentsB/fileBB.doc");
        let mut file = fs::File::open(p)?;
        contents.clear();
        file.read_to_string(&mut contents)?;
        assert_eq!(contents, "fileBB.doc");

        Ok(())
    }

    // Test utilities

    // Create a temporary directory/file stucture to back up. Each file contains a with the name of the file.
    // Structure is:
    // TempDir
    //   -> TestUser
    //       -> DocumentsA
    //          --> fileAA.txt
    //          --> fileBA.txt
    //       --> DocumentsB
    //           --> fileBA.pdf
    //           --> fileBB.doc
    //           --> fileBC.txt
    //       --> DocumentsC
    //           --> (empty)
    //
    fn setup_file_structure() -> Result<tempfile::TempDir, io::Error> {
        let test_dir = tempfile::tempdir()?;
        fs::create_dir(test_dir.path().join("TestUser"))?;
        fs::create_dir(test_dir.path().join("TestUser/DocumentsA"))?;
        fs::create_dir(test_dir.path().join("TestUser/DocumentsB"))?;
        fs::create_dir(test_dir.path().join("TestUser/DocumentsC"))?;
        let mut f = File::create(test_dir.path().join("TestUser/DocumentsA/fileAA.txt"))?;
        write!(f, "fileAA.txt")?;
        f = File::create(test_dir.path().join("TestUser/DocumentsA/fileAB.txt"))?;
        write!(f, "fileAB.txt")?;
        f = File::create(test_dir.path().join("TestUser/DocumentsB/fileBA.pdf"))?;
        write!(f, "fileBA.pdf")?;
        f = File::create(test_dir.path().join("TestUser/DocumentsB/fileBB.doc"))?;
        write!(f, "fileBB.doc")?;
        f = File::create(test_dir.path().join("TestUser/DocumentsB/fileBC.txt"))?;
        write!(f, "fileBC.txt")?;
        assert!(test_dir
            .path()
            .join("TestUser/DocumentsA/fileAB.txt")
            .exists());
        assert!(test_dir.path().join("TestUser/DocumentsC").exists());
        Ok(test_dir)
    }
}
