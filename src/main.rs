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
use std::path::PathBuf;

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

    let source_file_name = cli.source;

    let source_path = PathBuf::from(source_file_name);
    //let backup_path =         PathBuf::from("/c/Users/T440s/Documents/rust-projects/rackup/test_data/backup");

    let backup_path = PathBuf::from(cli.backup);

    // if let Ok(source_file_name) = source_file.into_os_string().into_string() {
    //     println!("backing up {}", source_file_name);
    // } else {
    //     // TODO error handling
    //     panic!();
    // }

    let copy_action = is_newer(&source_path, &backup_path);

    if copy_action {
        if let Err(err) = copy_file(&source_path, &backup_path) {
            eprintln!("Error copying file: {}", err);
        } else {
            println!("File copied successfully.");
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
}
