# rackup

## Usage
Performs a simple backup on a specified directory.

The files it backs up are determined by the following rules:

* It recursively traverses the directory specified looking for files that should be backed up.
* If a `.gitignore` file is found then the files and driectories specified to be ignored  will not be backed up.
* `.exe` files will not be backed up.
* Files are only backed up if they are newer then the ones in the backup.

## Project Status
* It is very slow. Perhaps it can be speeded up by:
  - Writing it in an asynchronise style.
  - The ignoring of files in the `.gitignore` file is currently performed by starting a process and
running `git check-ignore`. Parsing the `.gitignore` file directly (using, for instance,
the crate [ignore](https://docs.rs/ignore/latest/ignore/)) could be quicker.
* If a `.rackup_ignore` file is found then the files and directories specified in it will not be backed up.
* Have the backup directory specified by an environment variable.

