#[cfg(feature = "calibre")]
use palmdoc_compression::calibre;
use palmdoc_compression::palmdoc;

fn main() {
    let text = std::fs::read_to_string("resources/war_and_peace.txt").unwrap();
    let text = text.as_bytes().to_vec();

    #[cfg(feature = "calibre")]
    {
        let calibre_compressed = text
            .clone()
            .chunks(4096)
            .flat_map(|chunk| calibre::compress(&chunk))
            .collect::<Vec<_>>();
        println!(
            "Calibre compression ratio: {:.2}",
            text.len() as f64 / calibre_compressed.len() as f64
        );
    }

    let library_compressed = text
        .chunks(4096)
        .flat_map(|chunk| palmdoc::compress_palmdoc(&chunk))
        .collect::<Vec<_>>();
    println!(
        "Custom library compression ratio: {:.2}",
        text.len() as f64 / library_compressed.len() as f64
    );
}
