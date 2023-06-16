use clap::Parser;
use rebackup::{fail, walk, WalkerConfig};
use std::fs;
use std::io::{self, Read, Write};
/// Performs a simple backup on a specified directory.
///
/// The files it backs up are determined by the following rules.
///
/// * It traverses the directory specified looking for files that shoud be backed up
/// * If a .gitignore file is found then the files and driectories specified in it will not be backed up.
/// * If a .trackup_ignore file is found  then the files and driectories specified in it will not be backed up.
/// This file will take precendence over any .gitignore file.
/// * Files are only backed up up if they are newer then the one in the backup.  
// Used the rebackup crate (https://docs.rs/rebackup/latest/rebackup/).
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The source directory to be backed up
    source: PathBuf,

    /// The target directory where the backup is made. Does not need to be specified if
    /// setup as an environment varaiable (TODO)
    #[arg(short, long)]
    target: Option<String>,
}

fn main() {
    println!("Backing up (TODO)");

    let cli = Args::parse();

    println!("Source: {:?}", cli.source);

    let checked_file = std::path::PathBuf::from("path/to/checked_file.txt");
    let existing_file = std::path::PathBuf::from("path/to/existing_file.txt");

    let copy_action = is_newer(&checked_file, &existing_file);

    if copy_action {
        if let Err(err) = copy_file(&checked_file, &existing_file) {
            eprintln!("Error copying file: {}", err);
        } else {
            println!("File copied successfully.");
        }
    }
}

fn is_newer(checked_file: &PathBuf, existing_file: &std::path::PathBuf) -> bool {
    if let (Ok(checked_metadata), Ok(existing_metadata)) =
        (fs::metadata(checked_file), fs::metadata(existing_file))
    {
        if let (Ok(checked_modified), Ok(existing_modified)) =
            (checked_metadata.modified(), existing_metadata.modified())
        {
            return checked_modified > existing_modified;
        }
    }
    false
}

fn copy_file(
    checked_file: &std::path::PathBuf,
    existing_file: &std::path::PathBuf,
) -> io::Result<()> {
    // Open the checked file for reading
    let mut checked_file_content = Vec::new();
    let mut checked_file = fs::File::open(checked_file)?;
    checked_file.read_to_end(&mut checked_file_content)?;

    // Create or open the existing file for writing
    let mut existing_file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(existing_file)?;

    // Write the contents of the checked file to the existing file
    existing_file.write_all(&checked_file_content)?;

    Ok(())
}
