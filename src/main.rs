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
use rebackup::{fail, walk, WalkerConfig};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Components, PathBuf, Prefix};

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

    // NOTE: This can be shortened to `WalkerConfig::new(vec![])`
    //       (expanded here for explanations purpose)
    let config = WalkerConfig {
        rules: vec![],
        follow_symlinks: false,
        drop_empty_dirs: false,
    };

    let source_files_list =
        walk(&source_dir_path, &config).expect("Failed to build the files list");

    let mut components = source_files_list[0].components();
    for c in components {
        println!("component:{:?}", c);
    }

    for source_file_path in source_files_list {
        let backup_file_path = create_backup_file_path(&source_file_path, &backup_dir_path);
        if is_newer(&source_file_path, &backup_file_path) {
            if let Err(err) = copy_file(&source_dir_path, &backup_file_path) {
                eprintln!("Error copying file: {}", err);
            } else {
                println!("File copied successfully.");
            }
        }
    }
}

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

fn copy_file(source_file_path: &PathBuf, backup_file_path: &PathBuf) -> io::Result<()> {
    // Open the source file for reading
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

    Ok(())
}

// Create the path of the file being backed up, i.e.:
// With source file: C:/Users/bob/Documents/test.txt
// and backup directory C:/Users/bob/Backup it will create a PathBuf of
//      C:/Users/bob/Backup/c/Users/bob/Documents/test.txt

fn create_backup_file_path(source_file_path: &PathBuf, backup_dir_path: &PathBuf) -> PathBuf {
    let components = source_file_path.components();

    let mut backup_file_path = PathBuf::from(backup_dir_path);

    let mut sub_path = String::new();

    for component in components {
        match component {
            Component::Prefix(p) => match p.kind() {
                Prefix::Verbatim(osstr) | Prefix::DeviceNS(osstr) => {
                    sub_path.push_str(osstr.to_str().unwrap_or("?"))
                }
                Prefix::VerbatimUNC(hostname, sharename) | Prefix::UNC(hostname, sharename) => {
                    sub_path.push_str(hostname.to_str().unwrap_or("?"));
                    sub_path.push_str("/");
                    sub_path.push_str(sharename.to_str().unwrap_or("?"));
                }
                Prefix::Disk(disk_chr) | Prefix::VerbatimDisk(disk_chr) => {
                    sub_path.push(disk_chr as char);
                }
            },
            Component::RootDir => sub_path.push_str("/"),
            Component::Normal(c) => {
                sub_path.push_str(c.to_str().unwrap());
                sub_path.push_str("/");
            }
            _ => sub_path.push_str("unknown"),
        };
    }

    // Remove the trailing "/"
    sub_path.pop();

    backup_file_path.push(sub_path);

    println!("{backup_file_path:?}");

    backup_file_path
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
