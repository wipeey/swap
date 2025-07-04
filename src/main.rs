use std::fmt;
use std::fs;
use std::io;
use std::error::Error;
use std::path::{Path, PathBuf};
use clap::Parser;

// --- Custom Error Type for Clear and Specific Error Handling ---

/// Represents all possible errors that can occur in the application.
/// Using a custom error enum makes error handling explicit and robust.
#[derive(Debug)]
enum SwapError {
    /// An I/O error occurred, wrapping the standard `std::io::Error`.
    /// We also store the path that caused the error for better context.
    Io(std::io::Error, PathBuf),
    /// The specified path does not exist on the filesystem.
    PathNotFound(PathBuf),
    /// The user tried to swap a path with itself.
    SamePath,
    /// A critical safety check failed: attempting to swap a directory with one of its own children.
    /// This would lead to data loss or an invalid filesystem state.
    SwapIntoSubdirectory,
    /// Failed to get the parent directory of a path. Should not happen with canonicalized paths.
    MissingParent(PathBuf),
}

// Implement the Display trait to show user-friendly error messages.
impl fmt::Display for SwapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwapError::Io(err, path) => {
                write!(f, "I/O error for path '{}': {}", path.display(), err)
            }
            SwapError::PathNotFound(path) => {
                write!(f, "Error: Path not found: '{}'", path.display())
            }
            SwapError::SamePath => {
                write!(f, "Error: The two paths are identical. Nothing to swap.")
            }
            SwapError::SwapIntoSubdirectory => {
                write!(f, "Error: Cannot swap a directory with its own subdirectory. This is a safety prevention.")
            }
            SwapError::MissingParent(path) => {
                write!(f, "Error: Could not determine the parent directory of '{}'.", path.display())
            }
        }
    }
}

// Implement the Error trait to be compatible with Rust's error handling mechanisms.
impl Error for SwapError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SwapError::Io(err, _) => Some(err),
            _ => None,
        }
    }
}


// --- Command-Line Argument Parsing using `clap` ---

/// A robust CLI tool to swap two files or directories on Linux.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The first path to swap.
    #[arg(required = true)]
    path1: PathBuf,

    /// The second path to swap.
    #[arg(required = true)]
    path2: PathBuf,

    /// Swap names instead of locations.
    /// If this flag is present, items will be renamed to each other but stay in their original directories.
    /// By default, items are moved to each other's directories, keeping their original names.
    #[arg(short = 'n', long = "name-swap")]
    name_swap: bool,

	/// Add verbose to log advanced informations in the console.
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,
}

/// Macro rule to handle proper logging in case the verbose argument was passed.
macro_rules! log {
    ($cli:expr, $($arg:tt)*) => {
        if $cli.verbose {
            println!($($arg)*);
        }
    };
}

// --- Main Application Logic ---

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    println!("Swap successful!");
}

/// The core function that executes the swapping logic.
fn run(cli: &Cli) -> Result<(), SwapError> {
    // --- 1. Input Validation and Path Canonicalization ---
    
    // Helper closure to map IO errors correctly. This resolves the warning.
    let map_canonicalize_error = |e: io::Error, path: &PathBuf| {
        if e.kind() == io::ErrorKind::NotFound {
            SwapError::PathNotFound(path.clone())
        } else {
            SwapError::Io(e, path.clone())
        }
    };
    
    // `canonicalize` resolves symlinks, `..`, `.` and returns an absolute path.
    // We now check specifically for `NotFound` errors.
    let path1 = fs::canonicalize(&cli.path1)
        .map_err(|e| map_canonicalize_error(e, &cli.path1))?;
    let path2 = fs::canonicalize(&cli.path2)
        .map_err(|e| map_canonicalize_error(e, &cli.path2))?;

    // Check if the user is trying to swap a path with itself.
    if path1 == path2 {
        return Err(SwapError::SamePath);
    }

    // A critical safety check: prevent swapping a directory with its own child.
    if path1.is_dir() && path2.starts_with(&path1) {
        return Err(SwapError::SwapIntoSubdirectory);
    }
    if path2.is_dir() && path1.starts_with(&path2) {
        return Err(SwapError::SwapIntoSubdirectory);
    }

    // --- 2. Dispatch to the Correct Swap Logic ---

	log!(cli, "Swapping '{}' and '{}'...", path1.display(), path2.display());

	if cli.name_swap {
	    log!(cli, "Mode: Swapping names.");
	    swap_names(&path1, &path2, cli)
	} else {
	    log!(cli, "Mode: Swapping locations.");
	    swap_locations(&path1, &path2, cli)
	}
}

/// Swaps the locations of two paths.
fn swap_locations(path1: &Path, path2: &Path, cli: &Cli) -> Result<(), SwapError> {
    let parent1 = path1.parent().ok_or_else(|| SwapError::MissingParent(path1.to_path_buf()))?;
    let parent2 = path2.parent().ok_or_else(|| SwapError::MissingParent(path2.to_path_buf()))?;

    let name1 = path1.file_name().unwrap();
    let name2 = path2.file_name().unwrap();

    let final_dest1 = parent2.join(name1);
    let final_dest2 = parent1.join(name2);
    
    let temp_path = generate_temporary_path(path1)?;

    log!(cli, " 1. Moving '{}' -> '{}' (temporary)", path1.display(), temp_path.display());
    safe_rename(path1, &temp_path)?;
    
    log!(cli, " 2. Moving '{}' -> '{}'", path2.display(), final_dest2.display());
    safe_rename(path2, &final_dest2)?;

    log!(cli, " 3. Moving '{}' (temporary) -> '{}'", temp_path.display(), final_dest1.display());
    safe_rename(&temp_path, &final_dest1)?;

    Ok(())
}

/// Swaps the names of two paths.
fn swap_names(path1: &Path, path2: &Path, cli: &Cli) -> Result<(), SwapError> {
    let parent1 = path1.parent().ok_or_else(|| SwapError::MissingParent(path1.to_path_buf()))?;
    let parent2 = path2.parent().ok_or_else(|| SwapError::MissingParent(path2.to_path_buf()))?;
    
    let name1 = path1.file_name().unwrap();
    let name2 = path2.file_name().unwrap();

    let final_dest1 = parent1.join(name2);
    let final_dest2 = parent2.join(name1);

    let temp_path = generate_temporary_path(path1)?;

	log!(cli, " 1. Renaming '{}' -> '{}' (temporary)", path1.display(), temp_path.display());
    safe_rename(path1, &temp_path)?;
	
    log!(cli, " 2. Renaming '{}' -> '{}' (temporary)", path2.display(), final_dest2.display());
    safe_rename(path2, &final_dest2)?;
    
    log!(cli, " 3. Renaming '{}' (temporary) -> '{}' ", temp_path.display(), final_dest1.display());
    safe_rename(&temp_path, &final_dest1)?;

    Ok(())
}


// --- Helper Functions ---

/// A wrapper around `std::fs::rename` that maps errors to our custom `SwapError` type.
fn safe_rename(from: &Path, to: &Path) -> Result<(), SwapError> {
    fs::rename(from, to).map_err(|e| SwapError::Io(e, from.to_path_buf()))
}

/// Generates a unique temporary path in the same directory as the original path.
fn generate_temporary_path(original_path: &Path) -> Result<PathBuf, SwapError> {
    let parent = original_path.parent().ok_or_else(|| SwapError::MissingParent(original_path.to_path_buf()))?;
    let original_filename = original_path.file_name().unwrap().to_str().unwrap_or("temp");

    let unique_id = uuid::Uuid::new_v4();
    let temp_filename = format!("{}.swap.{}", original_filename, unique_id);
    
    Ok(parent.join(temp_filename))
}
