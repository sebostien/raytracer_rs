use clap::Parser;
use path_absolutize::Absolutize;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

use image::RgbImage;

/// The filename images will be saved as (appended with .png).
const DEFAULT_FILE_NAME: &str = "./raytraced";

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    out_file: Option<String>,
    #[arg(long)]
    width: Option<u32>,
    #[arg(long)]
    height: Option<u32>,
    #[arg(short, long)]
    recurse_depth: Option<usize>,
}

fn main() {
    let args = Args::parse();

    match run_raytracer(args) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn run_raytracer(args: Args) -> Result<String, String> {
    let buf = read_file(args.file)?;

    let mut raytracer =
        scene_parser::parse_string(&buf).map_err(|e| format!("Unable to parse file:\n {e}"))?;

    if let Some(w) = args.width {
        raytracer.set_width(w);
    }

    if let Some(h) = args.height {
        raytracer.set_height(h);
    }

    if let Some(depth) = args.recurse_depth {
        raytracer.set_recurse_depth(depth);
    }

    let out = raytracer.raycast();

    let width = out[0].len() as u32;
    let height = out.len() as u32;

    let mut img = RgbImage::new(width, height);

    for (y, row) in out.iter().enumerate() {
        // Flip image vertically
        let y = height - 1 - y as u32;

        for (x, color) in row.iter().enumerate() {
            let x = x as u32;
            img.put_pixel(x, y, image::Rgb((*color).into()));
        }
    }

    let out_file = if let Some(f) = args.out_file {
        Path::new(&f)
            .absolutize()
            .map_err(|e| e.to_string())?
            .to_path_buf()
    } else {
        find_unique_file_name()?
    };

    create_empty_file(&out_file)?;

    match img.save(&out_file) {
        Ok(_) => Ok(format!("Saved image to {}", out_file.to_string_lossy())),
        Err(e) => Err(format!("Could not save image!\n{e}")),
    }
}

fn read_file(file_name: String) -> Result<String, String> {
    match std::fs::File::open(file_name) {
        Ok(mut f) => {
            let mut buf = String::new();
            match f.read_to_string(&mut buf) {
                Ok(_) => Ok(buf),
                Err(_) => todo!(),
            }
        }
        Err(e) => Err(format!("Could not read input file!\n{e}")),
    }
}

fn create_empty_file<S: AsRef<Path>>(file: S) -> Result<(), String> {
    let file = if file.as_ref().is_absolute() {
        file.as_ref().to_path_buf()
    } else {
        let dir = std::env::current_dir().map_err(|_| {
            format!(
                "Could not save image to {}! Try using a absolute path instead.",
                file.as_ref().to_string_lossy()
            )
        })?;

        Path::new(&dir).join(file)
    };

    if let Err(err) = std::fs::File::create(file) {
        Err(format!("Could not create output file!\n{err}",))
    } else {
        Ok(())
    }
}

fn find_unique_file_name() -> Result<PathBuf, String> {
    let mut name: String = Path::new(&DEFAULT_FILE_NAME.to_string())
        .absolutize()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string()
        + ".png";
    let l = name.len() - 4;
    let mut i = 0;
    while let Ok(true) = Path::new(&name).try_exists() {
        i += 1;
        name.truncate(l);
        name += &format!("-{i}.png");

        if i > 1000 {
            return Err("Could not find a unique name for the file.\nConsider using --out-file and try again.".to_string());
        }
    }
    Ok(name.to_string().into())
}
