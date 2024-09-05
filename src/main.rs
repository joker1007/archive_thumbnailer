use std::{
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};

use clap::Parser;
use image::ImageReader;
use log::debug;
use zip::ZipArchive;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value = ".")]
    output_dir: PathBuf,

    #[arg(short, long, default_value = "256")]
    size: u32,
}

fn fetch_cover(archive: &mut ZipArchive<File>) -> Option<Vec<u8>> {
    let cover_index = archive
        .index_for_name("cover.jpg")
        .or(archive.index_for_name("cover.png"));
    debug!("cover_index: {:?}", cover_index);
    return match cover_index {
        Some(index) => {
            let cover = archive.by_index_raw(index);
            match cover {
                Ok(mut file) => {
                    let mut buf = vec![];
                    file.read_to_end(&mut buf).unwrap();
                    debug!("read cover: {:?}", file.name());
                    Some(buf)
                }
                Err(_) => None,
            }
        }
        None => {
            return fetch_first_image(archive);
        }
    };
}

fn is_image(filename: &str) -> bool {
    return filename.ends_with(".jpg")
        || filename.ends_with(".png")
        || filename.ends_with(".jpeg")
        || filename.ends_with(".JPG")
        || filename.ends_with(".PNG")
        || filename.ends_with(".JPEG");
}

fn fetch_first_image<'a>(archive: &'a mut ZipArchive<File>) -> Option<Vec<u8>> {
    let mut buf = vec![];

    debug!("fetch_first_image");
    for i in 0..archive.len() {
        let mut file = archive.by_index_raw(i).unwrap();
        if is_image(file.name()) {
            file.read_to_end(&mut buf).unwrap();
            debug!("read image: {:?}", file.name());
            break;
        }
    }

    return Some(buf);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    let input = std::path::Path::new(&args.input);
    let file = File::open(input)?;

    debug!("input: {:?}", input);
    let mut archive = ZipArchive::new(file)?;
    debug!("read finish: {:?}", input);
    let cover = fetch_cover(&mut archive);
    if cover.is_none() {
        return Err("cover is not found".into());
    }

    let reader = ImageReader::new(Cursor::new(cover.unwrap())).with_guessed_format()?;
    let format = reader.format().ok_or("unknown format")?;
    let image = reader.decode()?;

    let ext = match format {
        image::ImageFormat::Jpeg => ".jpg",
        image::ImageFormat::Png => ".png",
        _ => return Err("unsupported format".into()),
    };

    let output = args.output_dir.join("cover".to_string() + ext);

    output.parent().map(|p| {
        if !p.exists() {
            std::fs::create_dir_all(p).unwrap();
        }
    });

    image.thumbnail(args.size, u32::MAX).save(&output)?;

    let result = std::fs::canonicalize(output)?;
    println!("{}", result.display());

    return Ok(());
}
